//! Library API for the `cargo-feature-manifest` and `cargo-fm` tools.
//!
//! The crate reads Cargo feature definitions, attaches maintainer-authored
//! metadata, validates common documentation and policy mistakes, and renders the
//! result for docs or automation.
//!
//! Typical entry points:
//!
//! - [`load_workspace`] to discover workspace packages via `cargo metadata`.
//! - [`load_manifest`] to read a single `Cargo.toml`.
//! - [`validate`] to lint feature metadata.
//! - [`render_markdown`] to generate documentation output.
//! - [`render_mermaid`] to visualize feature relationships.
//! - [`render_json`] to emit a versioned machine-readable schema.
//! - [`sync_manifest`] to scaffold or normalize metadata tables.
//! - [`preview_sync_manifest`] to inspect sync rewrites before writing.

mod discover;
mod docs_io;
mod json_output;
mod model;
mod parse;
mod render;
mod source_map;
mod validate;

pub use discover::{PackageSelection, load_workspace, resolve_manifest_path};
pub use docs_io::{
    InjectionMarkers, InjectionReport, MarkerReport, ensure_injection_markers,
    inject_between_markers, injected_region_matches, inspect_markers, output_matches, write_output,
};
pub use json_output::render_json;
pub use model::{
    DependencyInfo, Feature, FeatureGroup, FeatureManifest, FeatureMetadata, FeatureRef, LintLevel,
    LintPreset, MetadataLayout, WorkspaceManifest,
};
pub use parse::{
    FEATURE_DOCS_METADATA_TABLE, FEATURE_MANIFEST_METADATA_TABLE, SyncOptions, SyncPreview,
    SyncReport, load_manifest, parse_manifest_str, preview_sync_manifest, render_sync_diff,
    sync_manifest,
};
pub use render::{render_explain, render_markdown, render_mermaid};
pub use source_map::{ManifestSourceMap, SourceSpan};
pub use validate::{
    Issue, KNOWN_LINT_CODES, LintDoc, Severity, ValidateOptions, ValidationReport,
    known_lint_codes, lint_docs, parse_lint_override, validate, validate_with_options,
};

#[doc(hidden)]
pub mod cli;

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_MANIFEST: &str = r#"
[package]
name = "demo"
version = "0.1.0"

[features]
default = ["serde", "tokio?/rt"]
serde = ["dep:serde"]
tokio = ["dep:tokio", "std"]
std = []
unstable = []

[package.metadata.feature-manifest]
serde = { description = "Enable serde support." }
tokio = { description = "Enable Tokio-backed APIs." }
std = { description = "Enable std support." }
unused = { description = "Not a real feature." }

[package.metadata.feature-manifest.lints]
small-group = "deny"

