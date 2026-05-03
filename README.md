# feature-manifest

[![Crates.io](https://img.shields.io/crates/v/feature-manifest.svg)](https://crates.io/crates/feature-manifest)
[![Docs.rs](https://docs.rs/feature-manifest/badge.svg)](https://docs.rs/feature-manifest)
[![CI](https://github.com/funwithcthulhu/feature-manifest/actions/workflows/ci.yml/badge.svg)](https://github.com/funwithcthulhu/feature-manifest/actions/workflows/ci.yml)
[![License](https://img.shields.io/crates/l/feature-manifest.svg)](LICENSE-MIT)

`feature-manifest` is a Rust crate plus Cargo subcommand for documenting, validating, and rendering Cargo feature flags.

It gives crate authors a structured layer on top of `[features]`, so feature intent can live in `Cargo.toml` while still powering docs, CI, editor tooling, workspace audits, and release automation.

## Why it exists

Cargo features are powerful, but feature intent is often trapped in a raw `[features]` table. `feature-manifest` helps maintainers:

- keep every feature documented in one place,
- fail CI when metadata drifts out of sync,
- scaffold missing metadata automatically,
- generate Markdown for docs and READMEs,
- emit stable JSON and SARIF for tooling,
- visualize feature relationships with Mermaid,
- work across whole Cargo workspaces.

## Installation

From crates.io:

```text
cargo install feature-manifest
```

This installs both `cargo-feature-manifest` and the short alias `cargo-fm`.

Recommended usage:

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

## Commands

```text
cargo fm
cargo fm init --ci
cargo fm doctor
cargo fm c --format json
cargo fm md -o FEATURES.md
cargo fm md --check -i README.md
cargo fm md -i README.md
cargo fm j
cargo fm g
cargo fm s
cargo fm show <feature>
cargo fm lints
```

The default command is still `check`, so `cargo fm` and `cargo feature-manifest` are both valid shorthand.

Short aliases:

- `check` -> `c`, `chk`
- `markdown` -> `md`, `m`
- `json` -> `j`
- `graph` -> `g`, `viz`
- `sync` -> `s`
- `explain` -> `show`, `x`
- `list-lints` -> `lints`

## Quick Workflow

1. Add or change features in `Cargo.toml`.
2. Run `cargo fm init --ci` to scaffold metadata, README markers, and CI.
3. Fill in real descriptions, visibility, and status flags.
4. Run `cargo fm doctor` to confirm the project is wired up.
5. Run `cargo fm` locally and in CI.

## Workspace Support

Point the tool at a workspace root or a single crate:

```text
cargo fm -w c -m path/to/workspace
cargo fm -p my-crate show serde -m path/to/workspace
cargo fm md -m path/to/crate
```

When a workspace has multiple members, the default behavior is intentionally strict: you must choose `--workspace` or `--package <name>`.

## Markdown Output and Injection

Write a generated document directly:

```text
cargo fm md -o FEATURES.md
```

Inject generated Markdown into an existing README using markers:

```markdown
<!-- feature-manifest:start -->
<!-- feature-manifest:end -->
```

Then run:

```text
cargo fm md -i README.md
```

Check whether generated docs are stale:

```text
cargo fm md --check -i README.md
```

Custom markers are supported with `--start-marker` and `--end-marker`.

## Validation Output Formats

`check` supports multiple output formats:

- `text`: default human-readable output
- `json`: machine-readable structured report
- `github`: GitHub Actions workflow commands
- `sarif`: SARIF 2.1.0 for code scanning pipelines

Example:

```text
cargo fm c -f sarif > feature-manifest.sarif
```

## Lint Configuration

Feature-manifest lints can be configured in `Cargo.toml`:

```toml
[package.metadata.feature-manifest.lints]
missing-description = "deny"
small-group = "allow"
private-enabled-by-public = "warn"
```

For gradual adoption or strict CI defaults:

```toml
[package.metadata.feature-manifest]
preset = "adopt"
```

You can also override them per-run:

```text
cargo fm c -l missing-description=warn
cargo fm c --preset strict
```

See [docs/metadata-format.md](docs/metadata-format.md) for the full list of lint names and meanings.

## More Documentation

- [Metadata format](docs/metadata-format.md)
- [JSON schema](docs/json-schema.md)
- [Getting started](docs/getting-started.md)
- [CI setup](docs/ci.md)
- [Cookbook](docs/cookbook.md)
- [Compatibility and migration](docs/compatibility-and-migration.md)
- [Real-world patterns](docs/real-world-patterns.md)
- [Release process](docs/releasing.md)
- [1.0 roadmap](docs/roadmap-1.0.md)

Example metadata snippets live in [examples](examples).

## Fixtures and Tests

The repo includes valid Cargo fixtures for both single-package and workspace flows:

- [`fixtures/basic/Cargo.toml`](fixtures/basic/Cargo.toml)
- [`fixtures/edge/Cargo.toml`](fixtures/edge/Cargo.toml)
- [`fixtures/messy/Cargo.toml`](fixtures/messy/Cargo.toml)
- [`fixtures/workspace/Cargo.toml`](fixtures/workspace/Cargo.toml)

The test suite includes:

- unit tests for parsing and validation,
- integration tests for CLI workflows,
- snapshot tests for Markdown, JSON, and Mermaid output.

Run everything with:

```text
cargo test
```

## Publish Readiness

Before publishing:

```text
cargo fmt
cargo test
cargo publish --dry-run
```

For the project’s automated release flow, see [docs/releasing.md](docs/releasing.md).

## License

Licensed under either of the following, at your option:

- Apache License, Version 2.0
- MIT license
