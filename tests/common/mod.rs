#![allow(dead_code)]

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use tempfile::TempDir;

pub fn fixture_path(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures")
        .join(name)
}

pub fn snapshot_path(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("snapshots")
        .join(name)
}

pub fn run_command(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_cargo-feature-manifest"))
        .args(args)
        .output()
        .expect("failed to run cargo-feature-manifest")
}

pub fn run_short_command(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_cargo-fm"))
        .args(args)
        .output()
        .expect("failed to run cargo-fm")
}

pub fn run_short_command_in(cwd: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_cargo-fm"))
        .current_dir(cwd)
        .args(args)
        .output()
        .expect("failed to run cargo-fm")
}

pub fn normalize(text: &[u8]) -> String {
    String::from_utf8_lossy(text).replace("\r\n", "\n")
}

pub fn read_snapshot(name: &str) -> String {
    fs::read_to_string(snapshot_path(name))
        .expect("failed to read snapshot file")
        .replace("\r\n", "\n")
}

pub fn copy_fixture_to_temp(name: &str) -> TempDir {
    let temp_dir = tempfile::tempdir().expect("failed to create temp dir");
    copy_dir_recursive(&fixture_path(name), temp_dir.path());
    temp_dir
}

pub fn copy_dir_recursive(source: &Path, destination: &Path) {
    fs::create_dir_all(destination).expect("failed to create destination directory");

    for entry in fs::read_dir(source).expect("failed to read fixture directory") {
        let entry = entry.expect("failed to read fixture entry");
        let file_type = entry.file_type().expect("failed to read file type");
        let destination_path = destination.join(entry.file_name());

        if file_type.is_dir() {
            copy_dir_recursive(&entry.path(), &destination_path);
        } else {
            fs::create_dir_all(
                destination_path
                    .parent()
                    .expect("destination file should have a parent"),
            )
            .expect("failed to create nested directory");
            copy_fixture_file(&entry.path(), &destination_path);
        }
    }
}

fn copy_fixture_file(source: &Path, destination: &Path) {
    if is_text_fixture(source) {
        let contents = fs::read_to_string(source).expect("failed to read text fixture file");
        fs::write(destination, contents.replace("\r\n", "\n"))
            .expect("failed to copy normalized text fixture file");
    } else {
        fs::copy(source, destination).expect("failed to copy fixture file");
    }
}

fn is_text_fixture(path: &Path) -> bool {
    matches!(
        path.extension().and_then(|extension| extension.to_str()),
        Some("toml" | "rs" | "md" | "txt" | "json" | "mmd" | "yml" | "yaml")
    )
}
