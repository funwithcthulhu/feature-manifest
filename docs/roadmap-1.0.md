# 1.0 Roadmap

`1.0` should mean downstream users can rely on the metadata format, lint names,
CLI commands, and JSON contracts without routine breaking changes.

## Stability Targets

- Stable metadata format with documented compatibility expectations.
- Stable, versioned JSON Schema files and compatibility guarantees.
- Stable lint names and default severities.
- Stable command names and primary flags for `init`, `doctor`, `check`,
  `markdown`, `json`, `graph`, `sync`, `explain`, and `list-lints`.

## Design Goals

- Keep single-crate workflows short and predictable.
- Keep workspace support documented and tested.
- Provide output formats that fit CI and editor integrations.
- Stay focused on documenting and validating feature intent rather than growing
  into a full Cargo feature management system.

## Candidate 1.0 Additions

- Optional docs.rs-oriented templates or presets.
- More targeted lint configuration such as package-scoped or code-scoped policy
  files.
- Additional output consumers such as editor integrations.
- Migration guidance if Cargo eventually standardizes feature metadata.

## Non-Goals

- Replacing Cargo's own feature resolution.
- Exhaustive feature powerset testing.
- Managing runtime configuration outside Cargo features.
