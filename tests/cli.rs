use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
    process::{Command, Output},
};
use tempfile::TempDir;
use zip::{ZipArchive, ZipWriter, write::SimpleFileOptions};

fn sandbox() -> TempDir {
    tempfile::tempdir().unwrap()
}
fn run(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_sjml-cli"))
        .args(args)
        .output()
        .unwrap()
}
fn run_from(directory: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_sjml-cli"))
        .current_dir(directory)
        .args(args)
        .output()
        .unwrap()
}
fn ok(output: Output) -> String {
    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).unwrap()
}
fn fails(output: Output, code: &str) {
    assert!(
        !output.status.success(),
        "unexpected success: {}",
        String::from_utf8_lossy(&output.stdout)
    );
    assert!(
        String::from_utf8_lossy(&output.stderr).contains(code),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}
fn manifest() -> &'static str {
    "[mod]\nid = 'demo'\nname = 'Demo'\nversion = '1.0.0'\n[runtime]\nabi = '^1.0'\n[[modules]]\nid = 'main'\npath = 'modules/main.wasm'\nside = 'server'\n"
}
fn raw_project(root: &Path) {
    fs::create_dir_all(root.join("modules")).unwrap();
    fs::write(root.join("sjml.toml"), manifest()).unwrap();
    fs::write(root.join("modules/main.wasm"), b"\0asm\x01\0\0\0").unwrap();
}
fn sdk_fixture(root: &Path) -> PathBuf {
    let sdk = root.join("sjml-sdk");
    let macros = root.join("sjml-macros");
    fs::create_dir_all(sdk.join("src")).unwrap();
    fs::create_dir_all(macros.join("src")).unwrap();
    fs::write(
        sdk.join("Cargo.toml"),
        "[package]\nname = \"sjml-sdk\"\nversion = \"0.40.0\"\nedition = \"2021\"\n\n[lib]\nname = \"sjml\"\n\n[dependencies]\nsjml-macros = { path = \"../sjml-macros\" }\n",
    )
    .unwrap();
    fs::write(
        sdk.join("src/lib.rs"),
        "pub use sjml_macros::mod_init;\npub type SjmlResult<T> = Result<T, String>;\npub mod prelude { pub use crate::{mod_init, SjmlResult}; }\n",
    )
    .unwrap();
    fs::write(
        macros.join("Cargo.toml"),
        "[package]\nname = \"sjml-macros\"\nversion = \"0.0.0\"\nedition = \"2021\"\n\n[lib]\nproc-macro = true\n",
    )
    .unwrap();
    fs::write(
        macros.join("src/lib.rs"),
        "use proc_macro::TokenStream;\n#[proc_macro_attribute]\npub fn mod_init(_: TokenStream, item: TokenStream) -> TokenStream { item }\n",
    )
    .unwrap();
    sdk
}
fn archive(path: &Path, entries: &[(&str, &[u8])]) {
    let mut zip = ZipWriter::new(fs::File::create(path).unwrap());
    for (name, bytes) in entries {
        zip.start_file(
            *name,
            SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored),
        )
        .unwrap();
        zip.write_all(bytes).unwrap();
    }
    zip.finish().unwrap();
}

