use crate::error::{Error, Result};
use base64::{Engine, engine::general_purpose::STANDARD};
use std::collections::{BTreeMap, BTreeSet};
use toml::{Value, map::Map};
use unicode_normalization::UnicodeNormalization;

pub const MANIFEST_LIMIT: u64 = 1024 * 1024;
type Table = Map<String, Value>;
#[derive(Debug, Clone)]
pub struct Module {
    pub id: String,
    pub path: String,
}
#[derive(Debug, Clone)]
pub struct Manifest {
    pub document: Value,
    pub id: String,
    pub version: String,
    pub modules: Vec<Module>,
    pub hashes: BTreeMap<String, String>,
}
fn invalid(message: impl Into<String>) -> Error {
    Error::new("E_MANIFEST", message)
}
fn require(condition: bool, message: impl Into<String>) -> Result<()> {
    if condition {
        Ok(())
    } else {
        Err(invalid(message))
    }
}
pub fn path_valid(path: &str) -> bool {
    !path.is_empty()
        && path.encode_utf16().count() <= 1024
        && !path.contains(['\\', ':'])
        && !path.chars().any(char::is_control)
        && path.nfc().eq(path.chars())
        && path
            .split('/')
            .all(|s| !s.is_empty() && s != "." && s != "..")
}
pub fn identifier(id: &str) -> bool {
    (1..=64).contains(&id.len())
        && id.as_bytes()[0].is_ascii_lowercase()
        && id
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_' || b == b'-')
}
fn named(id: &str) -> bool {
    id.split('.').all(|p| {
        !p.is_empty()
            && p.as_bytes()[0].is_ascii_lowercase()
            && p.bytes()
                .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_' || b == b'-')
    })
}
fn keys(table: &Table, allowed: &[&str], context: &str) -> Result<()> {
    for key in table.keys() {
        require(
            allowed.contains(&key.as_str()),
            format!("{context}: unknown key '{key}'"),
        )?;
    }
    Ok(())
}
fn table<'a>(value: &'a Value, context: &str) -> Result<&'a Table> {
    value
        .as_table()
        .ok_or_else(|| invalid(format!("{context}: expected table")))
}
fn field<'a>(table: &'a Table, key: &str) -> Result<&'a Value> {
    table
        .get(key)
        .ok_or_else(|| invalid(format!("missing '{key}'")))
}
fn text<'a>(table: &'a Table, key: &str) -> Result<&'a str> {
    let s = field(table, key)?
        .as_str()
        .ok_or_else(|| invalid(format!("{key}: expected string")))?;
    require(
        !s.trim().is_empty() && s.encode_utf16().count() <= 4096,
        format!("{key}: expected nonempty string of at most 4096 characters"),
    )?;
    Ok(s)
}
fn optional_text<'a>(table: &'a Table, key: &str) -> Result<Option<&'a str>> {
    if let Some(v) = table.get(key) {
        let s = v
            .as_str()
            .ok_or_else(|| invalid(format!("{key}: expected string")))?;
        require(
            s.encode_utf16().count() <= 4096,
            format!("{key}: string too long"),
        )?;
        Ok(Some(s))
    } else {
        Ok(None)
    }
}
fn array<'a>(table: &'a Table, key: &str, limit: usize) -> Result<&'a [Value]> {
    match table.get(key) {
        None => Ok(&[]),
        Some(v) => {
            let a = v
                .as_array()
                .ok_or_else(|| invalid(format!("{key}: expected array")))?;
            require(a.len() <= limit, format!("{key}: exceeds {limit} entries"))?;
            Ok(a)
        }
    }
}
fn strings<'a>(table: &'a Table, key: &str) -> Result<Vec<&'a str>> {
    let mut seen = BTreeSet::new();
    for value in array(table, key, 256)? {
        let s = value
            .as_str()
            .ok_or_else(|| invalid(format!("{key}: expected string array")))?;
        require(
            !s.trim().is_empty() && s.encode_utf16().count() <= 4096,
            format!("{key}: invalid string"),
        )?;
        require(seen.insert(s), format!("{key}: duplicate '{s}'"))?;
    }
    Ok(seen.into_iter().collect())
}
fn integer(table: &Table, key: &str, min: i64, max: i64) -> Result<i64> {
    let n = field(table, key)?
        .as_integer()
        .ok_or_else(|| invalid(format!("{key}: expected integer")))?;
    require(
        n >= min && n <= max,
        format!("{key}: must be in {min}..={max}"),
    )?;
    Ok(n)
}
fn optional_integer(table: &Table, key: &str, min: i64, max: i64, default: i64) -> Result<i64> {
    if table.contains_key(key) {
        integer(table, key, min, max)
    } else {
        Ok(default)
    }
}
fn numeric(s: &str) -> bool {
    !s.is_empty() && s.bytes().all(|b| b.is_ascii_digit()) && (s.len() == 1 || !s.starts_with('0'))
}
fn version(s: &str) -> bool {
    if s.len() > 512 {
        return false;
    }
    let mut build = s.split('+');
    let base = build.next().unwrap_or("");
    if let Some(suffix) = build.next()
        && !suffix
            .split('.')
            .all(|p| !p.is_empty() && p.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-'))
    {
        return false;
    }
    if build.next().is_some() {
        return false;
    }
    let (core, pre) = base
        .split_once('-')
        .map_or((base, None), |(a, b)| (a, Some(b)));
    let nums: Vec<_> = core.split('.').collect();
    nums.len() == 3
        && nums.into_iter().all(numeric)
        && pre.is_none_or(|p| {
            p.split('.').all(|s| {
                !s.is_empty()
                    && s.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-')
                    && (!s.bytes().all(|b| b.is_ascii_digit()) || numeric(s))
            })
        })
}
fn constraint(s: &str) -> bool {
    let s = s.trim();
    if s == "*" {
        return true;
    }
    if let Some(range) = s.strip_prefix('^').or_else(|| s.strip_prefix('~')) {
        let parts: Vec<_> = range.split('.').collect();
        version(range)
            || (range.len() <= 512
                && (1..=2).contains(&parts.len())
                && parts.into_iter().all(numeric))
    } else {
        version(s)
    }
}
fn side(table: &Table) -> Result<&str> {
    let s = optional_text(table, "side")?.unwrap_or("both");
    require(
        ["client", "server", "both"].contains(&s),
        "side: must be client, server, or both",
    )?;
    Ok(s)
}
fn metadata(value: &Value, depth: usize, nodes: &mut usize) -> Result<()> {
    *nodes += 1;
    require(
        depth <= 8 && *nodes <= 4096,
        "mod.metadata: nesting or node limit exceeded",
    )?;
    match value {
        Value::String(s) => require(s.encode_utf16().count() <= 4096, "metadata string too long")?,
        Value::Array(values) => {
            require(values.len() <= 256, "metadata array too large")?;
            for value in values {
                metadata(value, depth + 1, nodes)?;
            }
        }
        Value::Table(values) => {
            require(values.len() <= 256, "metadata table too large")?;
            for value in values.values() {
                metadata(value, depth + 1, nodes)?;
            }
        }
        _ => {}
    }
    Ok(())
}

fn validate_command_declarations(
    declarations: &[Value],
    key: &str,
    names: &mut BTreeSet<String>,
    client_command: bool,
    requested_permissions: &[&str],
) -> Result<()> {
    let context = format!("modules.{key}");
    for value in declarations {
        let command = table(value, &context)?;
        keys(
            command,
            &["name", "schema_version", "permission_level", "arguments"],
            &context,
        )?;
        let name = text(command, "name")?;
        require(
            name.len() <= 128
                && (name.as_bytes()[0].is_ascii_lowercase() || name.as_bytes()[0].is_ascii_digit())
                && name.bytes().all(|byte| {
                    byte.is_ascii_lowercase() || byte.is_ascii_digit() || b"._-".contains(&byte)
                })
                && names.insert(name.to_owned()),
            format!("{context}.name: invalid or duplicate callback name"),
        )?;
        integer(command, "schema_version", 1, u32::MAX as i64)?;
        integer(command, "permission_level", 0, 4)?;
        let arguments = array(command, "arguments", 16)?;
        let mut argument_names = BTreeSet::new();
        for (index, value) in arguments.iter().enumerate() {
            let argument = table(value, "command argument")?;
            keys(
                argument,
                &["name", "type", "min", "max", "suggestions"],
                "command argument",
            )?;
            let name = text(argument, "name")?;
            require(
                identifier(name) && !name.contains('-') && argument_names.insert(name),
                "argument.name: invalid or duplicate name",
            )?;
            let kind = text(argument, "type")?;
            require(
                ["word", "string", "greedy_string", "integer", "boolean"].contains(&kind),
                "argument.type: unsupported type",
            )?;
            require(
                kind != "greedy_string" || index + 1 == arguments.len(),
                "greedy_string must be final argument",
            )?;
            require(
                kind == "integer"
                    || (!argument.contains_key("min") && !argument.contains_key("max")),
                "only integer arguments accept min/max",
            )?;
            let suggestions = match argument.get("suggestions") {
                Some(value) => value
                    .as_bool()
                    .ok_or_else(|| invalid("argument.suggestions: expected boolean"))?,
                None => false,
            };
            require(
                !client_command
                    || !suggestions
                    || requested_permissions.contains(&"client.command.suggest"),
                "client command suggestions require the requested client.command.suggest capability",
            )?;
            let low = if argument.contains_key("min") {
                integer(argument, "min", i32::MIN as i64, i32::MAX as i64)?
            } else {
                i32::MIN as i64
            };
            let high = if argument.contains_key("max") {
                integer(argument, "max", i32::MIN as i64, i32::MAX as i64)?
            } else {
                i32::MAX as i64
            };
            require(low <= high, "argument min exceeds max")?;
        }
    }
    Ok(())
}

impl Manifest {
    pub fn parse(bytes: &[u8]) -> Result<Self> {
        require(
            bytes.len() as u64 <= MANIFEST_LIMIT,
            "sjml.toml exceeds 1 MiB",
        )?;
        let source = std::str::from_utf8(bytes).map_err(|_| invalid("sjml.toml is not UTF-8"))?;
        let document: Value =
            toml::from_str(source).map_err(|e| invalid(format!("sjml.toml: {e}")))?;
        let root = table(&document, "manifest")?;
        keys(
            root,
            &[
                "mod",
                "runtime",
                "modules",
                "dependencies",
                "conflicts",
                "files",
                "signature",
            ],
            "manifest",
        )?;
        let info = table(field(root, "mod")?, "mod")?;
        keys(
            info,
            &[
                "id",
                "name",
                "version",
                "authors",
                "description",
                "license",
                "metadata",
                "distribution",
            ],
            "mod",
        )?;
        let id = text(info, "id")?.to_owned();
        require(
            identifier(&id),
            "mod.id: expected lowercase identifier of at most 64 bytes",
        )?;
        text(info, "name")?;
        let ver = text(info, "version")?.to_owned();
        require(version(&ver), "mod.version: invalid SemVer")?;
        strings(info, "authors")?;
        optional_text(info, "description")?;
        optional_text(info, "license")?;
        if let Some(v) = info.get("metadata") {
            table(v, "mod.metadata")?;
            metadata(v, 0, &mut 0)?;
        }
        require(
            ["optional", "server_only", "client_only", "both_required"]
                .contains(&optional_text(info, "distribution")?.unwrap_or("optional")),
            "mod.distribution: must be optional, server_only, client_only, or both_required",
        )?;
        let runtime = table(field(root, "runtime")?, "runtime")?;
        keys(runtime, &["abi", "minecraft", "sjml"], "runtime")?;
        require(
            constraint(text(runtime, "abi")?),
            "runtime.abi: invalid compatibility constraint",
        )?;
        for key in ["minecraft", "sjml"] {
            if runtime.contains_key(key) {
                require(
                    constraint(text(runtime, key)?),
                    format!("runtime.{key}: invalid compatibility constraint"),
                )?;
            }
        }
        let mut modules = Vec::new();
        let mut module_ids = BTreeSet::new();
        let mut dependencies = Vec::new();
        let mut channel_count = 0;
        for v in array(root, "modules", 256)? {
            let m = table(v, "modules")?;
            keys(
                m,
                &[
                    "id",
                    "path",
                    "side",
                    "depends",
                    "permissions",
                    "provides",
                    "exports",
                    "commands",
                    "client_commands",
                    "channels",
                ],
                "modules",
            )?;
            let module_id = text(m, "id")?;
            require(
                identifier(module_id) && module_ids.insert(module_id.to_owned()),
                "modules.id: invalid or duplicate module identifier",
            )?;
            let path = text(m, "path")?;
            require(
                path_valid(path) && path.ends_with(".wasm"),
                "modules.path: invalid relative WASM path",
            )?;
            let side = side(m)?;
            let channels = array(m, "channels", 1024)?;
            channel_count += channels.len();
            require(
                channel_count <= 1024,
                "modules.channels: exceeds 1024 channels per manifest",
            )?;
            let mut channel_names = BTreeSet::new();
            for value in channels {
                let channel = table(value, "modules.channels")?;
                keys(
                    channel,
                    &[
                        "name",
                        "schema_version",
                        "directions",
                        "messages_per_second",
                        "burst",
                    ],
                    "modules.channels",
                )?;
                let name = text(channel, "name")?;
                require(
                    name.len() <= 128 && named(name) && channel_names.insert(name),
                    "modules.channels.name: invalid or duplicate dot-separated lowercase identifier",
                )?;
                integer(channel, "schema_version", 1, u32::MAX as i64)?;
                optional_integer(channel, "messages_per_second", 1, 1000, 100)?;
                optional_integer(channel, "burst", 1, 2000, 200)?;
                let directions = strings(channel, "directions")?;
                require(
                    !directions.is_empty()
                        && directions.len() <= 2
                        && directions
                            .iter()
                            .all(|name| ["client_to_server", "server_to_client"].contains(name)),
                    "modules.channels.directions: expected one or two distinct client_to_server/server_to_client values",
                )?;
            }
            for name in strings(m, "depends")? {
                require(identifier(name), "modules.depends: invalid identifier")?;
                dependencies.push(name.to_owned());
            }
            let requested_permissions = strings(m, "permissions")?;
            for name in requested_permissions
                .iter()
                .copied()
                .chain(strings(m, "provides")?)
            {
                require(
                    named(name),
                    "modules.permissions/provides: invalid capability/service name",
                )?;
            }
            let mut exports = BTreeSet::new();
            for v in array(m, "exports", 256)? {
                let e = table(v, "modules.exports")?;
                keys(e, &["id", "version", "visibility"], "modules.exports")?;
                let id = text(e, "id")?;
                require(
                    named(id) && exports.insert(id),
                    "modules.exports.id: invalid or duplicate name",
                )?;
                require(
                    version(text(e, "version")?),
                    "modules.exports.version: invalid SemVer",
                )?;
                require(
                    ["public", "private"]
                        .contains(&optional_text(e, "visibility")?.unwrap_or("private")),
                    "modules.exports.visibility: invalid visibility",
                )?;
            }
            let commands = array(m, "commands", 256)?;
            let client_commands = array(m, "client_commands", 256)?;
            require(
                commands.is_empty() || side != "client",
                "client-only modules cannot declare server commands",
            )?;
            require(
                client_commands.is_empty() || side != "server",
                "server-only modules cannot declare client commands",
            )?;
            require(
                commands.len() + client_commands.len() <= 256,
                "modules.commands: server and client commands combined exceed 256 entries per module",
            )?;
            let mut names = BTreeSet::new();
            validate_command_declarations(
                commands,
                "commands",
                &mut names,
                false,
                &requested_permissions,
            )?;
            validate_command_declarations(
                client_commands,
                "client_commands",
                &mut names,
                true,
                &requested_permissions,
            )?;
            modules.push(Module {
                id: module_id.to_owned(),
                path: path.to_owned(),
            });
        }
        require(
            !modules.is_empty(),
            "modules: at least one WASM module is required",
        )?;
        for dep in dependencies {
            require(
                module_ids.contains(&dep),
                format!("undeclared module dependency '{dep}'"),
            )?;
        }
        let mut dependency_ids = BTreeSet::new();
        for v in array(root, "dependencies", 256)? {
            let d = table(v, "dependencies")?;
            keys(d, &["id", "version", "optional", "side"], "dependencies")?;
            let name = text(d, "id")?;
            require(
                identifier(name) && dependency_ids.insert(name),
                "dependencies.id: invalid or duplicate identifier",
            )?;
            require(
                constraint(text(d, "version")?),
                "dependencies.version: invalid constraint",
            )?;
            side(d)?;
            if let Some(v) = d.get("optional") {
                require(v.is_bool(), "dependencies.optional: expected boolean")?;
            }
        }
        for name in strings(root, "conflicts")? {
            require(
                identifier(name) && name != id,
                "conflicts: invalid identifier or self conflict",
            )?;
        }
        let mut hashes = BTreeMap::new();
        for v in array(root, "files", 256)? {
            let f = table(v, "files")?;
            keys(f, &["path", "sha256"], "files")?;
            let path = text(f, "path")?;
            require(path_valid(path), "files.path: invalid archive path")?;
            let hash = text(f, "sha256")?;
            require(
                hash.len() == 64 && hash.bytes().all(|b| b.is_ascii_hexdigit()),
                "files.sha256: expected 64 hex digits",
            )?;
            require(
                hashes
                    .insert(path.to_owned(), hash.to_ascii_lowercase())
                    .is_none(),
                "files.path: duplicate checksum",
            )?;
        }
        if let Some(v) = root.get("signature") {
            let s = table(v, "signature")?;
            keys(
                s,
                &["algorithm", "key_id", "public_key", "value"],
                "signature",
            )?;
            require(
                text(s, "algorithm")? == "ed25519",
                "signature.algorithm: only ed25519 is supported",
            )?;
            let key = text(s, "key_id")?;
            require(
                key.len() <= 128
                    && key.as_bytes()[0].is_ascii_alphanumeric()
                    && key
                        .bytes()
                        .all(|b| b.is_ascii_alphanumeric() || b"._-".contains(&b)),
                "signature.key_id: invalid identifier",
            )?;
            for (key, len) in [("public_key", 32), ("value", 64)] {
                let encoded = text(s, key)?;
                let bytes = STANDARD
                    .decode(encoded)
                    .map_err(|_| invalid(format!("signature.{key}: invalid Base64")))?;
                require(
                    bytes.len() == len && STANDARD.encode(&bytes) == encoded,
                    format!("signature.{key}: expected canonical Base64 encoding of {len} bytes"),
                )?;
            }
        }
        Ok(Self {
            document,
            id,
            version: ver,
            modules,
            hashes,
        })
    }
    pub fn with_hashes(&self, hashes: BTreeMap<String, String>) -> Result<Vec<u8>> {
        require(
            self.document.get("signature").is_none(),
            "cannot repackage signature metadata; remove it and sign the final archive separately",
        )?;
        let mut document = self.document.clone();
        let files = hashes
            .into_iter()
            .map(|(path, hash)| {
                let mut record = Table::new();
                record.insert("path".into(), Value::String(path));
                record.insert("sha256".into(), Value::String(hash));
                Value::Table(record)
            })
            .collect();
        document
            .as_table_mut()
            .expect("validated table")
            .insert("files".into(), Value::Array(files));
        let text = toml::to_string(&document).map_err(|e| invalid(e.to_string()))?;
        Self::parse(text.as_bytes())?;
        Ok(text.into_bytes())
    }
}
