mod archive;
mod error;
mod manifest;
mod project;

use clap::{Parser, Subcommand};
use error::{Error, Result, io};
use project::{Project, ProjectArgs};
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    time::{Duration, SystemTime},
};

#[derive(Parser)]
#[command(
    name = "sjml-cli",
    version,
    about = "Build and inspect SJML WebAssembly guest packages"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}
#[derive(Subcommand)]
enum Commands {
    /// Create a Rust guest project and initialize Git on master.
    Init {
        #[arg(
            value_name = "NAME_OR_PATH",
            help = "Project name or destination path relative to the current directory"
        )]
        name_or_path: PathBuf,
        /// Override the package ID inferred from the name or final path component.
        #[arg(long)]
        id: Option<String>,
        /// Use a local checkout of the sjml-sdk crate for development.
        #[arg(long)]
        sdk_path: Option<PathBuf>,
    },
    /// Compile all declared guest crates incrementally for wasm32-unknown-unknown.
    Build(ProjectArgs),
    /// Validate the manifest and type-check all guest crates for WebAssembly.
    Check(ProjectArgs),
    /// Run the workspace's native Rust tests.
    Test(ProjectArgs),
    /// Build a deterministic, checksummed .sjml archive.
    Package(PackageArgs),
    /// Rebuild and package changed project sources (Ctrl-C to stop).
    Dev {
        #[command(flatten)]
        package: PackageArgs,
        #[arg(long)]
        once: bool,
        #[arg(long,default_value_t=500,value_parser=clap::value_parser!(u64).range(50..=60000))]
        interval_ms: u64,
    },
    /// Validate an archive without extracting it, and list modules, hashes and files.
    Inspect { archive: PathBuf },
}
#[derive(Clone, clap::Args)]
struct PackageArgs {
    #[command(flatten)]
    project: ProjectArgs,
    /// Output archive (default: package-id-version.sjml in the project's dist directory).
    #[arg(long)]
    output: Option<PathBuf>,
    /// Package existing declared WASM files without invoking Cargo.
    #[arg(long)]
    no_build: bool,
}
fn package(args: &PackageArgs) -> Result<()> {
    let project = Project::load(&args.project.project)?;
    let output = args.output.clone().unwrap_or_else(|| {
        project.root.join("dist").join(format!(
            "{}-{}.sjml",
            project.manifest.id, project.manifest.version
        ))
    });
    if output
        .extension()
        .is_none_or(|extension| extension != "sjml")
    {
        return Err(Error::new("E_PATH", "--output must end with .sjml"));
    }
    // An archive must not overwrite a project input or reside in an asset tree.
    let absolute = if output.is_absolute() {
        output.clone()
    } else {
        std::env::current_dir()
            .map_err(|e| io("get current directory", e))?
            .join(&output)
    };
    // Resolve real filesystem parents before removing '..': a preceding symlink
    // changes which directory '..' names.
    let mut ancestor = absolute;
    let mut suffix = Vec::new();
    loop {
        match ancestor.canonicalize() {
            Ok(canonical) => {
                ancestor = canonical;
                break;
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                let name = ancestor
                    .file_name()
                    .ok_or_else(|| Error::new("E_PATH", "invalid --output path"))?
                    .to_owned();
                suffix.push(name);
                ancestor.pop();
            }
            Err(e) => return Err(io("resolve --output", e)),
        }
    }
    for part in suffix.into_iter().rev() {
        ancestor.push(part);
    }
    let normalized = ancestor;
    let reserved = ["sjml.toml", "sjml-project.toml", "Cargo.toml", "Cargo.lock"];
    if reserved.iter().any(|p| normalized == project.root.join(p))
        || project
            .manifest
            .modules
            .iter()
            .any(|m| normalized == project.root.join(&m.path))
        || ["assets", "data", "config", "src"]
            .iter()
            .any(|p| normalized.starts_with(project.root.join(p)))
    {
        return Err(Error::new(
            "E_PATH",
            "--output must not overwrite project inputs or be inside assets/data/config/src",
        ));
    }
    if !args.no_build {
        project.cargo(&args.project, "build")?;
    }
    let entries = project.entries()?;
    archive::write(&output, &entries)?;
    println!(
        "packaged {} {} -> {} ({} files)",
        project.manifest.id,
        project.manifest.version,
        output.display(),
        entries.len()
    );
    Ok(())
}
fn normalize(path: &Path) -> PathBuf {
    let mut result = PathBuf::new();
    for part in path.components() {
        match part {
            std::path::Component::CurDir => {}
            std::path::Component::ParentDir => {
                result.pop();
            }
            part => result.push(part.as_os_str()),
        }
    }
    result
}
type Snapshot = BTreeMap<PathBuf, (u64, SystemTime)>;
fn snapshot(root: &Path, output: Option<&Path>, modules: &[manifest::Module]) -> Result<Snapshot> {
    let mut stack = vec![root.to_owned()];
    let mut result = BTreeMap::new();
    let output = output.map(|p| {
        if p.is_absolute() {
            normalize(p)
        } else {
            normalize(
                &std::env::current_dir()
                    .unwrap_or_else(|_| root.to_owned())
                    .join(p),
            )
        }
    });
    let mut visited = 0usize;
    while let Some(directory) = stack.pop() {
        for entry in
            fs::read_dir(&directory).map_err(|e| io(format!("watch {}", directory.display()), e))?
        {
            visited += 1;
            if visited > 100_000 {
                return Err(Error::new(
                    "E_LIMIT",
                    "dev watch exceeds 100000 filesystem entries",
                ));
            }
            let entry = entry.map_err(|e| io("read watched entry", e))?;
            let path = entry.path();
            let kind = entry
                .file_type()
                .map_err(|e| io("read watched file type", e))?;
            let name = entry.file_name();
            if kind.is_symlink()
                || (directory == root
                    && ["target", ".git", "dist", ".sjml"]
                        .iter()
                        .any(|n| name == *n))
                || output.as_ref().is_some_and(|p| *p == path)
                || modules.iter().any(|m| root.join(&m.path) == path)
            {
                continue;
            }
            if kind.is_dir() {
                stack.push(path);
            } else if kind.is_file() {
                let meta = entry
                    .metadata()
                    .map_err(|e| io("read watched metadata", e))?;
                result.insert(
                    path,
                    (
                        meta.len(),
                        meta.modified()
                            .map_err(|e| io("read watched modification time", e))?,
                    ),
                );
            }
        }
    }
    Ok(result)
}
fn execute(cli: Cli) -> Result<()> {
    match cli.command {
        Commands::Init {
            name_or_path,
            id,
            sdk_path,
        } => project::init(&name_or_path, id.as_deref(), sdk_path.as_deref()),
        Commands::Build(args) => Project::load(&args.project)?.cargo(&args, "build"),
        Commands::Check(args) => Project::load(&args.project)?.cargo(&args, "check"),
        Commands::Test(args) => Project::load(&args.project)?.cargo(&args, "test"),
        Commands::Package(args) => package(&args),
        Commands::Inspect { archive: path } => {
            let (manifest, entries) = archive::read(&path)?;
            println!("{} {}", manifest.id, manifest.version);
            for module in &manifest.modules {
                println!("module {}: {}", module.id, module.path);
            }
            for (name, bytes) in &entries {
                println!("{}  {:>10}  {}", archive::hash(bytes), bytes.len(), name);
            }
            if manifest.document.get("signature").is_some() {
                println!(
                    "signature metadata present; publisher authenticity has not been verified"
                );
            }
            Ok(())
        }
        Commands::Dev {
            package: args,
            once,
            interval_ms,
        } => {
            if once {
                return package(&args);
            }
            let project = Project::load(&args.project.project)?;
            let mut ignored_modules = if args.no_build {
                Vec::new()
            } else {
                project.manifest.modules.clone()
            };
            let mut previous = snapshot(&project.root, args.output.as_deref(), &ignored_modules)?;
            package(&args)?;
            println!(
                "watching {} (Ctrl-C to stop; restart Minecraft to load an updated package)",
                project.root.display()
            );
            loop {
                std::thread::sleep(Duration::from_millis(interval_ms));
                // Manifest edits can introduce new compiler output paths. Refresh
                // them before scanning; a temporarily invalid edit is diagnosed
                // by package() and does not terminate the watcher.
                if !args.no_build
                    && let Ok(updated) = Project::load(&project.root)
                {
                    ignored_modules = updated.manifest.modules;
                }
                let current = snapshot(&project.root, args.output.as_deref(), &ignored_modules)?;
                if current != previous {
                    // Keep the inputs observed before this build. Edits made while
                    // building must trigger the next iteration, not become baseline.
                    previous = current;
                    if let Err(e) = package(&args) {
                        eprintln!("{e}");
                    }
                }
            }
        }
    }
}
fn main() {
    if let Err(error) = execute(Cli::parse()) {
        eprintln!("{error}");
        std::process::exit(1);
    }
}