#[test]
fn generated_project_builds_tests_packages_and_inspects_offline() {
    let tmp = sandbox();
    let project = tmp.path().join("hello");
    let sdk = sdk_fixture(tmp.path());
    ok(run(&[
        "init",
        project.to_str().unwrap(),
        "--id",
        "hello",
        "--sdk-path",
        sdk.to_str().unwrap(),
    ]));
    assert!(project.join("src/lib.rs").is_file());
    assert!(
        fs::read_to_string(project.join("sjml.toml"))
            .unwrap()
            .contains("sjml = \"^0.42.0\"")
    );
    // Re-initialization must never overwrite user files.
    fs::write(project.join("keep.txt"), "owned").unwrap();
    fails(run(&["init", project.to_str().unwrap()]), "E_INIT");
    assert_eq!(
        fs::read_to_string(project.join("keep.txt")).unwrap(),
        "owned"
    );
    let p = project.to_str().unwrap();
    ok(run(&["check", "--project", p, "--offline"]));
    ok(run(&["test", "--project", p, "--offline"]));
    ok(run(&["build", "--project", p, "--offline"]));
    let output = tmp.path().join("hello.sjml");
    ok(run(&[
        "package",
        "--project",
        p,
        "--offline",
        "--output",
        output.to_str().unwrap(),
    ]));
    let description = ok(run(&["inspect", output.to_str().unwrap()]));
    assert!(description.contains("hello 0.1.0"));
    assert!(description.contains("modules/main.wasm"));
    let dev_output = tmp.path().join("dev.sjml");
    ok(run(&[
        "dev",
        "--once",
        "--project",
        p,
        "--offline",
        "--output",
        dev_output.to_str().unwrap(),
    ]));
    assert!(dev_output.is_file());
}

#[test]
fn init_accepts_a_name_or_relative_destination_path() {
    let tmp = sandbox();
    ok(run_from(tmp.path(), &["init", "home/mymod"]));

    let project = tmp.path().join("home/mymod");
    assert!(project.join("Cargo.toml").is_file());
    let cargo_manifest = fs::read_to_string(project.join("Cargo.toml")).unwrap();
    assert!(cargo_manifest.contains("name = \"mymod\""));
    let package_manifest = fs::read_to_string(project.join("sjml.toml")).unwrap();
    assert!(package_manifest.contains("id = \"mymod\""));
}

#[test]
fn generated_default_template_tracks_current_sdk_host_and_abi_versions() {
    let tmp = sandbox();
    let project = tmp.path().join("versioned-template");
    ok(run(&[
        "init",
        project.to_str().unwrap(),
        "--id",
        "versioned_template",
    ]));

    let cargo_manifest = fs::read_to_string(project.join("Cargo.toml")).unwrap();
    assert!(cargo_manifest.contains("sjml-sdk = \"0.40.0\""));

    let package_manifest = fs::read_to_string(project.join("sjml.toml")).unwrap();
    assert!(package_manifest.contains("abi = \"^1.32.0\""));
    assert!(package_manifest.contains("sjml = \"^0.42.0\""));
}

#[test]
fn packages_are_reproducible_sorted_and_checksum_every_payload() {
    let tmp = sandbox();
    raw_project(tmp.path());
    fs::create_dir_all(tmp.path().join("assets/demo")).unwrap();
    fs::write(tmp.path().join("assets/demo/z.txt"), "hello").unwrap();
    fs::write(tmp.path().join("assets/demo/a.txt"), "world").unwrap();
    let p = tmp.path().to_str().unwrap();
    let first = tmp.path().join("first.sjml");
    let second = tmp.path().join("second.sjml");
    for target in [&first, &second] {
        ok(run(&[
            "package",
            "--no-build",
            "--project",
            p,
            "--output",
            target.to_str().unwrap(),
        ]));
    }
    assert_eq!(fs::read(&first).unwrap(), fs::read(&second).unwrap());
    let mut zip = ZipArchive::new(fs::File::open(&first).unwrap()).unwrap();
    let names: Vec<_> = (0..zip.len())
        .map(|i| zip.by_index(i).unwrap().name().to_owned())
        .collect();
    assert_eq!(
        names,
        [
            "assets/demo/a.txt",
            "assets/demo/z.txt",
            "modules/main.wasm",
            "sjml.toml"
        ]
    );
    let mut manifest_text = String::new();
    std::io::Read::read_to_string(&mut zip.by_name("sjml.toml").unwrap(), &mut manifest_text)
        .unwrap();
    let parsed: toml::Value = toml::from_str(&manifest_text).unwrap();
    let hashes = parsed["files"].as_array().unwrap();
    assert_eq!(hashes.len(), 3);
    assert_eq!(
        hashes[1]["sha256"].as_str().unwrap(),
        "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824"
    );
    ok(run(&["inspect", first.to_str().unwrap()]));
}

