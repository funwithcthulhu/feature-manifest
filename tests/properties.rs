use std::collections::BTreeSet;
use std::fs;

use feature_manifest::{
    InjectionMarkers, MetadataLayout, SyncOptions, inject_between_markers, injected_region_matches,
    output_matches, parse_manifest_str, sync_manifest, write_output,
};
use proptest::prelude::*;

fn feature_name() -> impl Strategy<Value = String> {
    "[a-z][a-z0-9_-]{0,10}".prop_filter("feature name must not be `default`", |name| {
        name != "default"
    })
}

fn safe_text() -> impl Strategy<Value = String> {
    "[A-Za-z0-9 .,;:!?_()/-]{0,120}"
        .prop_filter("text must not contain injection markers", |text| {
            !text.contains("feature-manifest:start") && !text.contains("feature-manifest:end")
        })
}

proptest! {
    #[test]
    fn sync_manifest_outputs_parseable_idempotent_toml(
        features in prop::collection::btree_set(feature_name(), 1..12),
    ) {
        let temp_dir = tempfile::tempdir().expect("failed to create temp dir");
        let manifest_path = temp_dir.path().join("Cargo.toml");
        let manifest = manifest_with_features(&features);
        fs::write(&manifest_path, manifest).expect("failed to write generated manifest");

        let report = sync_manifest(&manifest_path, &SyncOptions::default())
            .expect("sync should handle generated manifests");
        prop_assert!(report.changed());

        let rewritten = fs::read_to_string(&manifest_path).expect("failed to read rewritten manifest");
        prop_assert!(rewritten.contains("# package comments should survive sync"));
        let parsed = parse_manifest_str(&rewritten, &manifest_path)
            .expect("rewritten manifest should remain parseable");
        prop_assert_eq!(parsed.features.len(), features.len());
        prop_assert!(parsed.features.values().all(|feature| feature.has_metadata));

        let check = sync_manifest(
            &manifest_path,
            &SyncOptions {
                check_only: true,
                remove_stale: true,
                style: Some(MetadataLayout::Structured),
            },
        )
        .expect("second sync check should succeed");
        prop_assert!(!check.changed());
    }

    #[test]
    fn marker_injection_preserves_surrounding_document(
        before in safe_text(),
        after in safe_text(),
        generated in safe_text(),
    ) {
        let temp_dir = tempfile::tempdir().expect("failed to create temp dir");
        let readme_path = temp_dir.path().join("README.md");
        let markers = InjectionMarkers::default();
        fs::write(
            &readme_path,
            format!("{before}\n{}\nold\n{}\n{after}\n", markers.start, markers.end),
        )
        .expect("failed to write generated README");

        inject_between_markers(&readme_path, &generated, &markers)
            .expect("injection should succeed");
        let injected = fs::read_to_string(&readme_path).expect("failed to read injected README");
        let expected_suffix = format!("{after}\n");
        prop_assert!(injected.starts_with(&before));
        prop_assert!(injected.ends_with(&expected_suffix));
        prop_assert!(
            injected_region_matches(&readme_path, &generated, &markers)
                .expect("injected region check should succeed")
        );
    }

    #[test]
    fn generated_output_checks_ignore_platform_line_endings(
        generated in safe_text(),
    ) {
        let temp_dir = tempfile::tempdir().expect("failed to create temp dir");
        let output_path = temp_dir.path().join("FEATURES.md");

        write_output(&output_path, &generated).expect("failed to write generated output");
        let crlf = fs::read_to_string(&output_path)
            .expect("failed to read generated output")
            .replace('\n', "\r\n");
        fs::write(&output_path, crlf).expect("failed to write CRLF generated output");

        prop_assert!(
            output_matches(&output_path, &generated)
                .expect("generated output check should succeed")
        );
    }
}

fn manifest_with_features(features: &BTreeSet<String>) -> String {
    let mut manifest = String::from(
        r#"[package]
name = "generated-fixture"
version = "0.1.0"

# package comments should survive sync
[features]
default = []
"#,
    );

    for feature in features {
        manifest.push_str(&format!("{feature} = []\n"));
    }

    manifest
}
