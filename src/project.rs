use crate::{
    archive::{self, ENTRY_COUNT, ENTRY_LIMIT, Entries, TOTAL_LIMIT},
    error::{Error, Result, io},
    manifest::{MANIFEST_LIMIT, Manifest, identifier, path_valid},
};
use serde::Deserialize;
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    io::{BufRead, BufReader},
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

#[derive(Clone, clap::Args)]
pub struct ProjectArgs {
    /// Directory containing sjml.toml and Cargo.toml.
    #[arg(long, default_value = ".")]
    pub project: PathBuf,
    /// Use Cargo's release profile (debug is the default).
    #[arg(long)]
    pub release: bool,
    /// Forbid Cargo network access; all dependencies must already be cached.
    #[arg(long)]
    pub offline: bool,
    /// Require an existing, unchanged Cargo.lock.
    #[arg(long)]
    pub locked: bool,
}
#[derive(Deserialize, Default)]
#[serde(deny_unknown_fields)]
struct ProjectConfig {
    #[serde(default)]
    crates: Vec<CrateMapping>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct CrateMapping {
    module: String,
    package: String,
}
#[derive(Deserialize)]
struct Metadata {
    packages: Vec<CargoPackage>,
    workspace_members: Vec<String>,
}
#[derive(Deserialize)]
struct CargoPackage {
    id: String,
    name: String,
    targets: Vec<CargoTarget>,
}
#[derive(Deserialize)]
struct CargoTarget {
    name: String,
    crate_types: Vec<String>,
}
pub struct Project {
    pub root: PathBuf,
    pub manifest: Manifest,
}
fn error(s: impl Into<String>) -> Error {
    Error::new("E_PROJECT", s)
}
fn reject_link(path: &Path) -> Result<()> {
    let meta = fs::symlink_metadata(path).map_err(|e| io(format!("stat {}", path.display()), e))?;
    if meta.file_type().is_symlink() {
        return Err(Error::new(
            "E_PATH",
            format!("symlink source {} is forbidden", path.display()),
        ));
    }
    Ok(())
}
fn safe_path(root: &Path, relative: &str, existing: bool) -> Result<PathBuf> {
    if !path_valid(relative) {
        return Err(Error::new(
            "E_PATH",
            format!("invalid project path '{relative}'"),
        ));
    }
    let mut path = root.to_owned();
    let parts: Vec<_> = relative.split('/').collect();
    for (i, part) in parts.iter().enumerate() {
        path.push(part);
        match fs::symlink_metadata(&path) {
            Ok(meta) => {
                if meta.file_type().is_symlink() || (i + 1 < parts.len() && !meta.is_dir()) {
                    return Err(Error::new(
                        "E_PATH",
                        format!("{} is a symlink or invalid parent", path.display()),
                    ));
                }
            }
            Err(e) if !existing && e.kind() == std::io::ErrorKind::NotFound => {}
            Err(e) => return Err(io(format!("stat {}", path.display()), e)),
        }
    }
    Ok(path)
}
impl Project {
    pub fn load(path: &Path) -> Result<Self> {
        let root = path
            .canonicalize()
            .map_err(|e| io(format!("open project {}", path.display()), e))?;
        let source = safe_path(&root, "sjml.toml", true)?;
        let manifest = Manifest::parse(&archive::bounded_read(&source, MANIFEST_LIMIT)?)?;
        Ok(Self { root, manifest })
    }
    fn command(&self, args: &ProjectArgs, subcommand: &str) -> Result<Command> {
        let cargo_path = safe_path(&self.root, "Cargo.toml", true)?;
        let mut c = Command::new("cargo");
        c.current_dir(&self.root)
            .arg(subcommand)
            .arg("--manifest-path")
            .arg(cargo_path);
        if args.offline {
            c.arg("--offline");
        }
        if args.locked {
            c.arg("--locked");
        }
        Ok(c)
    }
    fn mapping(&self, args: &ProjectArgs) -> Result<Vec<(String, String)>> {
        let config_path = self.root.join("sjml-project.toml");
        let config: ProjectConfig = if config_path
            .try_exists()
            .map_err(|e| io("check sjml-project.toml", e))?
        {
            reject_link(&config_path)?;
            let bytes = archive::bounded_read(&config_path, MANIFEST_LIMIT)?;
            let source =
                std::str::from_utf8(&bytes).map_err(|_| error("sjml-project.toml is not UTF-8"))?;
            toml::from_str(source).map_err(|e| error(format!("sjml-project.toml: {e}")))?
        } else {
            ProjectConfig::default()
        };
        let output = self
            .command(args, "metadata")?
            .args(["--no-deps", "--format-version", "1"])
            .output()
            .map_err(|e| {
                Error::new(
                    "E_CARGO",
                    format!("cannot start cargo: {e}; install Rust and add cargo to PATH"),
                )
            })?;
        if !output.status.success() {
            return Err(Error::new(
                "E_CARGO",
                format!(
                    "cargo metadata failed: {}",
                    String::from_utf8_lossy(&output.stderr)
                ),
            ));
        }
        let metadata: Metadata = serde_json::from_slice(&output.stdout)
            .map_err(|e| Error::new("E_CARGO", format!("invalid cargo metadata: {e}")))?;
        let members: Vec<_> = metadata
            .packages
            .iter()
            .filter(|p| metadata.workspace_members.contains(&p.id))
            .collect();
        let mut explicit = BTreeMap::new();
        for item in config.crates {
            if !self.manifest.modules.iter().any(|m| m.id == item.module)
                || explicit.insert(item.module, item.package).is_some()
            {
                return Err(error(
                    "sjml-project.toml: unknown or duplicate module mapping",
                ));
            }
        }
        let mut result = Vec::new();
        for module in &self.manifest.modules {
            let selected = if let Some(package) = explicit.get(&module.id) {
                members.iter().copied().find(|p| &p.name == package)
            } else if members.len() == 1 && self.manifest.modules.len() == 1 {
                Some(members[0])
            } else {
                let candidates: Vec<_> = members
                    .iter()
                    .copied()
                    .filter(|p| p.name.replace('-', "_") == module.id.replace('-', "_"))
                    .collect();
                if candidates.len() > 1 {
                    return Err(error(format!(
                        "module '{}': ambiguous Cargo package names; add an explicit [[crates]] mapping in sjml-project.toml",
                        module.id
                    )));
                }
                candidates.first().copied()
            };
            let package = selected.ok_or_else(|| error(format!("module '{}': declare [[crates]] module = '{}' and package = '<Cargo package>' in sjml-project.toml",module.id,module.id)))?;
            let libraries: Vec<_> = package
                .targets
                .iter()
                .filter(|t| t.crate_types.iter().any(|t| t == "cdylib"))
                .collect();
            if libraries.len() != 1 {
                return Err(error(format!(
                    "Cargo package '{}' must have exactly one cdylib target",
                    package.name
                )));
            }
            result.push((package.id.clone(), libraries[0].name.clone()));
        }
        Ok(result)
    }
    pub fn cargo(&self, args: &ProjectArgs, action: &str) -> Result<()> {
        if action == "test" {
            let mut command = self.command(args, "test")?;
            command.arg("--workspace");
            if args.release {
                command.arg("--release");
            }
            self.run_cargo(&mut command)?;
            println!("native Rust tests passed for {}", self.manifest.id);
            return Ok(());
        }
        let mapping = self.mapping(args)?;
        let mut c = self.command(args, action)?;
        c.args(["--target", "wasm32-unknown-unknown", "--lib"]);
        if args.release {
            c.arg("--release");
        }
        let packages: BTreeSet<_> = mapping.iter().map(|(id, _)| id.clone()).collect();
        for id in packages {
            c.arg("--package").arg(id);
        }
        if action == "build" {
            for module in &self.manifest.modules {
                safe_path(&self.root, &module.path, false)?;
            }
            c.arg("--message-format=json-render-diagnostics");
            let mut child = c
                .stdout(Stdio::piped())
                .stderr(Stdio::inherit())
                .spawn()
                .map_err(|e| Error::new("E_CARGO", format!("cannot start cargo: {e}")))?;
            let stdout = child.stdout.take().expect("piped cargo stdout");
            let mut artifacts = BTreeMap::new();
            let mut parse_error = None;
            for line in BufReader::new(stdout).lines() {
                match line {
                    Ok(line) => {
                        let value: serde_json::Value = match serde_json::from_str(&line) {
                            Ok(v) => v,
                            Err(_) => continue,
                        };
                        if value["reason"] == "compiler-artifact"
                            && value["target"]["crate_types"]
                                .as_array()
                                .is_some_and(|a| a.iter().any(|v| v == "cdylib"))
                            && let (Some(id), Some(name), Some(files)) = (
                                value["package_id"].as_str(),
                                value["target"]["name"].as_str(),
                                value["filenames"].as_array(),
                            )
                        {
                            for path in files
                                .iter()
                                .filter_map(|v| v.as_str())
                                .filter(|s| s.ends_with(".wasm"))
                            {
                                artifacts
                                    .insert((id.to_owned(), name.to_owned()), PathBuf::from(path));
                            }
                        }
                    }
                    Err(e) => {
                        parse_error = Some(io("read cargo output", e));
                        break;
                    }
                }
            }
            let status = child.wait().map_err(|e| io("wait for cargo", e))?;
            if let Some(e) = parse_error {
                return Err(e);
            }
            if !status.success() {
                return Err(Error::new(
                    "E_CARGO",
                    "cargo build failed; inspect the compiler diagnostics above; for a missing target run rustup target add wasm32-unknown-unknown",
                ));
            }
            // Read and validate every artifact before replacing project module outputs.
            let mut compiled = Entries::new();
            for (module, key) in self.manifest.modules.iter().zip(mapping) {
                let artifact = artifacts.get(&key).ok_or_else(|| {
                    Error::new(
                        "E_CARGO",
                        format!(
                            "cargo did not emit a WASM artifact for module '{}'",
                            module.id
                        ),
                    )
                })?;
                let bytes = archive::bounded_read(artifact, ENTRY_LIMIT)?;
                archive::validate_wasm(&module.path, &bytes)?;
                if let Some(previous) = compiled.insert(module.path.clone(), bytes.clone())
                    && previous != bytes
                {
                    return Err(error(format!(
                        "multiple crates map to '{}' with different bytes",
                        module.path
                    )));
                }
            }
            archive::validate_limits(&compiled)?;
            for (name, bytes) in compiled {
                let path = safe_path(&self.root, &name, false)?;
                let parent = path.parent().expect("relative module has parent");
                fs::create_dir_all(parent).map_err(|e| io("create module output directory", e))?;
                let mut temp = tempfile::NamedTempFile::new_in(parent)
                    .map_err(|e| io("create module output", e))?;
                std::io::Write::write_all(&mut temp, &bytes)
                    .map_err(|e| io("write module output", e))?;
                temp.persist(&path)
                    .map_err(|e| io("publish module output", e.error))?;
            }
        } else {
            self.run_cargo(&mut c)?;
        }
        println!("{action} succeeded for {}", self.manifest.id);
        Ok(())
    }
    fn run_cargo(&self, c: &mut Command) -> Result<()> {
        let status = c.status().map_err(|e| {
            Error::new(
                "E_CARGO",
                format!("cannot start cargo: {e}; install Rust and add cargo to PATH"),
            )
        })?;
        if !status.success() {
            return Err(Error::new(
                "E_CARGO",
                "Cargo command failed; inspect the compiler diagnostics above",
            ));
        }
        Ok(())
    }
    fn add_file(&self, name: &str, entries: &mut Entries, total: &mut u64) -> Result<()> {
        if entries.contains_key(name) {
            return Ok(());
        }
        if entries.len() >= ENTRY_COUNT - 1 {
            return Err(Error::new("E_LIMIT", "too many package entries"));
        }
        let path = safe_path(&self.root, name, true)?;
        let metadata = fs::metadata(&path).map_err(|e| io("stat package source", e))?;
        if !metadata.is_file() {
            return Err(Error::new(
                "E_PATH",
                format!("{} must be a regular file", path.display()),
            ));
        }
        let remaining = TOTAL_LIMIT.saturating_sub(*total);
        let bytes = archive::bounded_read(&path, ENTRY_LIMIT.min(remaining))?;
        *total += bytes.len() as u64;
        entries.insert(name.to_owned(), bytes);
        Ok(())
    }
    fn tree(
        &self,
        relative: &str,
        entries: &mut Entries,
        total: &mut u64,
        walked: &mut usize,
    ) -> Result<()> {
        *walked += 1;
        if *walked > ENTRY_COUNT {
            return Err(Error::new(
                "E_LIMIT",
                "asset source walk exceeds 4096 files and directories",
            ));
        }
        let path = safe_path(&self.root, relative, true)?;
        let metadata = fs::metadata(&path).map_err(|e| io("stat asset", e))?;
        if metadata.is_file() {
            return self.add_file(relative, entries, total);
        }
        if !metadata.is_dir() {
            return Err(Error::new(
                "E_PATH",
                format!("{} is not a regular file or directory", path.display()),
            ));
        }
        let mut children = Vec::new();
        for child in fs::read_dir(&path).map_err(|e| io("read assets directory", e))? {
            let child = child.map_err(|e| io("read directory entry", e))?;
            let child = child
                .file_name()
                .into_string()
                .map_err(|_| Error::new("E_PATH", "asset filename is not UTF-8"))?;
            children.push(child);
            if children.len() > ENTRY_COUNT {
                return Err(Error::new("E_LIMIT", "asset directory exceeds entry limit"));
            }
        }
        children.sort();
        for child in children {
            self.tree(&format!("{relative}/{child}"), entries, total, walked)?;
        }
        Ok(())
    }
    pub fn entries(&self) -> Result<Entries> {
        let mut entries = Entries::new();
        let mut total = 0;
        for module in &self.manifest.modules {
            self.add_file(&module.path, &mut entries, &mut total)?;
        }
        for path in self.manifest.hashes.keys() {
            if path == "sjml.toml" {
                return Err(error("sjml.toml cannot declare a checksum for itself"));
            }
            self.add_file(path, &mut entries, &mut total)?;
        }
        let mut walked = 0;
        for root in ["assets", "data", "config"] {
            match fs::symlink_metadata(self.root.join(root)) {
                Ok(_) => self.tree(root, &mut entries, &mut total, &mut walked)?,
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
                Err(e) => return Err(io("stat assets directory", e)),
            }
        }
        // Old checksum declarations are regenerated after successful builds.
        for module in &self.manifest.modules {
            archive::validate_wasm(&module.path, &entries[&module.path])?;
        }
        let hashes = entries
            .iter()
            .map(|(name, bytes)| (name.clone(), archive::hash(bytes)))
            .collect();
        entries.insert("sjml.toml".into(), self.manifest.with_hashes(hashes)?);
        archive::validate_limits(&entries)?;
        Ok(entries)
    }
}

pub fn init(path: &Path, id: Option<&str>, sdk: Option<&Path>) -> Result<()> {
    if path
        .try_exists()
        .map_err(|e| io("check initialization path", e))?
    {
        return Err(Error::new(
            "E_INIT",
            format!(
                "{} already exists; choose a new project directory",
                path.display()
            ),
        ));
    }
    let name = id
        .or_else(|| path.file_name().and_then(|n| n.to_str()))
        .ok_or_else(|| Error::new("E_INIT", "pass --id with a lowercase package identifier"))?;
    if !identifier(name) {
        return Err(Error::new(
            "E_INIT",
            "--id must be 1–64 lowercase ASCII letters, digits, '_' or '-', starting with a letter",
        ));
    }
    let dependency = if let Some(sdk) = sdk {
        let sdk = sdk
            .canonicalize()
            .map_err(|e| io("resolve --sdk-path", e))?;
        if !sdk.join("Cargo.toml").is_file() {
            return Err(Error::new(
                "E_INIT",
                "--sdk-path must point to sdk/sjml containing Cargo.toml",
            ));
        }
        let source = archive::bounded_read(&sdk.join("Cargo.toml"), MANIFEST_LIMIT)?;
        let source = std::str::from_utf8(&source)
            .map_err(|_| Error::new("E_INIT", "SDK Cargo.toml is not UTF-8"))?;
        let cargo: toml::Value = toml::from_str(source)
            .map_err(|e| Error::new("E_INIT", format!("invalid SDK Cargo.toml: {e}")))?;
        if cargo
            .get("package")
            .and_then(|p| p.get("name"))
            .and_then(|n| n.as_str())
            != Some("sjml-sdk")
        {
            return Err(Error::new(
                "E_INIT",
                "--sdk-path must point to the sjml-sdk package (sdk/sjml)",
            ));
        }
        format!(
            "sjml-sdk = {{ path = {} }}",
            toml::Value::String(sdk.to_string_lossy().into_owned())
        )
    } else {
        "sjml-sdk = \"0.40.0\"".into()
    };
    let parent = path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    fs::create_dir_all(parent).map_err(|e| io("create project parent", e))?;
    let temp = tempfile::tempdir_in(parent).map_err(|e| io("create temporary project", e))?;
    let files = [
        ("Cargo.toml",format!("[package]\nname = \"{name}\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[lib]\ncrate-type = [\"cdylib\", \"rlib\"]\n\n[dependencies]\n{dependency}\n\n[workspace]\n")),
        ("sjml.toml",format!("[mod]\nid = \"{name}\"\nname = \"{name}\"\nversion = \"0.1.0\"\n\n[runtime]\nabi = \"^1.32.0\"\nminecraft = \"~1.21.1\"\nsjml = \"^0.42.0\"\n\n[[modules]]\nid = \"main\"\npath = \"modules/main.wasm\"\nside = \"server\"\npermissions = []\n")),
        ("sjml-project.toml",format!("[[crates]]\nmodule = \"main\"\npackage = \"{name}\"\n")),
        ("src/lib.rs","use sjml::prelude::*;\n\n#[mod_init]\nfn initialize() -> SjmlResult<()> {\n    Ok(())\n}\n\n#[cfg(test)]\nmod tests {\n    #[test]\n    fn initialization_succeeds() {\n        assert!(super::initialize().is_ok());\n    }\n}\n".into()),
        (".gitignore","/target/\n/modules/\n/dist/\n".into()),
    ];
    for (relative, contents) in files {
        let destination = temp.path().join(relative);
        fs::create_dir_all(destination.parent().expect("generated parent"))
            .map_err(|e| io("create generated directory", e))?;
        fs::write(destination, contents).map_err(|e| io("write generated file", e))?;
    }
    // The destination was checked above; refuse a concurrent directory creation.
    fs::create_dir(path)
        .map_err(|e| Error::new("E_INIT", format!("cannot reserve {}: {e}", path.display())))?;
    for child in fs::read_dir(temp.path()).map_err(|e| io("read generated project", e))? {
        let child = child.map_err(|e| io("read generated entry", e))?;
        fs::rename(child.path(), path.join(child.file_name()))
            .map_err(|e| io("publish generated project", e))?;
    }
    println!("initialized {} at {}", name, path.display());
    Ok(())
}