#[test]
fn inspect_rejects_traversal_bad_checksums_and_missing_modules() {
    let tmp = sandbox();
    let file = tmp.path().join("invalid.sjml");
    archive(
        &file,
        &[("../escape", b"bad"), ("sjml.toml", manifest().as_bytes())],
    );
    fails(run(&["inspect", file.to_str().unwrap()]), "E_ARCHIVE");
    assert!(!tmp.path().parent().unwrap().join("escape").exists());
    archive(&file, &[("sjml.toml", manifest().as_bytes())]);
    fails(run(&["inspect", file.to_str().unwrap()]), "E_ARCHIVE");
    let hashed = format!(
        "{}\n[[files]]\npath = 'modules/main.wasm'\nsha256 = '{}'\n",
        manifest(),
        "0".repeat(64)
    );
    archive(
        &file,
        &[
            ("sjml.toml", hashed.as_bytes()),
            ("modules/main.wasm", b"\0asm\x01\0\0\0"),
        ],
    );
    fails(run(&["inspect", file.to_str().unwrap()]), "E_CHECKSUM");
}

#[test]
fn invalid_manifests_and_non_wasm_modules_fail_before_output() {
    let tmp = sandbox();
    raw_project(tmp.path());
    let target = tmp.path().join("result.sjml");
    let args = [
        "package",
        "--no-build",
        "--project",
        tmp.path().to_str().unwrap(),
        "--output",
        target.to_str().unwrap(),
    ];
    let invalid = [
        manifest().replace("id = 'demo'", "id = 'Demo'"),
        manifest().replace("abi = '^1.0'", "abi = '>=1.0'"),
        manifest().replace("side = 'server'", "side = 'unknown'"),
        manifest().replace("path = 'modules/main.wasm'", "path = '../main.wasm'"),
        format!(
            "{}\n[[modules]]\nid = 'main'\npath = 'modules/other.wasm'\n",
            manifest()
        ),
        manifest().replace("[runtime]", "typo = true\n[runtime]"),
    ];
    for source in invalid {
        fs::write(tmp.path().join("sjml.toml"), source).unwrap();
        fails(run(&args), "E_MANIFEST");
        assert!(!target.exists());
    }
    fs::write(tmp.path().join("sjml.toml"), manifest()).unwrap();
    fs::write(tmp.path().join("modules/main.wasm"), b"not wasm").unwrap();
    fails(run(&args), "E_WASM");
    assert!(!target.exists());
}

#[test]
fn multiple_crates_build_into_their_declared_module_paths() {
    let tmp = sandbox();
    raw_project(tmp.path());
    fs::write(
        tmp.path().join("sjml.toml"),
        format!(
            "{}\n[[modules]]\nid = 'other'\npath = 'modules/other.wasm'\n",
            manifest()
        ),
    )
    .unwrap();
    fs::write(
        tmp.path().join("Cargo.toml"),
        "[workspace]\nmembers = ['first', 'second']\nresolver = '2'\n",
    )
    .unwrap();
    fs::write(tmp.path().join("sjml-project.toml"), "[[crates]]\nmodule = 'main'\npackage = 'first'\n[[crates]]\nmodule = 'other'\npackage = 'second'\n").unwrap();
    for name in ["first", "second"] {
        fs::create_dir_all(tmp.path().join(name).join("src")).unwrap();
        fs::write(tmp.path().join(name).join("Cargo.toml"), format!("[package]\nname = '{name}'\nversion = '0.1.0'\nedition = '2021'\n[lib]\ncrate-type = ['cdylib']\n")).unwrap();
        fs::write(
            tmp.path().join(name).join("src/lib.rs"),
            "#[no_mangle]\npub extern \"C\" fn value() -> i32 { 42 }\n",
        )
        .unwrap();
    }
    let output = tmp.path().join("multi.sjml");
    ok(run(&[
        "package",
        "--project",
        tmp.path().to_str().unwrap(),
        "--offline",
        "--output",
        output.to_str().unwrap(),
    ]));
    let inspected = ok(run(&["inspect", output.to_str().unwrap()]));
    assert!(inspected.contains("modules/main.wasm"));
    assert!(inspected.contains("modules/other.wasm"));
    let metadata = Command::new("cargo")
        .current_dir(tmp.path())
        .args([
            "metadata",
            "--no-deps",
            "--format-version",
            "1",
            "--offline",
        ])
        .output()
        .unwrap();
    assert!(metadata.status.success());
    let metadata: serde_json::Value = serde_json::from_slice(&metadata.stdout).unwrap();
    let artifact = PathBuf::from(metadata["target_directory"].as_str().unwrap())
        .join("wasm32-unknown-unknown/debug/first.wasm");
    let modified = fs::metadata(&artifact).unwrap().modified().unwrap();
    ok(run(&[
        "build",
        "--project",
        tmp.path().to_str().unwrap(),
        "--offline",
    ]));
    assert_eq!(
        fs::metadata(artifact).unwrap().modified().unwrap(),
        modified,
        "unchanged Cargo artifacts must retain their incremental cache"
    );
}

