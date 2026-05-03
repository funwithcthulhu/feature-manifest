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
fn check_json_format_reports_issues() {
    let temp_dir = copy_fixture_to_temp("basic");
    let manifest_path = temp_dir.path().join("Cargo.toml");
    let manifest = fs::read_to_string(&manifest_path).expect("failed to read temp manifest");
    let updated_manifest = manifest.replace(
        "std = { description = \"Enables the standard library surface.\" }\n",
        "",
    );
    fs::write(&manifest_path, updated_manifest).expect("failed to write temp manifest");

    let output = run_command(&[
        "check",
        "--format",
        "json",
        "--manifest-path",
        manifest_path.to_str().expect("temp path should be UTF-8"),
    ]);

    assert!(
        !output.status.success(),
        "stdout:\n{}",
        normalize(&output.stdout)
    );
    let stdout = normalize(&output.stdout);
    assert!(stdout.contains("\"schema_version\": 1"));
    assert!(stdout.contains("\"code\": \"missing-metadata\""));
}

#[test]
fn check_formats_emit_github_and_sarif() {
    let temp_dir = copy_fixture_to_temp("basic");
    let manifest_path = temp_dir.path().join("Cargo.toml");
    let manifest = fs::read_to_string(&manifest_path).expect("failed to read temp manifest");
    let updated_manifest = manifest.replace(
        "docs-preview = { description = \"Generates docs | preview output.\\nIncludes async examples.\", note = \"Escapes table cells and Mermaid labels.\" }\n",
        "",
    );
    fs::write(&manifest_path, updated_manifest).expect("failed to write temp manifest");

    let github_output = run_command(&[
        "check",
        "--format",
        "github",
        "--manifest-path",
        manifest_path.to_str().expect("temp path should be UTF-8"),
    ]);
    assert!(!github_output.status.success());
    assert!(
        normalize(&github_output.stdout)
            .contains("::error file=Cargo.toml,title=feature-manifest missing-metadata::")
    );

    let sarif_output = run_command(&[
        "check",
        "--format",
        "sarif",
        "--manifest-path",
        manifest_path.to_str().expect("temp path should be UTF-8"),
    ]);
    assert!(!sarif_output.status.success());
    assert!(normalize(&sarif_output.stdout).contains("\"version\": \"2.1.0\""));
    assert!(normalize(&sarif_output.stdout).contains("\"ruleId\": \"missing-metadata\""));
}

#[test]
fn check_lint_overrides_can_change_exit_behavior() {
    let temp_dir = copy_fixture_to_temp("basic");
    let manifest_path = temp_dir.path().join("Cargo.toml");
    let manifest = fs::read_to_string(&manifest_path).expect("failed to read temp manifest");
    let updated_manifest =
        manifest.replace("rustls = { description = \"Use rustls for TLS.\" }\n", "");
    fs::write(&manifest_path, updated_manifest).expect("failed to write temp manifest");

    let failing = run_command(&[
        "check",
        "--manifest-path",
        manifest_path.to_str().expect("temp path should be UTF-8"),
    ]);
    assert!(!failing.status.success());

    let passing = run_command(&[
        "check",
        "--lint",
        "missing-metadata=allow",
        "--lint",
        "missing-description=allow",
        "--manifest-path",
        manifest_path.to_str().expect("temp path should be UTF-8"),
    ]);
    assert!(
        passing.status.success(),
        "stderr:\n{}",
        normalize(&passing.stderr)
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

#[test]
fn sync_check_remove_stale_and_style_flags_work() {
    let temp_dir = copy_fixture_to_temp("basic");
    let manifest_path = temp_dir.path().join("Cargo.toml");

    let manifest = fs::read_to_string(&manifest_path).expect("failed to read temp manifest");
    let stale_manifest = manifest
        .replace(
            "[package.metadata.feature-manifest.features]\n",
            "[package.metadata.feature-manifest]\nlegacy = { description = \"legacy\" }\n\n[package.metadata.feature-manifest.features]\n",
        )
        .replace(
            "docs-preview = { description = \"Generates docs | preview output.\\nIncludes async examples.\", note = \"Escapes table cells and Mermaid labels.\" }\n",
            "",
        );
    fs::write(&manifest_path, stale_manifest).expect("failed to write temp manifest");

    let check_output = run_command(&[
        "sync",
        "--check",
        "--remove-stale",
        "--style",
        "structured",
        "--manifest-path",
        manifest_path.to_str().expect("temp path should be UTF-8"),
    ]);
    assert!(!check_output.status.success());
    assert!(normalize(&check_output.stdout).contains("sync drift"));

    let apply_output = run_command(&[
        "sync",
        "--remove-stale",
        "--style",
        "structured",
        "--manifest-path",
        manifest_path.to_str().expect("temp path should be UTF-8"),
    ]);
    assert!(
        apply_output.status.success(),
        "stderr:\n{}",
        normalize(&apply_output.stderr)
    );

    let rewritten = fs::read_to_string(&manifest_path).expect("failed to read rewritten manifest");
    assert!(rewritten.contains("[package.metadata.feature-manifest.features]"));
    assert!(!rewritten.contains("legacy = { description = \"legacy\" }"));
    assert!(
        rewritten.contains("docs-preview = { description = \"TODO: describe `docs-preview`.\" }")
    );
}

#[test]
fn markdown_can_write_and_inject_into_docs() {
    let temp_dir = copy_fixture_to_temp("basic");
    let manifest_path = temp_dir.path().join("Cargo.toml");
    let output_path = temp_dir.path().join("FEATURES.md");
    let readme_path = temp_dir.path().join("README.md");

    fs::write(
        &readme_path,
        "# Fixture\n\n<!-- feature-manifest:start -->\nold\n<!-- feature-manifest:end -->\n",
    )
    .expect("failed to write readme fixture");

    let output = run_command(&[
        "markdown",
        "--write",
        output_path.to_str().expect("output path should be UTF-8"),
        "--insert-into",
        readme_path.to_str().expect("readme path should be UTF-8"),
        "--manifest-path",
        manifest_path
            .to_str()
            .expect("manifest path should be UTF-8"),
    ]);

    assert!(
        output.status.success(),
        "stderr:\n{}",
        normalize(&output.stderr)
    );
    let written = fs::read_to_string(&output_path).expect("failed to read FEATURES.md");
    let injected = fs::read_to_string(&readme_path).expect("failed to read injected README");
    assert!(written.contains("# feature-manifest-fixture feature manifest"));
    assert!(injected.contains("<!-- feature-manifest:start -->"));
    assert!(injected.contains("Default feature set: `serde`"));
}
