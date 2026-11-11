use crate::{
    error::{Error, Result, io},
    manifest::{MANIFEST_LIMIT, Manifest, path_valid},
};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs::{self, File},
    io::{Cursor, Read, Write},
    path::Path,
};
use zip::{CompressionMethod, ZipArchive, ZipWriter, write::SimpleFileOptions};

pub const ARCHIVE_LIMIT: u64 = 256 * 1024 * 1024;
pub const ENTRY_LIMIT: u64 = 64 * 1024 * 1024;
pub const TOTAL_LIMIT: u64 = 128 * 1024 * 1024;
pub const ENTRY_COUNT: usize = 4096;
pub type Entries = BTreeMap<String, Vec<u8>>;
fn invalid(s: impl Into<String>) -> Error {
    Error::new("E_ARCHIVE", s)
}
pub fn hash(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
pub fn bounded_read(path: &Path, limit: u64) -> Result<Vec<u8>> {
    let metadata = fs::metadata(path).map_err(|e| io(format!("stat {}", path.display()), e))?;
    if !metadata.is_file() {
        return Err(Error::new(
            "E_PATH",
            format!("{} must be a regular file", path.display()),
        ));
    }
    let f = File::open(path).map_err(|e| io(format!("open {}", path.display()), e))?;
    if f.metadata().map_err(|e| io("read file metadata", e))?.len() > limit {
        return Err(Error::new(
            "E_LIMIT",
            format!("{} exceeds {limit} bytes", path.display()),
        ));
    }
    let mut bytes = Vec::new();
    f.take(limit + 1)
        .read_to_end(&mut bytes)
        .map_err(|e| io(format!("read {}", path.display()), e))?;
    if bytes.len() as u64 > limit {
        return Err(Error::new(
            "E_LIMIT",
            format!("{} exceeds {limit} bytes", path.display()),
        ));
    }
    Ok(bytes)
}
pub fn validate_wasm(path: &str, bytes: &[u8]) -> Result<()> {
    if !bytes.starts_with(b"\0asm\x01\0\0\0") {
        return Err(Error::new(
            "E_WASM",
            format!("{path}: expected a WebAssembly core module"),
        ));
    }
    wasmparser::Validator::new()
        .validate_all(bytes)
        .map_err(|e| Error::new("E_WASM", format!("{path}: {e}")))?;
    Ok(())
}
pub fn validate_contents(manifest: &Manifest, entries: &Entries) -> Result<()> {
    for module in &manifest.modules {
        let bytes = entries
            .get(&module.path)
            .ok_or_else(|| invalid(format!("declared module '{}' is missing", module.path)))?;
        validate_wasm(&module.path, bytes)?;
    }
    for (name, expected) in &manifest.hashes {
        let bytes = entries
            .get(name)
            .ok_or_else(|| invalid(format!("checksum source '{name}' is missing")))?;
        if hash(bytes) != *expected {
            return Err(Error::new(
                "E_CHECKSUM",
                format!("{name}: SHA-256 does not match sjml.toml"),
            ));
        }
    }
    Ok(())
}
pub fn validate_limits(entries: &Entries) -> Result<()> {
    if entries.len() > ENTRY_COUNT {
        return Err(Error::new(
            "E_LIMIT",
            format!("archive exceeds {ENTRY_COUNT} entries"),
        ));
    }
    let mut total = 0u64;
    for (name, bytes) in entries {
        if !path_valid(name) {
            return Err(Error::new(
                "E_PATH",
                format!("invalid archive path '{name}'"),
            ));
        }
        let limit = if name == "sjml.toml" {
            MANIFEST_LIMIT
        } else {
            ENTRY_LIMIT
        };
        if bytes.len() as u64 > limit {
            return Err(Error::new(
                "E_LIMIT",
                format!("{name} exceeds {limit} bytes"),
            ));
        }
        total += bytes.len() as u64;
        if total > TOTAL_LIMIT {
            return Err(Error::new(
                "E_LIMIT",
                "archive exceeds 128 MiB uncompressed",
            ));
        }
    }
    Ok(())
}
fn u16_at(bytes: &[u8], offset: usize) -> Result<u16> {
    let a = bytes
        .get(offset..offset + 2)
        .ok_or_else(|| invalid("truncated ZIP header"))?;
    Ok(u16::from_le_bytes([a[0], a[1]]))
}
fn u32_at(bytes: &[u8], offset: usize) -> Result<u32> {
    let a = bytes
        .get(offset..offset + 4)
        .ok_or_else(|| invalid("truncated ZIP header"))?;
    Ok(u32::from_le_bytes([a[0], a[1], a[2], a[3]]))
}
// Bound the directory before ZipArchive allocates its index. Also reconcile local
// headers with the central directory: there must be no hidden or overlapping data.
fn preflight(bytes: &[u8]) -> Result<()> {
    let start = bytes.len().saturating_sub(65535 + 22);
    let end = (start..bytes.len().saturating_sub(21))
        .rev()
        .find(|&i| {
            bytes.get(i..i + 4) == Some(b"PK\x05\x06")
                && u16_at(bytes, i + 20).is_ok_and(|n| i + 22 + n as usize == bytes.len())
        })
        .ok_or_else(|| invalid("ZIP end record is missing or archive has trailing data"))?;
    let count = u16_at(bytes, end + 10)? as usize;
    if u16_at(bytes, end + 4)? != 0
        || u16_at(bytes, end + 6)? != 0
        || u16_at(bytes, end + 8)? as usize != count
    {
        return Err(invalid("multi-disk ZIP archives are unsupported"));
    }
    if count > ENTRY_COUNT {
        return Err(Error::new("E_LIMIT", "archive exceeds 4096 entries"));
    }
    let size = u32_at(bytes, end + 12)? as usize;
    let offset = u32_at(bytes, end + 16)? as usize;
    if count == u16::MAX as usize || size == u32::MAX as usize || offset == u32::MAX as usize {
        return Err(invalid(
            "ZIP64 packages are unsupported; recreate the package with sjml package",
        ));
    }
    if offset.checked_add(size) != Some(end) || count > size / 46 {
        return Err(invalid("invalid ZIP central directory bounds"));
    }
    let mut pos = offset;
    let mut ranges = Vec::new();
    let mut paths = BTreeSet::new();
    let mut total = 0u64;
    for _ in 0..count {
        if pos + 46 > end || u32_at(bytes, pos)? != 0x02014b50 {
            return Err(invalid("invalid ZIP central header"));
        }
        let flags = u16_at(bytes, pos + 8)?;
        let method = u16_at(bytes, pos + 10)?;
        let crc = u32_at(bytes, pos + 16)?;
        let compressed = u32_at(bytes, pos + 20)? as usize;
        let plain = u32_at(bytes, pos + 24)? as usize;
        let name_len = u16_at(bytes, pos + 28)? as usize;
        let extra_len = u16_at(bytes, pos + 30)? as usize;
        let comment_len = u16_at(bytes, pos + 32)? as usize;
        if flags & (1 | 64 | 8192) != 0
            || ![0, 8].contains(&method)
            || u16_at(bytes, pos + 34)? != 0
        {
            return Err(invalid("encrypted, split, or unsupported ZIP compression"));
        }
        let next = pos + 46 + name_len + extra_len + comment_len;
        if next > end {
            return Err(invalid("truncated ZIP central entry"));
        }
        let raw_name = &bytes[pos + 46..pos + 46 + name_len];
        let name = std::str::from_utf8(raw_name).map_err(|_| invalid("ZIP name is not UTF-8"))?;
        let normalized = name.strip_suffix('/').unwrap_or(name);
        if !path_valid(normalized) || !paths.insert(normalized.to_owned()) {
            return Err(invalid(format!("invalid or duplicate ZIP path '{name}'")));
        }
        let mode = u32_at(bytes, pos + 38)? >> 16;
        if mode & 0o170000 == 0o120000 {
            return Err(invalid(format!("symlink entry '{name}' is forbidden")));
        }
        let limit = if normalized == "sjml.toml" {
            MANIFEST_LIMIT
        } else {
            ENTRY_LIMIT
        };
        if plain as u64 > limit
            || (name.ends_with('/') && (plain != 0 || normalized == "sjml.toml"))
        {
            return Err(invalid(format!("invalid size or directory for '{name}'")));
        }
        if plain != 0 && (compressed == 0 || plain as u64 > compressed as u64 * 200) {
            return Err(invalid(format!("'{name}' exceeds 200:1 compression ratio")));
        }
        if method == 0 && plain != compressed {
            return Err(invalid("stored ZIP entry has inconsistent sizes"));
        }
        total += plain as u64;
        if total > TOTAL_LIMIT {
            return Err(Error::new(
                "E_LIMIT",
                "archive exceeds 128 MiB uncompressed",
            ));
        }
        let local = u32_at(bytes, pos + 42)? as usize;
        if local + 30 > offset
            || u32_at(bytes, local)? != 0x04034b50
            || flags != u16_at(bytes, local + 6)?
            || method != u16_at(bytes, local + 8)?
        {
            return Err(invalid("ZIP local and central headers disagree"));
        }
        let ln = u16_at(bytes, local + 26)? as usize;
        let lx = u16_at(bytes, local + 28)? as usize;
        let data = local + 30 + ln + lx;
        let data_end = data
            .checked_add(compressed)
            .ok_or_else(|| invalid("ZIP size overflow"))?;
        if data_end > offset || bytes.get(local + 30..local + 30 + ln) != Some(raw_name) {
            return Err(invalid("ZIP local filename or bounds disagree"));
        }
        if method == 8 {
            let mut decoder = flate2::bufread::DeflateDecoder::new(&bytes[data..data_end]);
            let decoded = std::io::copy(
                &mut (&mut decoder).take(plain as u64 + 1),
                &mut std::io::sink(),
            )
            .map_err(|e| invalid(format!("{name}: invalid Deflate stream: {e}")))?;
            let measured = decoder.total_in();
            if decoded != plain as u64 || measured != compressed as u64 {
                return Err(invalid(format!(
                    "{name}: Deflate stream size disagrees with ZIP metadata"
                )));
            }
            if decoded > 0 && (measured == 0 || decoded > measured * 200) {
                return Err(invalid(format!(
                    "{name}: measured Deflate ratio exceeds 200:1"
                )));
            }
        }
        let mut local_end = data_end;
        if flags & 8 == 0 {
            if crc != u32_at(bytes, local + 14)?
                || compressed != u32_at(bytes, local + 18)? as usize
                || plain != u32_at(bytes, local + 22)? as usize
            {
                return Err(invalid("ZIP local size or CRC disagrees"));
            }
        } else {
            let descriptor = if u32_at(bytes, data_end)? == 0x08074b50 {
                data_end + 4
            } else {
                data_end
            };
            if descriptor + 12 > offset
                || crc != u32_at(bytes, descriptor)?
                || compressed != u32_at(bytes, descriptor + 4)? as usize
                || plain != u32_at(bytes, descriptor + 8)? as usize
            {
                return Err(invalid("ZIP data descriptor disagrees"));
            }
            local_end = descriptor + 12;
        }
        ranges.push((local, local_end));
        pos = next;
    }
    if pos != end {
        return Err(invalid("ZIP entry count does not match directory"));
    }
    ranges.sort_unstable();
    let mut expected = 0;
    for (start, end) in ranges {
        if start != expected {
            return Err(invalid("ZIP local entries overlap or contain hidden data"));
        }
        expected = end;
    }
    if expected != offset {
        return Err(invalid("ZIP has undeclared local data"));
    }
    Ok(())
}
pub fn read(path: &Path) -> Result<(Manifest, Entries)> {
    let bytes = bounded_read(path, ARCHIVE_LIMIT)?;
    preflight(&bytes)?;
    let mut zip =
        ZipArchive::new(Cursor::new(bytes)).map_err(|e| invalid(format!("invalid ZIP: {e}")))?;
    let mut entries = Entries::new();
    for index in 0..zip.len() {
        let mut entry = zip
            .by_index(index)
            .map_err(|e| invalid(format!("invalid ZIP entry: {e}")))?;
        let name = std::str::from_utf8(entry.name_raw())
            .map_err(|_| invalid("ZIP name is not UTF-8"))?
            .to_owned();
        if entry.is_dir() {
            continue;
        }
        let expected = entry.size();
        let mut contents = Vec::new();
        (&mut entry)
            .take(expected + 1)
            .read_to_end(&mut contents)
            .map_err(|e| invalid(format!("{name}: invalid data or CRC: {e}")))?;
        if contents.len() as u64 != expected {
            return Err(invalid(format!("{name}: ZIP entry size mismatch")));
        }
        entries.insert(name, contents);
    }
    let manifest = Manifest::parse(
        entries
            .get("sjml.toml")
            .ok_or_else(|| invalid("archive does not contain sjml.toml"))?,
    )?;
    validate_contents(&manifest, &entries)?;
    Ok((manifest, entries))
}
pub fn write(path: &Path, entries: &Entries) -> Result<()> {
    validate_limits(entries)?;
    let parent = path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    fs::create_dir_all(parent).map_err(|e| io(format!("create {}", parent.display()), e))?;
    let mut temporary =
        tempfile::NamedTempFile::new_in(parent).map_err(|e| io("create temporary package", e))?;
    {
        let mut zip = ZipWriter::new(temporary.as_file_mut());
        let options = SimpleFileOptions::default()
            .compression_method(CompressionMethod::Stored)
            .last_modified_time(zip::DateTime::default())
            .unix_permissions(0o644);
        for (name, bytes) in entries {
            zip.start_file(name, options)
                .map_err(|e| invalid(format!("write {name}: {e}")))?;
            zip.write_all(bytes)
                .map_err(|e| io(format!("write {name}"), e))?;
        }
        zip.finish()
            .map_err(|e| invalid(format!("finish package: {e}")))?;
    }
    // Validate the exact artifact before atomically replacing a prior package.
    read(temporary.path())?;
    temporary
        .as_file()
        .sync_all()
        .map_err(|e| io("sync package", e))?;
    temporary
        .persist(path)
        .map_err(|e| io(format!("publish {}", path.display()), e.error))?;
    Ok(())
}