#[cfg(unix)]
#[test]
fn package_rejects_symlink_sources_without_overwriting_an_output() {
    let tmp = sandbox();
    raw_project(tmp.path());
    fs::create_dir(tmp.path().join("assets")).unwrap();
    std::os::unix::fs::symlink(tmp.path().join("sjml.toml"), tmp.path().join("assets/link"))
        .unwrap();
    let output = tmp.path().join("preserved.sjml");
    fs::write(&output, b"original").unwrap();
    fails(
        run(&[
            "package",
            "--no-build",
            "--project",
            tmp.path().to_str().unwrap(),
            "--output",
            output.to_str().unwrap(),
        ]),
        "E_PATH",
    );
    assert_eq!(fs::read(output).unwrap(), b"original");
}

#[test]
fn inspect_checks_crc_directory_consistency_and_limits_before_reading() {
    let tmp = sandbox();
    let file = tmp.path().join("invalid.sjml");
    archive(
        &file,
        &[
            ("sjml.toml", manifest().as_bytes()),
            ("modules/main.wasm", b"\0asm\x01\0\0\0"),
        ],
    );
    let clean = fs::read(&file).unwrap();
    let mut corrupt = clean.clone();
    let payload = corrupt
        .windows(8)
        .position(|w| w == b"\0asm\x01\0\0\0")
        .unwrap();
    corrupt[payload + 4] = 2;
    fs::write(&file, &corrupt).unwrap();
    fails(run(&["inspect", file.to_str().unwrap()]), "E_ARCHIVE");
    let central = clean.windows(4).position(|w| w == b"PK\x01\x02").unwrap();
    let mut corrupt = clean.clone();
    corrupt[central + 24..central + 28].copy_from_slice(&(65u32 * 1024 * 1024).to_le_bytes());
    fs::write(&file, &corrupt).unwrap();
    fails(run(&["inspect", file.to_str().unwrap()]), "E_ARCHIVE");
    let mut corrupt = clean.clone();
    let end = corrupt.len() - 22;
    corrupt[end + 8..end + 10].copy_from_slice(&5000u16.to_le_bytes());
    corrupt[end + 10..end + 12].copy_from_slice(&5000u16.to_le_bytes());
    fs::write(&file, &corrupt).unwrap();
    fails(run(&["inspect", file.to_str().unwrap()]), "E_LIMIT");
    let oversized = fs::File::create(&file).unwrap();
    oversized.set_len(256 * 1024 * 1024 + 1).unwrap();
    fails(run(&["inspect", file.to_str().unwrap()]), "E_LIMIT");
    // Regular files are checked before opening: pipes/devices must not hang inspect.
    fails(run(&["inspect", tmp.path().to_str().unwrap()]), "E_PATH");
}

