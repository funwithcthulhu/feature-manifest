mod common;

use common::{fixture_path, normalize, read_snapshot, run_command};

#[test]
fn markdown_output_matches_snapshot() {
    let manifest_path = fixture_path("basic");
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
    assert_eq!(
        normalize(&output.stdout),
        read_snapshot("basic.markdown.md")
    );
}

#[test]
fn graph_output_matches_snapshot() {
    let manifest_path = fixture_path("basic");
    let output = run_command(&[
        "graph",
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
    assert_eq!(normalize(&output.stdout), read_snapshot("basic.graph.mmd"));
}

#[test]
fn json_output_matches_snapshot() {
    let manifest_path = fixture_path("basic");
    let output = run_command(&[
        "json",
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
    assert_eq!(normalize(&output.stdout), read_snapshot("basic.json"));
}