[[package.metadata.feature-manifest.groups]]
name = "runtime"
members = ["tokio", "unstable"]
mutually_exclusive = true
"#;

    #[test]
    fn parses_typed_feature_references() {
        let manifest = parse_manifest_str(SAMPLE_MANIFEST, "Cargo.toml").unwrap();
        assert_eq!(manifest.package_name.as_deref(), Some("demo"));
        assert_eq!(manifest.features.len(), 4);
        assert_eq!(
            manifest.default_members,
            vec![
                FeatureRef::Feature {
                    name: "serde".to_owned()
                },
                FeatureRef::DependencyFeature {
                    dependency: "tokio".to_owned(),
                    feature: "rt".to_owned(),
                    weak: true
                }
            ]
        );
        assert_eq!(
            manifest.features["tokio"].enables,
            vec![
                FeatureRef::Dependency {
                    name: "tokio".to_owned()
                },
                FeatureRef::Feature {
                    name: "std".to_owned()
                }
            ]
        );
        assert_eq!(manifest.lint_overrides["small-group"], LintLevel::Deny);
    }

    #[test]
    fn parses_structured_metadata_table() {
        let manifest = parse_manifest_str(
            r#"
[package]
name = "demo"
version = "0.1.0"

[features]
cli = []

[package.metadata.feature-manifest.features]
cli = "Enable the CLI layer."
"#,
            "Cargo.toml",
        )
        .unwrap();

        let cli = &manifest.features["cli"];
        assert!(cli.has_metadata);
        assert_eq!(
            cli.metadata.description.as_deref(),
            Some("Enable the CLI layer.")
        );
    }

    #[test]
    fn validation_reports_missing_and_stale_metadata() {
        let manifest = parse_manifest_str(SAMPLE_MANIFEST, "Cargo.toml").unwrap();
        let report = validate(&manifest);

        assert!(report.has_errors());
        assert!(
            report
                .issues
                .iter()
                .any(|issue| issue.code == "missing-metadata"
                    && issue.feature.as_deref() == Some("unstable"))
        );
        assert!(
            report
                .issues
                .iter()
                .any(|issue| issue.code == "unknown-metadata"
                    && issue.feature.as_deref() == Some("unused"))
        );
    }

    #[test]
    fn lint_overrides_can_downgrade_or_silence_issues() {
        let manifest = parse_manifest_str(
            r#"
[package]
name = "demo"
version = "0.1.0"

[features]
alpha = []

[package.metadata.feature-manifest]
"#,
            "Cargo.toml",
        )
        .unwrap();

        let downgraded = validate_with_options(
            &manifest,
            &ValidateOptions::with_cli_lint_overrides([(
                "missing-metadata".to_owned(),
                LintLevel::Warn,
            )]),
        );
        assert!(downgraded.warning_count() >= 1);
        assert_eq!(downgraded.error_count(), 1);

        let silenced = validate_with_options(
            &manifest,
            &ValidateOptions::with_cli_lint_overrides([
                ("missing-metadata".to_owned(), LintLevel::Allow),
                ("missing-description".to_owned(), LintLevel::Allow),
            ]),
        );
        assert_eq!(silenced.issues.len(), 0);
    }

    #[test]
    fn validation_reports_mutually_exclusive_default_conflicts() {
        let manifest = parse_manifest_str(
            r#"
[package]
name = "demo"
version = "0.1.0"

[features]
default = ["native-tls", "rustls"]
native-tls = []
rustls = []

[package.metadata.feature-manifest]
native-tls = { description = "Use native-tls." }
rustls = { description = "Use rustls." }

[[package.metadata.feature-manifest.groups]]
name = "tls"
members = ["native-tls", "rustls"]
mutually_exclusive = true
"#,
            "Cargo.toml",
        )
        .unwrap();

        let report = validate(&manifest);
        assert!(
            report
                .issues
                .iter()
                .any(|issue| issue.code == "mutually-exclusive-default")
        );
    }

    #[test]
    fn validation_allows_default_optional_dependency_features() {
        let manifest = parse_manifest_str(
            r#"
[package]
name = "demo"
version = "0.1.0"

[dependencies]
serde = { version = "1", optional = true }

[features]
default = ["serde"]
"#,
            "Cargo.toml",
        )
        .unwrap();

        let report = validate(&manifest);
        assert!(
            !report
                .issues
                .iter()
                .any(|issue| issue.code == "unknown-default-member")
        );
    }

    #[test]
    fn validation_reports_unknown_plain_feature_references() {
        let mut manifest = parse_manifest_str(
            r#"
[package]
name = "demo"
version = "0.1.0"

[features]
tls = ["native-tls"]

[package.metadata.feature-manifest]
tls = { description = "Enable TLS support." }
"#,
            "Cargo.toml",
        )
        .unwrap();
        manifest.dependencies.insert(
            "serde".to_owned(),
            DependencyInfo {
                key: "serde".to_owned(),
                package: "serde".to_owned(),
                optional: true,
            },
        );

        let report = validate(&manifest);
        assert!(report.issues.iter().any(|issue| {
            issue.code == "unknown-feature-reference" && issue.feature.as_deref() == Some("tls")
        }));
    }

    #[test]
    fn validation_allows_plain_optional_dependency_references() {
        let manifest = parse_manifest_str(
            r#"
[package]
name = "demo"
version = "0.1.0"

[dependencies]
native-tls = { version = "1", optional = true }

[features]
tls = ["native-tls"]

[package.metadata.feature-manifest]
tls = { description = "Enable TLS support." }
"#,
            "Cargo.toml",
        )
        .unwrap();

        let report = validate(&manifest);
        assert!(
            !report
                .issues
                .iter()
                .any(|issue| issue.code == "unknown-feature-reference")
        );
    }

    #[test]
    fn markdown_hides_private_features_by_default_and_shows_default_summary() {
        let manifest = parse_manifest_str(
            r#"
[package]
name = "demo"
version = "0.1.0"

[features]
default = ["public-api"]
public-api = []
internal = []

[package.metadata.feature-manifest]
public-api = { description = "Stable public API surface." }
internal = { description = "Internal glue.", public = false }
"#,
            "Cargo.toml",
        )
        .unwrap();

        let workspace = WorkspaceManifest {
            root_manifest_path: "Cargo.toml".into(),
            packages: vec![manifest],
        };
        let markdown = render_markdown(&workspace, false);
        assert!(markdown.contains("Default feature set: `public-api`"));
        assert!(markdown.contains("public-api"));
        assert!(!markdown.contains("| `internal` |"));
        assert!(markdown.contains("internal/private feature(s) hidden"));
    }

    #[test]
    fn source_map_finds_feature_metadata_and_group_spans() {
        let source = r#"
[features]
serde = []
"tls+rustls" = []

[package.metadata.feature-manifest.features]
serde = { description = "" }

[[package.metadata.feature-manifest.groups]]
name = "tls"
members = ["tls+rustls"]
"#;
        let map = ManifestSourceMap::new(source);

        assert_eq!(
            map.feature_key_span("tls+rustls"),
            Some(SourceSpan { line: 4, column: 1 })
        );
        assert_eq!(
            map.metadata_key_span("serde"),
            Some(SourceSpan { line: 7, column: 1 })
        );
        assert_eq!(
            map.group_name_span("tls"),
            Some(SourceSpan {
                line: 10,
                column: 1
            })
        );
    }

    #[test]
    fn sync_diff_keeps_insertions_readable() {
        let diff = render_sync_diff(
            std::path::Path::new("Cargo.toml"),
            "a\nb\nc\n",
            "a\nb\nnew\nc\n",
        );

        assert!(diff.contains("\n b\n+new\n c\n"));
        assert!(!diff.contains("-c\n+new"));
    }

    #[test]
    fn lint_docs_match_known_codes() {
        let documented = lint_docs().iter().map(|lint| lint.code).collect::<Vec<_>>();

        assert_eq!(documented, known_lint_codes());
    }
}
