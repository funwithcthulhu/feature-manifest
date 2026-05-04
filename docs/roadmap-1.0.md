# 1.0 Roadmap

`1.0` should mark the point where downstream users can rely on the metadata
format, lint names, CLI surface, and JSON contracts without expecting routine
breaking changes.

## Stability Targets

- Stable metadata format with documented compatibility expectations.
- Stable, versioned JSON Schema files and compatibility guarantees.
- Stable lint names and default severities.
- Stable command names and primary flags for `init`, `doctor`, `check`,
  `markdown`, `json`, `graph`, `sync`, `explain`, and `list-lints`.

## Product Goals

- Keep single-crate workflows short and predictable.
- Make workspace support first-class.
- Provide output formats that work cleanly in CI and editor integrations.
- Stay focused on documenting and validating feature intent rather than growing
  into a full Cargo feature management system.

## Candidate 1.0 Additions

- Optional docs.rs-oriented templates or presets.
- More targeted lint configuration such as package-scoped or code-scoped policy
  files.
- Additional output consumers such as editor integrations.
- Clear guidance for eventual Cargo-native feature metadata if the ecosystem
  standardizes there.

## Non-Goals

- Replacing Cargo's own feature resolution.
- Exhaustive feature powerset testing.
- Managing runtime configuration outside Cargo features.