#[test]
fn package_rejects_oversized_inputs_and_non_package_output_names() {
    let tmp = sandbox();
    raw_project(tmp.path());
    let bad_output = tmp.path().join("Cargo.toml");
    fs::write(&bad_output, b"preserved").unwrap();
    fails(
        run(&[
            "package",
            "--no-build",
            "--project",
            tmp.path().to_str().unwrap(),
            "--output",
            bad_output.to_str().unwrap(),
        ]),
        "E_PATH",
    );
    assert_eq!(fs::read(&bad_output).unwrap(), b"preserved");
    let bad_output = tmp.path().join("readme.txt");
    fails(
        run(&[
            "package",
            "--no-build",
            "--project",
            tmp.path().to_str().unwrap(),
            "--output",
            bad_output.to_str().unwrap(),
        ]),
        "E_PATH",
    );
    fs::create_dir(tmp.path().join("assets")).unwrap();
    fs::File::create(tmp.path().join("assets/huge.bin"))
        .unwrap()
        .set_len(64 * 1024 * 1024 + 1)
        .unwrap();
    let output = tmp.path().join("oversized.sjml");
    fails(
        run(&[
            "package",
            "--no-build",
            "--project",
            tmp.path().to_str().unwrap(),
            "--output",
            output.to_str().unwrap(),
        ]),
        "E_LIMIT",
    );
    assert!(!output.exists());
}

#[cfg(unix)]
#[test]
fn package_resolves_output_parent_aliases_before_protecting_asset_inputs() {
    let tmp = sandbox();
    let project = tmp.path().join("project");
    raw_project(&project);
    fs::create_dir(project.join("assets")).unwrap();
    let alias = tmp.path().join("alias");
    std::os::unix::fs::symlink(project.join("assets"), &alias).unwrap();
    let output = alias.join("result.sjml");
    fails(
        run(&[
            "package",
            "--no-build",
            "--project",
            project.to_str().unwrap(),
            "--output",
            output.to_str().unwrap(),
        ]),
        "E_PATH",
    );
    assert!(!output.exists());
}

#[test]
fn init_rejects_an_incorrect_sdk_package_before_creating_the_project() {
    let tmp = sandbox();
    let wrong_sdk = tmp.path().join("wrong-sdk");
    fs::create_dir_all(&wrong_sdk).unwrap();
    fs::write(
        wrong_sdk.join("Cargo.toml"),
        "[package]\nname = \"sjml-macros\"\nversion = \"0.1.0\"\n",
    )
    .unwrap();
    let output = tmp.path().join("bad_sdk");
    fails(
        run(&[
            "init",
            output.to_str().unwrap(),
            "--sdk-path",
            wrong_sdk.to_str().unwrap(),
        ]),
        "E_INIT",
    );
    assert!(!output.exists());
}

#[test]
fn inspect_uses_utf8_archive_paths_even_without_the_optional_utf8_flag() {
    let tmp = sandbox();
    let file = tmp.path().join("unicode.sjml");
    archive(
        &file,
        &[
            ("sjml.toml", manifest().as_bytes()),
            ("modules/main.wasm", b"\0asm\x01\0\0\0"),
            ("assets/café.txt", b"hello"),
        ],
    );
    let mut bytes = fs::read(&file).unwrap();
    for i in 0..bytes.len().saturating_sub(46) {
        let offset = if &bytes[i..i + 4] == b"PK\x03\x04" {
            Some(i + 6)
        } else if &bytes[i..i + 4] == b"PK\x01\x02" {
            Some(i + 8)
        } else {
            None
        };
        if let Some(offset) = offset {
            bytes[offset + 1] &= !8;
        }
    }
    fs::write(&file, bytes).unwrap();
    let inspected = ok(run(&["inspect", file.to_str().unwrap()]));
    assert!(inspected.contains("assets/café.txt"), "{inspected}");
}

