# 1.0 Roadmap

`1.0` should mean downstream users can rely on the metadata format, lint names,
CLI commands, and JSON contracts without routine breaking changes.

## Stability Targets

- Stable metadata format with documented compatibility expectations.
- Stable, versioned JSON Schema files and documented compatibility expectations.
- Stable lint names and default severities.
- Stable command names and primary flags for `init`, `doctor`, `check`,
  `markdown`, `json`, `graph`, `sync`, `explain`, and `list-lints`.

## Current Compatibility Boundaries

Treated as compatibility-sensitive before `1.0`:

- metadata table names and field meanings documented in `metadata-format.md`,
- primary CLI commands and long flags,
- JSON `schema_version = 1` output contracts,
- lint codes used by `check`, JSON, GitHub annotations, and SARIF.

Still allowed to move before `1.0`:

- short aliases such as `c`, `md`, and `s`,
- exact human-readable diagnostic wording,
- library module layout and exported Rust types,
- experimental templates or presets added for docs.rs and editor workflows.

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
