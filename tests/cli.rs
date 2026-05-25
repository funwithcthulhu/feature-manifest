mod common;

use std::fs;

use common::{
    copy_fixture_to_temp, fixture_path, normalize, run_command, run_short_command,
    run_short_command_in,
};

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
fn short_binary_and_aliases_work() {
    let manifest_path = fixture_path("basic");
    let check_output = run_short_command(&[
        "fm",
        "c",
        "-m",
        manifest_path
            .to_str()
            .expect("fixture path should be UTF-8"),
    ]);

    assert!(
        check_output.status.success(),
        "stderr:\n{}",
        normalize(&check_output.stderr)
    );
    assert_eq!(
        normalize(&check_output.stdout),
        "validated 8 feature(s) and 1 group(s): 0 error(s), 0 warning(s)\n"
    );

    let lints_output = run_short_command(&["lints"]);
    assert!(lints_output.status.success());
    assert!(normalize(&lints_output.stdout).contains("missing-metadata"));

    let lint_docs_output = run_short_command(&["lints", "--markdown"]);
    assert!(lint_docs_output.status.success());
    assert!(normalize(&lint_docs_output.stdout).contains("# Lint Reference"));
}

#[test]
fn cargo_subcommand_names_are_normalized_for_short_binary() {
    let manifest_path = fixture_path("basic");

    for subcommand_name in ["feature-manifest", "feature_manifest"] {
        let output = run_short_command(&[
            subcommand_name,
            "check",
            "-m",
            manifest_path
                .to_str()
                .expect("fixture path should be UTF-8"),
        ]);

        assert!(
            output.status.success(),
            "{subcommand_name} stderr:\n{}",
            normalize(&output.stderr)
        );
        assert_eq!(
            normalize(&output.stdout),
            "validated 8 feature(s) and 1 group(s): 0 error(s), 0 warning(s)\n"
        );
    }
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
        concat!(
            "docs-preview = { description = \"Generates docs | preview output.\\n",
            "Includes async examples.\", category = \"docs\", since = \"0.2.0\", ",
            "docs = \"https://docs.rs/feature-manifest\", tracking_issue = ",
            "\"https://github.com/funwithcthulhu/feature-manifest/issues/1\", ",
            "requires = [\"serde\"], note = \"Escapes table cells and Mermaid labels.\" }\n"
        ),
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
    assert!(normalize(&github_output.stdout).contains(
        "::error file=Cargo.toml,line=14,col=1,title=feature-manifest missing-metadata::"
    ));

    let sarif_output = run_command(&[
        "check",
        "--format",
        "sarif",
        "--manifest-path",
        manifest_path.to_str().expect("temp path should be UTF-8"),
    ]);
    assert!(!sarif_output.status.success());
    let sarif_stdout = normalize(&sarif_output.stdout);
    assert!(sarif_stdout.contains("\"version\": \"2.1.0\""));
    assert!(sarif_stdout.contains("\"ruleId\": \"missing-metadata\""));
    assert!(sarif_stdout.contains("\"startLine\": 14"));
    assert!(sarif_stdout.contains("\"startColumn\": 1"));
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
fn check_preset_can_soften_adoption_failures() {
    let temp_dir = copy_fixture_to_temp("basic");
    let manifest_path = temp_dir.path().join("Cargo.toml");
    let manifest = fs::read_to_string(&manifest_path).expect("failed to read temp manifest");
    let updated_manifest =
        manifest.replace("rustls = { description = \"Use rustls for TLS.\" }\n", "");
    fs::write(&manifest_path, updated_manifest).expect("failed to write temp manifest");

    let output = run_command(&[
        "check",
        "--preset",
        "adopt",
        "--manifest-path",
        manifest_path.to_str().expect("temp path should be UTF-8"),
    ]);

    assert!(
        output.status.success(),
        "stderr:\n{}",
        normalize(&output.stderr)
    );
    assert!(normalize(&output.stderr).contains("warning[missing-metadata]"));
}

#[test]
fn check_rejects_invalid_lint_overrides_actionably() {
    let manifest_path = fixture_path("basic");

    let unknown = run_command(&[
        "check",
        "--lint",
        "not-a-lint=warn",
        "--manifest-path",
        manifest_path
            .to_str()
            .expect("fixture path should be UTF-8"),
    ]);
    assert!(
        !unknown.status.success(),
        "stdout:\n{}",
        normalize(&unknown.stdout)
    );
    let stderr = normalize(&unknown.stderr);
    assert!(stderr.contains("invalid lint override `not-a-lint=warn`"));
    assert!(stderr.contains("unknown lint code `not-a-lint`"));
    assert!(stderr.contains("known codes:"));
    assert!(stderr.contains("missing-metadata"));

    let invalid_level = run_command(&[
        "check",
        "--lint",
        "missing-metadata=sometimes",
        "--manifest-path",
        manifest_path
            .to_str()
            .expect("fixture path should be UTF-8"),
    ]);
    assert!(
        !invalid_level.status.success(),
        "stdout:\n{}",
        normalize(&invalid_level.stdout)
    );
    let stderr = normalize(&invalid_level.stderr);
    assert!(stderr.contains("invalid lint override `missing-metadata=sometimes`"));
    assert!(stderr.contains("expected `allow`, `warn`, or `deny`, found `sometimes`"));
}

#[test]
fn check_reports_stale_and_missing_metadata_actionably() {
    let manifest_path = fixture_path("realistic");
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
    let stderr = normalize(&output.stderr);
    assert!(stderr.contains("error[missing-metadata] `undocumented`"));
    assert!(stderr.contains("feature is defined in `[features]` but missing metadata"));
    assert!(stderr.contains("add an entry under `[package.metadata.feature-manifest]`"));
    assert!(stderr.contains("error[unknown-metadata] `stale-feature`"));
    assert!(stderr.contains("metadata exists for a feature that is not declared in `[features]`"));
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
    assert!(normalize(&output.stderr).contains(concat!(
        "workspace summary: validated 2 package(s), 7 feature(s), 1 group(s): ",
        "0 error(s), 0 warning(s)"
    )));
}

#[test]
fn workspace_package_selection_reports_one_selected_package() {
    let manifest_path = fixture_path("workspace");
    let output = run_command(&[
        "--package",
        "workspace-core-fixture",
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
        "validated 3 feature(s) and 0 group(s): 0 error(s), 0 warning(s)\n"
    );
    assert_eq!(normalize(&output.stderr), "");
}

#[test]
fn realistic_workspace_selection_requires_and_honors_explicit_package() {
    let manifest_path = fixture_path("workspace-selection");

    let ambiguous = run_command(&[
        "check",
        "--manifest-path",
        manifest_path
            .to_str()
            .expect("fixture path should be UTF-8"),
    ]);
    assert!(
        !ambiguous.status.success(),
        "stdout:\n{}",
        normalize(&ambiguous.stdout)
    );
    let stderr = normalize(&ambiguous.stderr);
    assert!(
        stderr.contains("the selected manifest resolves to a workspace with multiple packages")
    );
    assert!(stderr.contains("use `--workspace` or `--package <name>`"));
    assert!(stderr.contains("workspace-selection-cli"));
    assert!(stderr.contains("workspace-selection-core"));

    let selected_check = run_command(&[
        "--package",
        "workspace-selection-cli",
        "check",
        "--manifest-path",
        manifest_path
            .to_str()
            .expect("fixture path should be UTF-8"),
    ]);
    assert!(
        selected_check.status.success(),
        "stderr:\n{}",
        normalize(&selected_check.stderr)
    );
    assert_eq!(
        normalize(&selected_check.stdout),
        "validated 4 feature(s) and 1 group(s): 0 error(s), 0 warning(s)\n"
    );

    let selected_markdown = run_command(&[
        "--package",
        "workspace-selection-cli",
        "markdown",
        "--manifest-path",
        manifest_path
            .to_str()
            .expect("fixture path should be UTF-8"),
    ]);
    assert!(
        selected_markdown.status.success(),
        "stderr:\n{}",
        normalize(&selected_markdown.stderr)
    );
    let stdout = normalize(&selected_markdown.stdout);
    assert!(stdout.contains("# workspace-selection-cli feature manifest"));
    assert!(!stdout.contains("workspace-selection-core feature manifest"));
    assert!(
        stdout.contains(
            "| `color` | yes | public | stable | output | — | Enable ANSI color output. |"
        )
    );
    assert!(stdout.contains("| `core` | no | public | stable | integration | `dep:workspace-selection-core`, `workspace-selection-core/serde` | Enable integration with the core package. |"));
    assert!(stdout.contains("_1 internal/private feature(s) hidden."));
    assert!(stdout.contains("- `output`: User-facing output formats. Members: `color`, `json`."));
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
            concat!(
                "[package.metadata.feature-manifest]\n",
                "legacy = { description = \"legacy\" }\n\n",
                "[package.metadata.feature-manifest.features]\n",
            ),
        )
        .replace(
            concat!(
                "docs-preview = { description = \"Generates docs | preview output.\\n",
                "Includes async examples.\", category = \"docs\", since = \"0.2.0\", ",
                "docs = \"https://docs.rs/feature-manifest\", tracking_issue = ",
                "\"https://github.com/funwithcthulhu/feature-manifest/issues/1\", ",
                "requires = [\"serde\"], note = \"Escapes table cells and Mermaid labels.\" }\n"
            ),
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
fn sync_diff_previews_rewrite_without_editing() {
    let temp_dir = copy_fixture_to_temp("basic");
    let manifest_path = temp_dir.path().join("Cargo.toml");

    let manifest = fs::read_to_string(&manifest_path).expect("failed to read temp manifest");
    let updated_manifest = manifest.replace(
        "unstable = { description = \"Experimental APIs; semver not guaranteed.\", unstable = true }\n",
        "",
    );
    fs::write(&manifest_path, updated_manifest.clone()).expect("failed to write temp manifest");

    let output = run_command(&[
        "sync",
        "--diff",
        "--manifest-path",
        manifest_path.to_str().expect("temp path should be UTF-8"),
    ]);

    assert!(!output.status.success());
    let stdout = normalize(&output.stdout);
    assert!(stdout.contains("sync drift in `feature-manifest-fixture`"));
    assert!(stdout.contains("--- a/"));
    assert!(stdout.contains("+unstable = { description = \"TODO: describe `unstable`.\" }"));

    let after = fs::read_to_string(&manifest_path).expect("failed to read temp manifest");
    assert_eq!(after, updated_manifest);
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

#[test]
fn markdown_insert_and_check_report_missing_marker_pair_actionably() {
    let temp_dir = copy_fixture_to_temp("basic");
    let manifest_path = temp_dir.path().join("Cargo.toml");
    let readme_path = temp_dir.path().join("README.md");

    fs::write(&readme_path, "# Fixture\n\nNo feature markers yet.\n")
        .expect("failed to write README without markers");

    for args in [
        vec![
            "md",
            "-i",
            readme_path.to_str().expect("README path should be UTF-8"),
            "-m",
            manifest_path
                .to_str()
                .expect("manifest path should be UTF-8"),
        ],
        vec![
            "md",
            "--check",
            "-i",
            readme_path.to_str().expect("README path should be UTF-8"),
            "-m",
            manifest_path
                .to_str()
                .expect("manifest path should be UTF-8"),
        ],
    ] {
        let output = run_command(&args);
        assert!(
            !output.status.success(),
            "args {args:?} stdout:\n{}",
            normalize(&output.stdout)
        );
        let stderr = normalize(&output.stderr);
        assert!(stderr.contains("feature-manifest marker pair was not found"));
        assert!(stderr.contains("README.md"));
        assert!(stderr.contains("<!-- feature-manifest:start -->"));
        assert!(stderr.contains("<!-- feature-manifest:end -->"));
        assert!(stderr.contains("cargo fm init --readme"));
    }
}

#[test]
fn markdown_check_detects_stale_files_and_injected_regions() {
    let temp_dir = copy_fixture_to_temp("basic");
    let manifest_path = temp_dir.path().join("Cargo.toml");
    let output_path = temp_dir.path().join("FEATURES.md");
    let readme_path = temp_dir.path().join("README.md");

    fs::write(&output_path, "stale\n").expect("failed to write stale FEATURES.md");
    fs::write(
        &readme_path,
        "# Fixture\n\n<!-- feature-manifest:start -->\nstale\n<!-- feature-manifest:end -->\n",
    )
    .expect("failed to write stale README");

    let stale = run_command(&[
        "md",
        "--check",
        "-o",
        output_path.to_str().expect("output path should be UTF-8"),
        "-i",
        readme_path.to_str().expect("readme path should be UTF-8"),
        "-m",
        manifest_path
            .to_str()
            .expect("manifest path should be UTF-8"),
    ]);
    assert!(!stale.status.success());
    assert!(normalize(&stale.stdout).contains("stale"));
    assert!(normalize(&stale.stderr).contains(
        "generated Markdown is stale; rerun the markdown command without `--check` to update it"
    ));

    let rewrite = run_command(&[
        "md",
        "-o",
        output_path.to_str().expect("output path should be UTF-8"),
        "-i",
        readme_path.to_str().expect("readme path should be UTF-8"),
        "-m",
        manifest_path
            .to_str()
            .expect("manifest path should be UTF-8"),
    ]);
    assert!(rewrite.status.success());

    let fresh = run_command(&[
        "md",
        "--check",
        "-o",
        output_path.to_str().expect("output path should be UTF-8"),
        "-i",
        readme_path.to_str().expect("readme path should be UTF-8"),
        "-m",
        manifest_path
            .to_str()
            .expect("manifest path should be UTF-8"),
    ]);
    assert!(
        fresh.status.success(),
        "stderr:\n{}",
        normalize(&fresh.stderr)
    );

    let generated_file =
        fs::read_to_string(&output_path).expect("failed to read generated FEATURES.md");
    fs::write(&output_path, generated_file.replace('\n', "\r\n"))
        .expect("failed to rewrite FEATURES.md with CRLF line endings");
    let generated_readme =
        fs::read_to_string(&readme_path).expect("failed to read generated README");
    fs::write(&readme_path, generated_readme.replace('\n', "\r\n"))
        .expect("failed to rewrite README with CRLF line endings");

    let crlf_fresh = run_command(&[
        "md",
        "--check",
        "-o",
        output_path.to_str().expect("output path should be UTF-8"),
        "-i",
        readme_path.to_str().expect("readme path should be UTF-8"),
        "-m",
        manifest_path
            .to_str()
            .expect("manifest path should be UTF-8"),
    ]);
    assert!(
        crlf_fresh.status.success(),
        "stderr:\n{}",
        normalize(&crlf_fresh.stderr)
    );
}

#[test]
fn markdown_check_reports_stale_injected_region_with_update_command() {
    let temp_dir = copy_fixture_to_temp("basic");
    let manifest_path = temp_dir.path().join("Cargo.toml");
    let readme_path = temp_dir.path().join("README.md");

    fs::write(
        &readme_path,
        "# Fixture\n\n<!-- feature-manifest:start -->\nstale\n<!-- feature-manifest:end -->\n",
    )
    .expect("failed to write stale README");

    let output = run_command(&[
        "md",
        "--check",
        "-i",
        readme_path.to_str().expect("README path should be UTF-8"),
        "-m",
        manifest_path
            .to_str()
            .expect("manifest path should be UTF-8"),
    ]);

    assert!(
        !output.status.success(),
        "stdout:\n{}",
        normalize(&output.stdout)
    );
    assert!(normalize(&output.stdout).contains("README.md` injected region is stale"));
    let stderr = normalize(&output.stderr);
    assert!(stderr.contains("generated Markdown is stale"));
    assert!(stderr.contains("cargo fm markdown --insert-into"));
    assert!(stderr.contains("README.md"));
}

#[test]
fn malformed_feature_manifest_metadata_reports_table_shape() {
    let temp_dir = tempfile::tempdir().expect("failed to create temp dir");
    let manifest_path = temp_dir.path().join("Cargo.toml");
    let src_dir = temp_dir.path().join("src");
    fs::create_dir(&src_dir).expect("failed to create src dir");
    fs::write(src_dir.join("lib.rs"), "pub fn fixture() {}\n").expect("failed to write lib.rs");
    fs::write(
        &manifest_path,
        r#"[package]
name = "malformed-metadata-fixture"
version = "0.1.0"
edition = "2024"

[features]
serde = []

[package.metadata]
feature-manifest = "not a table"
"#,
    )
    .expect("failed to write malformed manifest");

    let output = run_command(&[
        "check",
        "-m",
        manifest_path
            .to_str()
            .expect("manifest path should be UTF-8"),
    ]);

    assert!(
        !output.status.success(),
        "stdout:\n{}",
        normalize(&output.stdout)
    );
    let stderr = normalize(&output.stderr);
    assert!(stderr.contains("failed to parse feature metadata from"));
    assert!(stderr.contains("Cargo.toml"));
    assert!(stderr.contains("`[package.metadata.feature-manifest]` must be a TOML table"));
}

#[test]
fn markdown_injection_rejects_duplicate_markers() {
    let temp_dir = copy_fixture_to_temp("basic");
    let manifest_path = temp_dir.path().join("Cargo.toml");
    let readme_path = temp_dir.path().join("README.md");

    fs::write(
        &readme_path,
        "<!-- feature-manifest:start -->\nold\n<!-- feature-manifest:start -->\n<!-- feature-manifest:end -->\n",
    )
    .expect("failed to write duplicate marker README");

    let output = run_command(&[
        "md",
        "-i",
        readme_path.to_str().expect("readme path should be UTF-8"),
        "-m",
        manifest_path
            .to_str()
            .expect("manifest path should be UTF-8"),
    ]);

    assert!(!output.status.success());
    assert!(normalize(&output.stderr).contains("duplicate feature-manifest markers"));
}

#[test]
fn short_markdown_alias_supports_short_flags() {
    let temp_dir = copy_fixture_to_temp("basic");
    let manifest_path = temp_dir.path().join("Cargo.toml");
    let output_path = temp_dir.path().join("FEATURES.md");

    let output = run_short_command(&[
        "md",
        "-o",
        output_path.to_str().expect("output path should be UTF-8"),
        "-m",
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
    assert!(written.contains("# feature-manifest-fixture feature manifest"));
}

#[test]
fn init_scaffolds_metadata_readme_and_optional_ci() {
    let temp_dir = copy_fixture_to_temp("basic");
    let manifest_path = temp_dir.path().join("Cargo.toml");
    let readme_path = temp_dir.path().join("README.md");

    let manifest = fs::read_to_string(&manifest_path).expect("failed to read temp manifest");
    let stripped_manifest = manifest.replace(
        "unstable = { description = \"Experimental APIs; semver not guaranteed.\", unstable = true }\n",
        "",
    );
    fs::write(&manifest_path, stripped_manifest).expect("failed to write temp manifest");

    let output = run_command(&[
        "init",
        "--ci",
        "--readme",
        readme_path.to_str().expect("readme path should be UTF-8"),
        "-m",
        manifest_path
            .to_str()
            .expect("manifest path should be UTF-8"),
    ]);

    assert!(
        output.status.success(),
        "stderr:\n{}",
        normalize(&output.stderr)
    );

    let rewritten_manifest = fs::read_to_string(&manifest_path).expect("manifest should exist");
    let rewritten_readme = fs::read_to_string(&readme_path).expect("README should exist");
    let workflow = fs::read_to_string(
        temp_dir
            .path()
            .join(".github")
            .join("workflows")
            .join("feature-manifest.yml"),
    )
    .expect("workflow should exist");

    assert!(
        rewritten_manifest.contains("unstable = { description = \"TODO: describe `unstable`.\" }")
    );
    assert!(rewritten_readme.contains("<!-- feature-manifest:start -->"));
    assert!(rewritten_readme.contains("Default feature set: `serde`"));
    assert!(workflow.contains("cargo fm"));
    assert!(workflow.contains("actions/checkout@v6"));
}

#[test]
fn init_dry_run_reports_actions_without_writing() {
    let temp_dir = copy_fixture_to_temp("basic");
    let manifest_path = temp_dir.path().join("Cargo.toml");
    let readme_path = temp_dir.path().join("README.md");

    let manifest = fs::read_to_string(&manifest_path).expect("failed to read temp manifest");
    let stripped_manifest = manifest.replace(
        "unstable = { description = \"Experimental APIs; semver not guaranteed.\", unstable = true }\n",
        "",
    );
    fs::write(&manifest_path, stripped_manifest.clone()).expect("failed to write temp manifest");

    let output = run_command(&[
        "init",
        "--dry-run",
        "--ci",
        "--readme",
        readme_path.to_str().expect("readme path should be UTF-8"),
        "-m",
        manifest_path
            .to_str()
            .expect("manifest path should be UTF-8"),
    ]);

    assert!(
        output.status.success(),
        "stderr:\n{}",
        normalize(&output.stderr)
    );
    let stdout = normalize(&output.stdout);
    assert!(stdout.contains("would initialize metadata"));
    assert!(stdout.contains("would create README"));
    assert!(stdout.contains("would add CI workflow"));
    assert!(stdout.contains("dry run complete"));

    assert_eq!(
        fs::read_to_string(&manifest_path).expect("manifest should exist"),
        stripped_manifest
    );
    assert!(!readme_path.exists());
    assert!(
        !temp_dir
            .path()
            .join(".github")
            .join("workflows")
            .join("feature-manifest.yml")
            .exists()
    );
}

#[test]
fn doctor_reports_project_wiring() {
    let temp_dir = copy_fixture_to_temp("basic");
    let manifest_path = temp_dir.path().join("Cargo.toml");
    let readme_path = temp_dir.path().join("README.md");

    let init = run_command(&[
        "init",
        "--ci",
        "--readme",
        readme_path.to_str().expect("readme path should be UTF-8"),
        "-m",
        manifest_path
            .to_str()
            .expect("manifest path should be UTF-8"),
    ]);
    assert!(init.status.success());

    let output = run_command(&[
        "doctor",
        "--readme",
        readme_path.to_str().expect("readme path should be UTF-8"),
        "-m",
        manifest_path
            .to_str()
            .expect("manifest path should be UTF-8"),
    ]);

    assert!(
        output.status.success(),
        "stderr:\n{}",
        normalize(&output.stderr)
    );
    let stdout = normalize(&output.stdout);
    assert!(stdout.contains("feature metadata validates cleanly"));
    assert!(stdout.contains("README feature section is up to date"));
    assert!(stdout.contains("CI workflow references feature-manifest"));
}

#[test]
fn doctor_explain_prints_next_actions_for_findings() {
    let temp_dir = copy_fixture_to_temp("basic");
    let manifest_path = temp_dir.path().join("Cargo.toml");

    let output = run_command(&[
        "doctor",
        "--explain",
        "-m",
        manifest_path
            .to_str()
            .expect("manifest path should be UTF-8"),
    ]);

    assert!(
        output.status.success(),
        "stderr:\n{}",
        normalize(&output.stderr)
    );
    let stdout = normalize(&output.stdout);
    assert!(stdout.contains("warn: README markers were not found"));
    assert!(stdout.contains("next: run `cargo fm init --readme"));
}

#[test]
fn doctor_strict_fails_on_warnings() {
    let temp_dir = copy_fixture_to_temp("basic");
    let manifest_path = temp_dir.path().join("Cargo.toml");

    let output = run_command(&[
        "doctor",
        "--strict",
        "-m",
        manifest_path
            .to_str()
            .expect("manifest path should be UTF-8"),
    ]);

    assert!(!output.status.success());
    assert!(normalize(&output.stderr).contains("doctor found warnings in strict mode"));
}

#[test]
fn schema_and_completions_do_not_require_manifest() {
    let temp_dir = tempfile::tempdir().expect("failed to create temp dir");

    let schema = run_short_command_in(temp_dir.path(), &["schema", "metadata"]);
    assert!(
        schema.status.success(),
        "stderr:\n{}",
        normalize(&schema.stderr)
    );
    assert!(normalize(&schema.stdout).contains("\"title\": \"feature-manifest metadata report\""));

    let completions = run_short_command_in(temp_dir.path(), &["completions", "powershell"]);
    assert!(
        completions.status.success(),
        "stderr:\n{}",
        normalize(&completions.stderr)
    );
    assert!(normalize(&completions.stdout).contains("cargo-fm"));
}

#[test]
fn schema_write_failure_reports_output_path() {
    let temp_dir = tempfile::tempdir().expect("failed to create temp dir");
    let schema_path = temp_dir.path().join("missing-parent").join("metadata.json");

    let output = run_short_command(&[
        "schema",
        "metadata",
        "-o",
        schema_path
            .to_str()
            .expect("schema output path should be UTF-8"),
    ]);

    let stderr = normalize(&output.stderr);
    assert!(!output.status.success());
    assert!(stderr.contains("failed to write generated output"));
    assert!(stderr.contains(&schema_path.display().to_string()));
}

#[test]
fn edge_fixture_covers_mixed_layout_and_dependency_shapes() {
    let manifest_path = fixture_path("edge");
    let output = run_command(&[
        "check",
        "-m",
        manifest_path
            .to_str()
            .expect("fixture path should be UTF-8"),
    ]);

    assert!(
        output.status.success(),
        "stderr:\n{}",
        normalize(&output.stderr)
    );
}

#[test]
fn realistic_fixture_markdown_output_keeps_table_order_and_visibility() {
    let manifest_path = fixture_path("realistic");
    let output = run_command(&[
        "markdown",
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

    let expected = r#"# feature-manifest-realistic-fixture feature manifest

Default feature set: `serde`

| Feature | Default | Visibility | Status | Category | Enables | Description |
| --- | --- | --- | --- | --- | --- | --- |
| `public-api` | no | public | stable | api | — | Expose the public API helpers. |
| `runtime` | no | public | stable | runtime | `dep:tokio`, `tokio/rt` | Enable the Tokio runtime integration. |
| `serde` | yes | public | stable | serialization | `dep:serde` | Enable serialization support. |
| `undocumented` | no | public | stable | — | — | No description provided. |

_1 internal/private feature(s) hidden. Use `--include-private` to render all._

## Groups

- `api`: Public API and internal implementation toggles. Members: `public-api`, `internal-codegen`.
"#;

    assert_eq!(normalize(&output.stdout), expected);
}

#[test]
fn realistic_fixture_supports_small_readme_marker_check_cycle() {
    let temp_dir = copy_fixture_to_temp("realistic");
    let manifest_path = temp_dir.path().join("Cargo.toml");
    let readme_path = temp_dir.path().join("README.md");

    let stale_check = run_command(&[
        "md",
        "--check",
        "-i",
        readme_path.to_str().expect("README path should be UTF-8"),
        "-m",
        manifest_path
            .to_str()
            .expect("manifest path should be UTF-8"),
    ]);
    assert!(
        !stale_check.status.success(),
        "stdout:\n{}",
        normalize(&stale_check.stdout)
    );
    assert!(normalize(&stale_check.stdout).contains("stale"));

    let write_output = run_command(&[
        "md",
        "-i",
        readme_path.to_str().expect("README path should be UTF-8"),
        "-m",
        manifest_path
            .to_str()
            .expect("manifest path should be UTF-8"),
    ]);
    assert!(
        write_output.status.success(),
        "stderr:\n{}",
        normalize(&write_output.stderr)
    );

    let rewritten = fs::read_to_string(&readme_path).expect("failed to read injected README");
    assert!(rewritten.contains("<!-- feature-manifest:start -->"));
    assert!(rewritten.contains("| `runtime` | no | public | stable | runtime | `dep:tokio`, `tokio/rt` | Enable the Tokio runtime integration. |"));
    assert!(rewritten.contains("_1 internal/private feature(s) hidden."));
    assert!(
        rewritten.contains(
            "- `api`: Public API and internal implementation toggles. Members: `public-api`, `internal-codegen`."
        ),
        "README should keep group members visible inside the generated region"
    );

    let check_output = run_command(&[
        "md",
        "--check",
        "-i",
        readme_path.to_str().expect("README path should be UTF-8"),
        "-m",
        manifest_path
            .to_str()
            .expect("manifest path should be UTF-8"),
    ]);
    assert!(
        check_output.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        normalize(&check_output.stdout),
        normalize(&check_output.stderr)
    );
}