#[test]
fn init_errors_do_not_create_partial_projects_or_panic() {
    let tmp = sandbox();
    for id in ["bad.name", "Bad", "a/b", ""] {
        let path = tmp.path().join("invalid");
        fails(run(&["init", path.to_str().unwrap(), "--id", id]), "E_INIT");
        assert!(!path.exists());
    }
    fails(run(&["nonsense"]), "unrecognized subcommand");
}

#[test]
fn package_bounds_the_entire_source_walk_including_empty_directories() {
    let tmp = sandbox();
    raw_project(tmp.path());
    for tree in ["assets", "data"] {
        fs::create_dir(tmp.path().join(tree)).unwrap();
        for i in 0..2050 {
            fs::create_dir(tmp.path().join(tree).join(format!("d{i}"))).unwrap();
        }
    }
    let output = tmp.path().join("too_many.sjml");
    fails(
        run(&[
            "package",
            "--no-build",
            "--project",
            tmp.path().to_str().unwrap(),
            "--output",
            output.to_str().unwrap(),
        ]),
        "E_LIMIT",
    );
    assert!(!output.exists());
}

#[test]
fn test_release_runs_native_tests_with_the_requested_optimization_profile() {
    let tmp = sandbox();
    raw_project(tmp.path());
    fs::create_dir(tmp.path().join("src")).unwrap();
    fs::write(
        tmp.path().join("Cargo.toml"),
        "[package]\nname = 'native_check'\nversion = '0.1.0'\nedition = '2021'\n[workspace]\n",
    )
    .unwrap();
    fs::write(
        tmp.path().join("src/lib.rs"),
        "#[cfg(test)]\nmod tests { #[test] fn release() { assert!(!cfg!(debug_assertions)); } }\n",
    )
    .unwrap();
    ok(run(&[
        "test",
        "--project",
        tmp.path().to_str().unwrap(),
        "--offline",
        "--release",
    ]));
}

#[test]
fn network_channel_declarations_round_trip_and_validate_the_host_contract() {
    let tmp = sandbox();
    raw_project(tmp.path());
    let channel = "\n[[modules.channels]]\nname = 'chat.message'\nschema_version = 4294967295\ndirections = ['client_to_server', 'server_to_client']\n";
    fs::write(
        tmp.path().join("sjml.toml"),
        format!("{}{channel}", manifest()),
    )
    .unwrap();
    let output = tmp.path().join("channels.sjml");
    let args = [
        "package",
        "--no-build",
        "--project",
        tmp.path().to_str().unwrap(),
        "--output",
        output.to_str().unwrap(),
    ];
    ok(run(&args));
    ok(run(&["inspect", output.to_str().unwrap()]));
    let original = fs::read(&output).unwrap();
    let invalid = [
        channel.replace("chat.message", "Chat.message"),
        channel.replace("4294967295", "0"),
        channel.replace("4294967295", "4294967296"),
        channel.replace("['client_to_server', 'server_to_client']", "[]"),
        channel.replace("'server_to_client'", "'client_to_server'"),
        channel.replace("'server_to_client'", "'both'"),
        channel.replace("directions =", "direction ="),
        format!("{channel}{channel}"),
    ];
    for declaration in invalid {
        fs::write(
            tmp.path().join("sjml.toml"),
            format!("{}{declaration}", manifest()),
        )
        .unwrap();
        fails(run(&args), "E_MANIFEST");
        assert_eq!(fs::read(&output).unwrap(), original);
    }
}

