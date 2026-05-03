# 1.0 Roadmap

`feature-manifest` is already useful as a maintainer tool. A good `1.0`
milestone should mean that downstream users can rely on the crate and CLI
without worrying that core concepts will keep moving.

## Stability Targets

- Stable metadata format with documented compatibility expectations.
- Stable JSON schema versioning policy.
- Stable lint names and default severities.
- Stable command names and primary flags for `init`, `doctor`, `check`,
  `markdown`, `json`, `graph`, `sync`, `explain`, and `list-lints`.

## Product Goals

- Keep single-crate workflows frictionless.
- Make workspace support first-class.
- Provide CI-friendly and editor-friendly output formats.
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
