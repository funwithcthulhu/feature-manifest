# feature-manifest

[![Crates.io](https://img.shields.io/crates/v/feature-manifest.svg)](https://crates.io/crates/feature-manifest)
[![Docs.rs](https://docs.rs/feature-manifest/badge.svg)](https://docs.rs/feature-manifest)
[![CI](https://github.com/funwithcthulhu/feature-manifest/actions/workflows/ci.yml/badge.svg)](https://github.com/funwithcthulhu/feature-manifest/actions/workflows/ci.yml)
[![License](https://img.shields.io/crates/l/feature-manifest.svg)](LICENSE-MIT)

`feature-manifest` is a Rust crate and Cargo subcommand for documenting,
validating, and rendering Cargo feature flags.

Cargo feature tables are precise enough for builds, but often too terse for
users. A crate can start with:

```toml
[features]
default = ["serde"]
serde = ["dep:serde"]
tokio = ["dep:tokio"]
```

Then keep the missing intent beside it:

```toml
[package.metadata.feature-manifest.features]
serde = { description = "Enable Serialize and Deserialize support." }
tokio = { description = "Enable Tokio-backed async APIs." }
```

From there, `feature-manifest` can generate a README feature table and fail CI
when the table drifts:

```text
cargo fm markdown --insert-into README.md
cargo fm markdown --check --insert-into README.md
```

The same commands can be used in CI when a project wants generated feature
documentation to be checked rather than updated by hand.

## Should I Use This?

Use it if:

- your crate has more than a few public features,
- your README or docs.rs page has a feature table,
- feature documentation drifts during releases,
- your workspace repeats feature patterns across packages.

Probably skip it if:

- your crate has no optional features,
- all features are internal and intentionally undocumented,
- you do not want feature metadata in `Cargo.toml`.

## Installation

From crates.io:

```text
cargo install feature-manifest
```

This installs both `cargo-feature-manifest` and the short alias `cargo-fm`.
`feature-manifest` currently requires Rust 1.85 or newer because it uses the
2024 edition. Since it runs as a developer tool, that does not change the MSRV
of crates it checks.

Use the short Cargo subcommand:

```text
cargo fm
```

The original long form still works:

```text
cargo feature-manifest
```

From source:

```text
git clone https://github.com/funwithcthulhu/feature-manifest.git
cd feature-manifest
cargo install --path .
```

## Smallest Useful Setup

```text
cargo install feature-manifest
cargo fm init --dry-run
cargo fm init
cargo fm markdown --insert-into README.md
cargo fm markdown --check --insert-into README.md
```

## Common Commands

```text
cargo fm check
cargo fm init --dry-run
cargo fm doctor --explain
cargo fm markdown --write FEATURES.md
cargo fm markdown --check --insert-into README.md
cargo fm json
cargo fm graph
cargo fm sync --diff
cargo fm explain <feature>
```

The default command is `check`, so `cargo fm` and `cargo feature-manifest` are
both valid shorthand.

See the [generated CLI reference](docs/cli.md) for the full command surface,
including schemas, completions, and lint-reference generation.

Short aliases are available for daily use:

- `check` -> `c`, `chk`
- `markdown` -> `md`, `m`
- `json` -> `j`
- `graph` -> `g`, `viz`
- `sync` -> `s`
- `explain` -> `show`, `x`
- `list-lints` -> `lints`

## Workspace Support

Point the tool at a workspace root or a single crate:

```text
cargo fm --workspace check --manifest-path path/to/workspace
cargo fm --package my-crate explain serde --manifest-path path/to/workspace
cargo fm markdown --manifest-path path/to/crate
```

When a workspace has multiple members, the default behavior is intentionally
strict: you must choose `--workspace` or `--package <name>`.

## Markdown Output and Injection

Write a generated document directly:

```text
cargo fm markdown --write FEATURES.md
```

Inject generated Markdown into an existing README using markers:

```markdown
<!-- feature-manifest:start -->

# feature-manifest feature manifest

Default feature set: _none_

| Feature | Default | Visibility | Status | Category | Enables | Description |
| --- | --- | --- | --- | --- | --- | --- |
<!-- feature-manifest:end -->
```

Then run:

```text
cargo fm markdown --insert-into README.md
```

Check whether generated docs are stale:

```text
cargo fm markdown --check --insert-into README.md
```

Custom markers are supported with `--start-marker` and `--end-marker`.

## Additional Output Formats

`check` supports multiple output formats:

- `text`: default human-readable output
- `json`: machine-readable structured report
- `github`: GitHub Actions workflow commands
- `sarif`: SARIF 2.1.0 for code scanning pipelines

Example:

```text
cargo fm check --format sarif > feature-manifest.sarif
```

GitHub annotations include manifest line numbers when the relevant feature,
metadata entry, or group can be located.

## Lint Configuration

`feature-manifest` lints can be configured in `Cargo.toml`:

```toml
[package.metadata.feature-manifest.lints]
missing-description = "deny"
small-group = "allow"
private-enabled-by-public = "warn"
```

For staged adoption or strict CI defaults:

```toml
[package.metadata.feature-manifest]
preset = "adopt"
```

You can also override them per-run:

```text
cargo fm check --lint missing-description=warn
cargo fm check --preset strict
```

For stricter project setup checks:

```text
cargo fm doctor --strict
cargo fm doctor --explain
```

See [docs/lints.md](docs/lints.md) for the generated lint reference.

## Stability

The CLI and metadata format are the primary supported surface before `1.0`.
The library API is public for integration tests and early tooling, but it may
still move while the crate is pre-1.0. See the [1.0 roadmap](docs/roadmap-1.0.md)
for the stabilization checklist.

## Documentation

- Start using it: [Getting started](docs/getting-started.md),
  [CI setup](docs/ci.md), [Adoption recipes](docs/adoption-recipes.md),
  [Cookbook](docs/cookbook.md)
- Metadata and output: [Metadata format](docs/metadata-format.md),
  [Lint reference](docs/lints.md), [JSON schema](docs/json-schema.md),
  [Generated CLI reference](docs/cli.md)
- Migration examples: [Before and after adoption](docs/before-after-adoption.md),
  [Compatibility and migration](docs/compatibility-and-migration.md),
  [Common feature patterns](docs/real-world-patterns.md)
- Project and policy: [Dogfooding](docs/dogfooding.md),
  [Architecture](docs/architecture.md),
  [Supply-chain trust](docs/supply-chain-trust.md),
  [1.0 roadmap](docs/roadmap-1.0.md),
  [Security policy](SECURITY.md), [Contributing guide](CONTRIBUTING.md),
  [Support](SUPPORT.md)

Example metadata snippets live in [examples](examples).

## Contributing

Development setup, test guidance, and release steps live in
[CONTRIBUTING.md](CONTRIBUTING.md) and [docs/releasing.md](docs/releasing.md).

## License

Licensed under either of the following, at your option:

- Apache License, Version 2.0
- MIT license
