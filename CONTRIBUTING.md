# Contributing

Thanks for improving Cargo feature documentation.

## Good First Contributions

Useful starter issues include:

- new real-world metadata examples,
- documentation clarifications,
- fixture coverage for less common `Cargo.toml` layouts,
- lint proposal write-ups using the issue template,
- small CLI ergonomics improvements with tests.

## Development

```text
cargo fmt
cargo test --all-targets
cargo fm doctor --explain
cargo publish --dry-run --locked
```

When CLI flags or help text change, regenerate the CLI reference:

```text
cargo fm help-markdown > docs/cli.md
```

When lint behavior or descriptions change, regenerate the lint reference:

```text
cargo fm lints --markdown > docs/lints.md
```

## Design Principles

- Keep the crate focused on feature metadata, validation, and rendering.
- Prefer explicit, predictable output formats over implicit inference.
- Make rewrite commands previewable before they write.
- Treat generated docs and schemas as public contracts.
- Add fixtures for manifest shapes that exposed bugs or ambiguity.

## Pull Requests

Please include:

- a short explanation of the user-facing behavior,
- tests or fixtures for behavior changes,
- docs updates for new flags, metadata fields, or lints,
- any release-readiness notes if the change affects publishing.
