use std::env;
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};
use zip::ZipArchive;

struct TempWorkspace {
    path: PathBuf,
}

impl TempWorkspace {
    fn new(prefix: &str) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time")
            .as_nanos();
        let path = env::temp_dir().join(format!("{prefix}-{}-{now}", std::process::id()));
        fs::create_dir_all(&path).unwrap();
        Self { path }
    }

    fn as_path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TempWorkspace {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

fn write_file(base: &Path, rel_path: &str, content: &str) {
    let path = base.join(rel_path);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    let mut file = File::create(path).unwrap();
    file.write_all(content.as_bytes()).unwrap();
}

fn run_cli_in_dir(args: &[&str], cwd: &Path) -> std::process::Output {
    let binary = env!("CARGO_BIN_EXE_rustzen-zipper");
    Command::new(binary)
        .args(args)
        .current_dir(cwd)
        .output()
        .expect("run zipper binary")
}

fn run_cli(args: &[&str]) -> std::process::Output {
    let cwd = env::current_dir().unwrap();
    run_cli_in_dir(args, &cwd)
}

fn archive_names(path: &Path) -> Vec<String> {
    let mut archive = ZipArchive::new(File::open(path).unwrap()).unwrap();
    (0..archive.len())
        .map(|idx| archive.by_index(idx).unwrap().name().to_string())
        .collect()
}

#[test]
fn default_pack_keeps_root_prefix_and_has_expected_entries() {
    let workspace = TempWorkspace::new("rustzen_zipper_default");
    let source = workspace.as_path().join("project");
    write_file(&source, "a.txt", "hello");
    write_file(&source, "sub/b.txt", "world");

    let output_dir = workspace.as_path().join("out");
    let zip_path = output_dir.join("pkg-fixed.zip");

    let result = run_cli(&[
        "-s",
        source.to_str().unwrap(),
        "-o",
        "pkg",
        "-f",
        "fixed",
        "-d",
        output_dir.to_str().unwrap(),
        "-q",
    ]);

    assert!(result.status.success());
    let names = archive_names(&zip_path);
    assert!(names.contains(&"project/a.txt".to_string()));
    assert!(names.contains(&"project/sub/b.txt".to_string()));
    assert!(names.contains(&"project/sub/".to_string()));
}

#[test]
fn default_pack_short_flags_works() {
    let workspace = TempWorkspace::new("rustzen_zipper_subcmd");
    let source = workspace.as_path().join("project");
    write_file(&source, "keep.txt", "keep");

    let output_dir = workspace.as_path().join("out");
    let zip_path = output_dir.join("pkg-fixed.zip");

    let result = run_cli_in_dir(
        &[
            "-s",
            source.to_str().unwrap(),
            "-o",
            "pkg",
            "-f",
            "fixed",
            "--no-prefix",
            "-d",
            output_dir.to_str().unwrap(),
        ],
        workspace.as_path(),
    );

    assert!(result.status.success());
    assert!(zip_path.exists());

    let names = archive_names(&zip_path);
    assert!(names.contains(&"keep.txt".to_string()));
    assert!(!names.contains(&"project/keep.txt".to_string()));
}

#[test]
fn default_pack_keeps_root_prefix() {
    let workspace = TempWorkspace::new("rustzen_zipper_nested");
    let source = workspace.as_path().join("project");
    write_file(&source, "keep.txt", "keep");

    let output_dir = workspace.as_path().join("out");
    let zip_path = output_dir.join("pkg-fixed.zip");

    let result = run_cli_in_dir(
        &[
            "-s",
            source.to_str().unwrap(),
            "-o",
            "pkg",
            "-f",
            "fixed",
            "-q",
            "-d",
            output_dir.to_str().unwrap(),
        ],
        workspace.as_path(),
    );

    assert!(result.status.success());
    assert!(zip_path.exists());

    let names = archive_names(&zip_path);
    assert!(names.contains(&"project/keep.txt".to_string()));
}

#[test]
fn excludes_glob_pattern_ignores_matching_files() {
    let workspace = TempWorkspace::new("rustzen_zipper_exclude");
    let source = workspace.as_path().join("project");
    write_file(&source, "keep.txt", "keep");
    write_file(&source, "skip.log", "skip");

    let output_dir = workspace.as_path().join("out");
    let zip_path = output_dir.join("pkg-fixed.zip");

    let result = run_cli(&[
        "-s",
        source.to_str().unwrap(),
        "-o",
        "pkg",
        "-f",
        "fixed",
        "--exclude",
        "*.log",
        "-d",
        output_dir.to_str().unwrap(),
        "-q",
    ]);

    assert!(result.status.success());
    let names = archive_names(&zip_path);
    assert!(names.iter().any(|name| name.ends_with("keep.txt")));
    assert!(!names.iter().any(|name| name.ends_with("skip.log")));
}

#[test]
fn excludes_alias_still_supported_for_backward_compatibility() {
    let workspace = TempWorkspace::new("rustzen_zipper_exclude_alias");
    let source = workspace.as_path().join("project");
    write_file(&source, "keep.txt", "keep");
    write_file(&source, "skip.log", "skip");

    let output_dir = workspace.as_path().join("out");
    let zip_path = output_dir.join("pkg-fixed.zip");

    let result = run_cli(&[
        "-s",
        source.to_str().unwrap(),
        "-o",
        "pkg",
        "-f",
        "fixed",
        "--excludes",
        "*.log",
        "-d",
        output_dir.to_str().unwrap(),
        "-q",
    ]);

    assert!(result.status.success());
    let names = archive_names(&zip_path);
    assert!(names.iter().any(|name| name.ends_with("keep.txt")));
    assert!(!names.iter().any(|name| name.ends_with("skip.log")));
}

#[test]
fn level_option_controls_compression_argument() {
    let workspace = TempWorkspace::new("rustzen_zipper_level");
    let source = workspace.as_path().join("project");
    write_file(&source, "hello.txt", "hello world");

    let output_dir = workspace.as_path().join("out");
    let zip_path = output_dir.join("pkg-fixed.zip");

    let result = run_cli(&[
        "-s",
        source.to_str().unwrap(),
        "-o",
        "pkg",
        "-f",
        "fixed",
        "--level",
        "9",
        "--no-prefix",
        "-d",
        output_dir.to_str().unwrap(),
        "-q",
    ]);

    assert!(result.status.success());
    assert!(zip_path.exists());
}

#[test]
fn quiet_mode_suppresses_standard_output() {
    let workspace = TempWorkspace::new("rustzen_zipper_quiet");
    let source = workspace.as_path().join("project");
    write_file(&source, "hello.txt", "hello");

    let output_dir = workspace.as_path().join("out");
    let result = run_cli(&[
        "-s",
        source.to_str().unwrap(),
        "-o",
        "pkg",
        "-f",
        "fixed",
        "--quiet",
        "-d",
        output_dir.to_str().unwrap(),
    ]);

    assert!(result.status.success());
    assert!(String::from_utf8_lossy(&result.stdout).trim().is_empty());
}

#[test]
fn includes_glob_pattern_only_keeps_matching_files() {
    let workspace = TempWorkspace::new("rustzen_zipper_include");
    let source = workspace.as_path().join("project");
    write_file(&source, "keep.txt", "keep");
    write_file(&source, "skip.log", "skip");

    let output_dir = workspace.as_path().join("out");
    let zip_path = output_dir.join("pkg-fixed.zip");

    let result = run_cli(&[
        "-s",
        source.to_str().unwrap(),
        "-o",
        "pkg",
        "-f",
        "fixed",
        "--include",
        "*keep*",
        "-d",
        output_dir.to_str().unwrap(),
        "-q",
    ]);

    assert!(result.status.success());
    let names = archive_names(&zip_path);
    assert!(names.iter().any(|name| name.ends_with("keep.txt")));
    assert!(!names.iter().any(|name| name.ends_with("skip.log")));
}

#[test]
fn base_dir_overwrites_zip_root_prefix() {
    let workspace = TempWorkspace::new("rustzen_zipper_base_dir");
    let source = workspace.as_path().join("project");
    write_file(&source, "keep.txt", "keep");

    let output_dir = workspace.as_path().join("out");
    let zip_path = output_dir.join("pkg-fixed.zip");

    let result = run_cli(&[
        "-s",
        source.to_str().unwrap(),
        "-o",
        "pkg",
        "-f",
        "fixed",
        "--base-dir",
        "artifact",
        "-d",
        output_dir.to_str().unwrap(),
        "-q",
    ]);

    assert!(result.status.success());
    let names = archive_names(&zip_path);
    assert!(names.contains(&"artifact/keep.txt".to_string()));
    assert!(!names.contains(&"project/keep.txt".to_string()));
}

#[test]
fn strip_prefix_removes_matching_entry_prefix() {
    let workspace = TempWorkspace::new("rustzen_zipper_strip_prefix");
    let source = workspace.as_path().join("project");
    write_file(&source, "bundle/keep.txt", "keep");

    let output_dir = workspace.as_path().join("out");
    let zip_path = output_dir.join("pkg-fixed.zip");

    let result = run_cli(&[
        "-s",
        source.to_str().unwrap(),
        "-o",
        "pkg",
        "-f",
        "fixed",
        "--strip-prefix",
        "bundle",
        "--no-prefix",
        "-d",
        output_dir.to_str().unwrap(),
        "-q",
    ]);

    assert!(result.status.success());
    let names = archive_names(&zip_path);
    assert!(names.contains(&"keep.txt".to_string()));
    assert!(!names.contains(&"bundle/keep.txt".to_string()));
}

#[test]
fn sha256_output_file_is_generated_when_enabled() {
    let workspace = TempWorkspace::new("rustzen_zipper_sha256");
    let source = workspace.as_path().join("project");
    write_file(&source, "hello.txt", "hello");

    let output_dir = workspace.as_path().join("out");
    let zip_path = output_dir.join("pkg-fixed.zip");
    let sha_path = output_dir.join("pkg-fixed.zip.sha256");

    let result = run_cli(&[
        "-s",
        source.to_str().unwrap(),
        "-o",
        "pkg",
        "-f",
        "fixed",
        "--sha256",
        "--no-prefix",
        "-d",
        output_dir.to_str().unwrap(),
    ]);

    assert!(result.status.success());
    assert!(zip_path.exists());
    assert!(sha_path.exists());

    let checksum = fs::read_to_string(sha_path).unwrap();
    let mut parts = checksum.split_whitespace();
    let hash = parts.next().unwrap_or("");
    let name = parts.next().unwrap_or("");
    assert_eq!(hash.len(), 64);
    assert!(hash.chars().all(|c| c.is_ascii_hexdigit()));
    assert_eq!(name, "pkg-fixed.zip");
}

#[test]
fn overwrite_skip_keeps_existing_output() {
    let workspace = TempWorkspace::new("rustzen_zipper_overwrite_skip");
    let source = workspace.as_path().join("project");
    write_file(&source, "hello.txt", "hello");

    let output_dir = workspace.as_path().join("out");
    let zip_path = output_dir.join("pkg-fixed.zip");

    let first = run_cli(&[
        "-s",
        source.to_str().unwrap(),
        "-o",
        "pkg",
        "-f",
        "fixed",
        "-d",
        output_dir.to_str().unwrap(),
    ]);
    assert!(first.status.success());
    assert!(zip_path.exists());

    let second = run_cli(&[
        "-s",
        source.to_str().unwrap(),
        "-o",
        "pkg",
        "-f",
        "fixed",
        "-d",
        output_dir.to_str().unwrap(),
        "--overwrite",
        "skip",
    ]);
    assert!(second.status.success());
    let stdout = String::from_utf8_lossy(&second.stdout);
    assert!(stdout.contains("Skipped: output already exists"));
}

#[test]
fn overwrite_error_rejects_existing_output() {
    let workspace = TempWorkspace::new("rustzen_zipper_overwrite_error");
    let source = workspace.as_path().join("project");
    write_file(&source, "hello.txt", "hello");

    let output_dir = workspace.as_path().join("out");
    let zip_path = output_dir.join("pkg-fixed.zip");

    let first = run_cli(&[
        "-s",
        source.to_str().unwrap(),
        "-o",
        "pkg",
        "-f",
        "fixed",
        "-d",
        output_dir.to_str().unwrap(),
    ]);
    assert!(first.status.success());
    assert!(zip_path.exists());

    let second = run_cli(&[
        "-s",
        source.to_str().unwrap(),
        "-o",
        "pkg",
        "-f",
        "fixed",
        "-d",
        output_dir.to_str().unwrap(),
        "--overwrite",
        "error",
    ]);
    assert!(!second.status.success());
}

#[test]
fn output_dir_is_created_and_used() {
    let workspace = TempWorkspace::new("rustzen_zipper_output_dir");
    let source = workspace.as_path().join("project");
    write_file(&source, "hello.txt", "hello");

    let output_dir = workspace.as_path().join("nested").join("artifacts");
    let zip_path = output_dir.join("pkg-fixed.zip");

    let result = run_cli(&[
        "-s",
        source.to_str().unwrap(),
        "-o",
        "pkg",
        "-f",
        "fixed",
        "-d",
        output_dir.to_str().unwrap(),
    ]);

    assert!(result.status.success());
    assert!(output_dir.exists());
    assert!(zip_path.exists());
}

#[test]
fn invalid_pattern_returns_error() {
    let workspace = TempWorkspace::new("rustzen_zipper_invalid_pattern");
    let source = workspace.as_path().join("project");
    write_file(&source, "hello.txt", "hello");

    let output_dir = workspace.as_path().join("out");
    let result = run_cli(&[
        "-s",
        source.to_str().unwrap(),
        "-o",
        "pkg",
        "-f",
        "fixed",
        "-i",
        "[",
        "-d",
        output_dir.to_str().unwrap(),
    ]);

    assert!(!result.status.success());
}

#[test]
fn invalid_level_parameter_rejected() {
    let workspace = TempWorkspace::new("rustzen_zipper_invalid_level");
    let source = workspace.as_path().join("project");
    write_file(&source, "hello.txt", "hello");

    let output_dir = workspace.as_path().join("out");
    let result = run_cli(&[
        "-s",
        source.to_str().unwrap(),
        "-o",
        "pkg",
        "-f",
        "fixed",
        "--level",
        "10",
        "-d",
        output_dir.to_str().unwrap(),
    ]);

    assert!(!result.status.success());
}

#[test]
fn config_file_not_found_fails() {
    let workspace = TempWorkspace::new("rustzen_zipper_config_not_found");
    let source = workspace.as_path().join("project");
    write_file(&source, "hello.txt", "hello");

    let output_dir = workspace.as_path().join("out");
    let result = run_cli_in_dir(
        &[
            "--config",
            "does-not-exist.json",
            "-s",
            source.to_str().unwrap(),
            "-o",
            "pkg",
            "-f",
            "fixed",
            "-d",
            output_dir.to_str().unwrap(),
            "-q",
        ],
        workspace.as_path(),
    );

    assert!(!result.status.success());
    assert!(String::from_utf8_lossy(&result.stderr).contains("Config file does not exist"));
}

#[test]
fn config_file_invalid_values_rejected() {
    let workspace = TempWorkspace::new("rustzen_zipper_invalid_config");
    let source = workspace.as_path().join("project");
    write_file(&source, "hello.txt", "hello");

    let config = workspace.as_path().join(".rzrc");
    let mut file = File::create(&config).unwrap();
    file.write_all(br#"{"source":"project","compression":"invalid","level":"9"}"#)
        .unwrap();

    let result = run_cli_in_dir(
        &["--config", ".rzrc", "-o", "pkg", "-f", "fixed", "-q"],
        workspace.as_path(),
    );

    assert!(!result.status.success());
    assert!(String::from_utf8_lossy(&result.stderr).contains("Invalid compression value"));
}

#[test]
fn dry_run_does_not_create_zip_file() {
    let workspace = TempWorkspace::new("rustzen_zipper_dry_run");
    let source = workspace.as_path().join("project");
    write_file(&source, "hello.txt", "hello");

    let output_dir = workspace.as_path().join("out");
    let zip_path = output_dir.join("pkg-fixed.zip");
    let result = run_cli(&[
        "-s",
        source.to_str().unwrap(),
        "-o",
        "pkg",
        "-f",
        "fixed",
        "--dry-run",
        "-d",
        output_dir.to_str().unwrap(),
    ]);

    assert!(result.status.success());
    assert!(!zip_path.exists());
}

#[test]
fn unpack_command_extracts_created_archive() {
    let workspace = TempWorkspace::new("rustzen_zipper_unpack");
    let source = workspace.as_path().join("project");
    write_file(&source, "hello.txt", "hello world");

    let zip_output = workspace.as_path().join("in");
    let zip_path = zip_output.join("archive-fixed.zip");
    let unpack_output = workspace.as_path().join("out");

    let pack = run_cli_in_dir(
        &[
            "-s",
            source.to_str().unwrap(),
            "-d",
            zip_output.to_str().unwrap(),
            "-o",
            "archive",
            "-f",
            "fixed",
            "--no-prefix",
        ],
        workspace.as_path(),
    );
    assert!(pack.status.success());

    let unpack = run_cli_in_dir(
        &[
            "unpack",
            "--source",
            zip_path.to_str().unwrap(),
            "-o",
            unpack_output.to_str().unwrap(),
        ],
        workspace.as_path(),
    );
    assert!(unpack.status.success());

    assert!(unpack_output.join("hello.txt").exists());
    let unpacked = fs::read_to_string(unpack_output.join("hello.txt")).unwrap();
    assert_eq!(unpacked, "hello world");
}

#[test]
fn config_file_is_merged_into_pack_options() {
    let workspace = TempWorkspace::new("rustzen_zipper_config_merge");
    let source = workspace.as_path().join("project");
    write_file(&source, "keep.txt", "keep");
    write_file(&source, "skip.txt", "skip");

    let config = workspace.as_path().join(".rzrc");
    let mut file = File::create(&config).unwrap();
    file.write_all(
        br#"{
  "excludes": ["*skip.txt"],
  "output_dir": "configured",
  "no_prefix": true,
  "source": "project"
}"#,
    )
    .unwrap();

    let zip_path = workspace.as_path().join("configured").join("pkg-fixed.zip");

    let result = run_cli_in_dir(
        &["--config", ".rzrc", "-o", "pkg", "-f", "fixed", "-q"],
        workspace.as_path(),
    );

    assert!(result.status.success());
    let names = archive_names(&zip_path);
    assert!(names.contains(&"keep.txt".to_string()));
    assert!(!names.contains(&"skip.txt".to_string()));
}

#[test]
fn config_boolean_true_can_be_overridden_to_false_in_cli() {
    let workspace = TempWorkspace::new("rustzen_zipper_config_bool_override");
    let source = workspace.as_path().join("project");
    write_file(&source, "hello.txt", "hello");

    let config = workspace.as_path().join(".rzrc");
    let mut file = File::create(&config).unwrap();
    file.write_all(br#"{"source":"project","output_dir":"configured","quiet":true}"#)
        .unwrap();

    let zip_path = workspace.as_path().join("configured").join("pkg-fixed.zip");

    let result = run_cli_in_dir(
        &[
            "--config",
            ".rzrc",
            "--no-quiet",
            "-o",
            "pkg",
            "-f",
            "fixed",
        ],
        workspace.as_path(),
    );

    assert!(result.status.success());
    assert!(zip_path.exists());
    assert!(!String::from_utf8_lossy(&result.stdout).is_empty());
    assert!(String::from_utf8_lossy(&result.stderr).is_empty());
}

#[test]
fn config_no_prefix_can_be_overridden_to_prefix_in_cli() {
    let workspace = TempWorkspace::new("rustzen_zipper_config_prefix_override");
    let source = workspace.as_path().join("project");
    write_file(&source, "keep.txt", "keep");

    let config = workspace.as_path().join(".rzrc");
    let mut file = File::create(&config).unwrap();
    file.write_all(br#"{"source":"project","no_prefix":true,"output_dir":"configured"}"#)
        .unwrap();

    let zip_path = workspace.as_path().join("configured").join("pkg-fixed.zip");
    let result = run_cli_in_dir(
        &[
            "--config", ".rzrc", "--prefix", "-o", "pkg", "-f", "fixed", "-q",
        ],
        workspace.as_path(),
    );

    assert!(result.status.success());
    let names = archive_names(&zip_path);
    assert!(names.contains(&"project/keep.txt".to_string()));
    assert!(!names.contains(&"keep.txt".to_string()));
}

#[test]
fn config_sha256_can_be_disabled_in_cli() {
    let workspace = TempWorkspace::new("rustzen_zipper_config_sha256_override");
    let source = workspace.as_path().join("project");
    write_file(&source, "hello.txt", "hello");

    let config = workspace.as_path().join(".rzrc");
    let mut file = File::create(&config).unwrap();
    file.write_all(br#"{"source":"project","output_dir":"configured","sha256":true}"#)
        .unwrap();

    let zip_path = workspace.as_path().join("configured").join("pkg-fixed.zip");
    let sha_path = workspace
        .as_path()
        .join("configured")
        .join("pkg-fixed.zip.sha256");

    let result = run_cli_in_dir(
        &[
            "--config",
            ".rzrc",
            "--no-sha256",
            "-o",
            "pkg",
            "-f",
            "fixed",
            "-q",
        ],
        workspace.as_path(),
    );

    assert!(result.status.success());
    assert!(zip_path.exists());
    assert!(!sha_path.exists());
}

#[test]
fn config_dry_run_true_can_be_forced_to_pack_in_cli() {
    let workspace = TempWorkspace::new("rustzen_zipper_config_dry_run_override");
    let source = workspace.as_path().join("project");
    write_file(&source, "hello.txt", "hello");

    let config = workspace.as_path().join(".rzrc");
    let mut file = File::create(&config).unwrap();
    file.write_all(br#"{"source":"project","dry_run":true,"output_dir":"configured"}"#)
        .unwrap();

    let zip_path = workspace.as_path().join("configured").join("pkg-fixed.zip");
    let result = run_cli_in_dir(
        &[
            "--config",
            ".rzrc",
            "--no-dry-run",
            "-o",
            "pkg",
            "-f",
            "fixed",
            "-q",
        ],
        workspace.as_path(),
    );

    assert!(result.status.success());
    assert!(zip_path.exists());
}

#[test]
fn package_json_top_level_zipper_field_is_merged_into_pack_options() {
    let workspace = TempWorkspace::new("rustzen_zipper_package_json_config");
    let source = workspace.as_path().join("project");
    write_file(&source, "keep.txt", "keep");
    write_file(&source, "skip.log", "skip");

    let package_json = workspace.as_path().join("package.json");
    let mut file = File::create(&package_json).unwrap();
    file.write_all(
        br#"{
  "zipper": {
    "source": "project",
    "output_dir": "configured",
    "excludes": ["*skip.log"],
    "no_prefix": true
  }
}"#,
    )
    .unwrap();

    let zip_path = workspace.as_path().join("configured").join("pkg-fixed.zip");

    let result = run_cli_in_dir(&["-o", "pkg", "-f", "fixed", "-q"], workspace.as_path());

    assert!(result.status.success());
    let names = archive_names(&zip_path);
    assert!(names.contains(&"keep.txt".to_string()));
    assert!(!names.iter().any(|name| name.ends_with("skip.log")));
}