#[test]
fn network_channel_quota_is_global_across_modules() {
    let tmp = sandbox();
    raw_project(tmp.path());
    let mut text = manifest().to_owned();
    for i in 0..600 {
        text += &format!(
            "\n[[modules.channels]]\nname = 'channel_{i}'\nschema_version = 1\ndirections = ['client_to_server']\n"
        );
    }
    text += "\n[[modules]]\nid = 'other'\npath = 'modules/main.wasm'\n";
    for i in 0..424 {
        text += &format!(
            "\n[[modules.channels]]\nname = 'channel_{i}'\nschema_version = 1\ndirections = ['server_to_client']\n"
        );
    }
    fs::write(tmp.path().join("sjml.toml"), &text).unwrap();
    let output = tmp.path().join("too_many_channels.sjml");
    ok(run(&[
        "package",
        "--no-build",
        "--project",
        tmp.path().to_str().unwrap(),
        "--output",
        output.to_str().unwrap(),
    ]));
    let accepted = fs::read(&output).unwrap();
    text += "\n[[modules.channels]]\nname = 'one_too_many'\nschema_version = 1\ndirections = ['server_to_client']\n";
    fs::write(tmp.path().join("sjml.toml"), text).unwrap();
    fails(
        run(&[
            "package",
            "--no-build",
            "--project",
            tmp.path().to_str().unwrap(),
            "--output",
            output.to_str().unwrap(),
        ]),
        "E_MANIFEST",
    );
    assert_eq!(fs::read(output).unwrap(), accepted);
}

#[cfg(unix)]
#[test]
fn package_does_not_lexically_erase_a_symlink_before_resolving_parent_traversal() {
    let tmp = sandbox();
    let project = tmp.path().join("project");
    raw_project(&project);
    fs::create_dir_all(project.join("assets/nested")).unwrap();
    let alias = tmp.path().join("alias");
    std::os::unix::fs::symlink(project.join("assets/nested"), &alias).unwrap();
    let output = alias.join("../result.sjml");
    fails(
        run(&[
            "package",
            "--no-build",
            "--project",
            project.to_str().unwrap(),
            "--output",
            output.to_str().unwrap(),
        ]),
        "E_PATH",
    );
    assert!(!project.join("assets/result.sjml").exists());
}

#[test]
fn dev_watches_changes_preserves_the_last_good_package_and_recovers() {
    use std::{
        io::{BufRead, BufReader},
        process::{Child, Stdio},
        sync::mpsc,
        time::Duration,
    };
    struct Running(Child);
    impl Drop for Running {
        fn drop(&mut self) {
            let _ = self.0.kill();
            let _ = self.0.wait();
        }
    }
    let tmp = sandbox();
    raw_project(tmp.path());
    fs::create_dir_all(tmp.path().join("assets/target")).unwrap();
    let asset = tmp.path().join("assets/target/value.txt");
    fs::write(&asset, "first").unwrap();
    let output = tmp.path().join("watched.sjml");
    let mut running = Running(
        Command::new(env!("CARGO_BIN_EXE_sjml-cli"))
            .args([
                "dev",
                "--no-build",
                "--project",
                tmp.path().to_str().unwrap(),
                "--output",
                output.to_str().unwrap(),
                "--interval-ms",
                "50",
            ])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap(),
    );
    let (tx, rx) = mpsc::channel();
    let out = running.0.stdout.take().unwrap();
    let err = running.0.stderr.take().unwrap();
    let tx_out = tx.clone();
    std::thread::spawn(move || {
        for line in BufReader::new(out).lines() {
            if tx_out.send(line.unwrap()).is_err() {
                break;
            }
        }
    });
    std::thread::spawn(move || {
        for line in BufReader::new(err).lines() {
            if tx.send(line.unwrap()).is_err() {
                break;
            }
        }
    });
    let wait = |needle: &str| loop {
        let line = rx
            .recv_timeout(Duration::from_secs(10))
            .expect("watcher did not make progress");
        if line.contains(needle) {
            break;
        }
    };
    wait("watching");
    let first = fs::read(&output).unwrap();
    fs::write(&asset, "second version").unwrap();
    wait("packaged");
    let second = fs::read(&output).unwrap();
    assert_ne!(first, second);
    fs::File::create(&asset)
        .unwrap()
        .set_len(64 * 1024 * 1024 + 1)
        .unwrap();
    wait("E_LIMIT");
    assert_eq!(fs::read(&output).unwrap(), second);
    fs::write(&asset, "recovered").unwrap();
    wait("packaged");
    assert_ne!(fs::read(&output).unwrap(), second);
    ok(run(&["inspect", output.to_str().unwrap()]));
    // With --no-build, WASM files are inputs and must participate in watching.
    let recovered = fs::read(&output).unwrap();
    fs::write(tmp.path().join("modules/main.wasm"), b"invalid wasm").unwrap();
    wait("E_WASM");
    assert_eq!(fs::read(&output).unwrap(), recovered);
}

