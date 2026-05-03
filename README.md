# feature-manifest

`feature-manifest` is a small Rust crate plus Cargo subcommand for documenting, validating, and rendering Cargo feature flags.

It gives crate authors a place to describe feature intent today, using `Cargo.toml`, while also producing outputs that are useful for README tables, docs pages, CI, and editor tooling.

## Commands

```text
cargo feature-manifest check
cargo feature-manifest markdown > FEATURES.md
cargo feature-manifest json
cargo feature-manifest graph
```

During local development, you can run the same commands with:

```text
cargo run -- check
cargo run -- markdown
cargo run -- json
cargo run -- graph
```

## Metadata format

Structured form:

```toml
[features]
default = ["serde"]
serde = ["dep:serde"]
tokio = ["dep:tokio"]
unstable = []
internal-codegen = []

[package.metadata.feature-manifest.features]
serde = { description = "Enables Serialize/Deserialize impls." }
tokio = { description = "Enables async APIs backed by Tokio." }
unstable = { description = "Experimental APIs; semver not guaranteed.", unstable = true }
internal-codegen = { description = "Internal codegen support.", public = false }

[[package.metadata.feature-manifest.groups]]
name = "runtime"
description = "Choose one async runtime backend."
members = ["tokio", "async-std"]
mutually_exclusive = true
```

Flat shorthand is also supported:

```toml
[package.metadata.feature-manifest]
serde = "Enables Serialize/Deserialize impls."
tokio = { description = "Enables async APIs backed by Tokio." }
```

## What `check` validates

- Features declared in `[features]` but missing metadata.
- Metadata entries that do not correspond to a real feature.
- Empty or missing descriptions.
- Unstable, deprecated, or private features that are enabled by default without `allow_default = true`.
- Mutually exclusive groups that default-enable more than one member.
- Unknown or duplicate feature names inside configured groups.

## Output goals

- `markdown` produces a docs-friendly feature table.
- `json` emits normalized machine-readable metadata.
- `graph` emits a Mermaid dependency graph for feature relationships.

## Why this layout

The crate is split into a reusable library and a `cargo-feature-manifest` binary so the validation logic stays testable and future integrations can call the library directly.
