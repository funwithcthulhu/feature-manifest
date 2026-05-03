mod common;

use std::fs;

use common::{copy_fixture_to_temp, fixture_path, normalize, run_command};

#[test]
fn check_command_succeeds_for_basic_fixture() {
    let manifest_path = fixture_path("basic");
    let output = run_command(&[
        "check",
        "--manifest-path",
        manifest_path
            .to_str()
            .expect("fixture path should be UTF-8"),
    ]);

    assert!(
        output.status.success(),
        "stderr:\n{}",
        normalize(&output.stderr)
    );
    assert_eq!(
        normalize(&output.stdout),
        "validated 8 feature(s) and 1 group(s): 0 error(s), 0 warning(s)\n"
    );
}

#[test]
fn workspace_root_requires_explicit_package_selection() {
    let manifest_path = fixture_path("workspace");
    let output = run_command(&[
        "check",
        "--manifest-path",
        manifest_path
            .to_str()
            .expect("fixture path should be UTF-8"),
    ]);

    assert!(
        !output.status.success(),
        "stdout:\n{}",
        normalize(&output.stdout)
    );
    assert!(normalize(&output.stderr).contains("use `--workspace` or `--package <name>`"));
}

#[test]
fn workspace_check_reports_all_selected_packages() {
    let manifest_path = fixture_path("workspace");
    let output = run_command(&[
        "--workspace",
        "check",
        "--manifest-path",
        manifest_path
            .to_str()
            .expect("fixture path should be UTF-8"),
    ]);

    assert!(
        output.status.success(),
        "stderr:\n{}",
        normalize(&output.stderr)
    );
    let stdout = normalize(&output.stdout);
    assert!(stdout.contains("package `workspace-cli-fixture`"));
    assert!(stdout.contains("package `workspace-core-fixture`"));
    assert!(normalize(&output.stderr).contains("workspace summary: validated 2 package(s), 7 feature(s), 1 group(s): 0 error(s), 0 warning(s)"));
}

#[test]
fn explain_reports_feature_details() {
    let manifest_path = fixture_path("basic");
    let output = run_command(&[
        "explain",
        "docs-preview",
        "--manifest-path",
        manifest_path
            .to_str()
            .expect("fixture path should be UTF-8"),
    ]);

    assert!(
        output.status.success(),
        "stderr:\n{}",
        normalize(&output.stderr)
    );
    let stdout = normalize(&output.stdout);
    assert!(stdout.contains("Feature: `docs-preview`"));
    assert!(stdout.contains("Package: feature-manifest-fixture"));
    assert!(stdout.contains("Enables: `serde`, `tokio?/rt`"));
    assert!(stdout.contains("Included in default feature set: no"));
}

#[test]
fn sync_scaffolds_missing_metadata_entries() {
    let temp_dir = copy_fixture_to_temp("basic");
    let manifest_path = temp_dir.path().join("Cargo.toml");

    let manifest = fs::read_to_string(&manifest_path).expect("failed to read temp manifest");
    let updated_manifest = manifest.replace(
        "unstable = { description = \"Experimental APIs; semver not guaranteed.\", unstable = true }\n",
        "",
    );
    fs::write(&manifest_path, updated_manifest).expect("failed to write temp manifest");

    let output = run_command(&[
        "sync",
        "--manifest-path",
        manifest_path.to_str().expect("temp path should be UTF-8"),
    ]);

    assert!(
        output.status.success(),
        "stderr:\n{}",
        normalize(&output.stderr)
    );
    let stdout = normalize(&output.stdout);
    assert!(stdout.contains("synced `feature-manifest-fixture`"));
    assert!(stdout.contains("unstable"));

    let rewritten_manifest =
        fs::read_to_string(&manifest_path).expect("failed to read rewritten manifest");
    assert!(
        rewritten_manifest.contains("unstable = { description = \"TODO: describe `unstable`.\" }")
    );
}