#[test]
fn implicit_workspace_mapping_rejects_ambiguous_normalized_package_names() {
    let tmp = sandbox();
    raw_project(tmp.path());
    fs::write(
        tmp.path().join("sjml.toml"),
        manifest().replace("id = 'main'", "id = 'a_b'"),
    )
    .unwrap();
    fs::write(
        tmp.path().join("Cargo.toml"),
        "[workspace]\nmembers = ['left', 'right']\nresolver = '2'\n",
    )
    .unwrap();
    for (directory, name) in [("left", "a-b"), ("right", "a_b")] {
        fs::create_dir_all(tmp.path().join(directory).join("src")).unwrap();
        fs::write(tmp.path().join(directory).join("Cargo.toml"),format!("[package]\nname = '{name}'\nversion = '0.1.0'\nedition = '2021'\n[lib]\nname = '{directory}'\ncrate-type = ['cdylib']\n")).unwrap();
        fs::write(tmp.path().join(directory).join("src/lib.rs"), "").unwrap();
    }
    fails(
        run(&[
            "check",
            "--project",
            tmp.path().to_str().unwrap(),
            "--offline",
        ]),
        "E_PROJECT",
    );
}

#[test]
fn inspect_measures_deflate_streams_instead_of_accepting_padded_size_claims() {
    let tmp = sandbox();
    let path = tmp.path().join("padded.sjml");
    let make = |payload: &[u8]| {
        let mut zip = ZipWriter::new(fs::File::create(&path).unwrap());
        for (name, bytes) in [
            ("sjml.toml", manifest().as_bytes()),
            ("modules/main.wasm", b"\0asm\x01\0\0\0"),
            ("assets/value.bin", payload),
        ] {
            zip.start_file(
                name,
                SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated),
            )
            .unwrap();
            zip.write_all(bytes).unwrap();
        }
        zip.finish().unwrap();
    };
    make(b"hello");
    ok(run(&["inspect", path.to_str().unwrap()]));
    make(&vec![0; 65536]);
    let mut bytes = fs::read(&path).unwrap();
    let central = bytes
        .windows(4)
        .enumerate()
        .filter(|(_, w)| *w == b"PK\x01\x02")
        .map(|(i, _)| i)
        .next_back()
        .unwrap();
    let u16_at = |p: usize| u16::from_le_bytes(bytes[p..p + 2].try_into().unwrap()) as usize;
    let u32_at = |p: usize| u32::from_le_bytes(bytes[p..p + 4].try_into().unwrap()) as usize;
    let local = u32_at(central + 42);
    assert_eq!(u16_at(local + 6) & 8, 0, "fixture uses known local sizes");
    let compressed = u32_at(central + 20);
    let data_end = local + 30 + u16_at(local + 26) + u16_at(local + 28) + compressed;
    let directory_offset = u32_at(bytes.len() - 22 + 16);
    // Inflate the declared compressed size with padding while retaining a valid
    // CRC and central/local metadata. The real stream still expands >200:1.
    bytes.splice(data_end..data_end, vec![0; 1000]);
    bytes[local + 18..local + 22].copy_from_slice(&((compressed + 1000) as u32).to_le_bytes());
    bytes[central + 1000 + 20..central + 1000 + 24]
        .copy_from_slice(&((compressed + 1000) as u32).to_le_bytes());
    let end = bytes.len() - 22;
    bytes[end + 16..end + 20].copy_from_slice(&((directory_offset + 1000) as u32).to_le_bytes());
    fs::write(&path, bytes).unwrap();
    fails(run(&["inspect", path.to_str().unwrap()]), "E_ARCHIVE");
}
