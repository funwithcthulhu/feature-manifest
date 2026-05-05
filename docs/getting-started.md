# Getting Started

Use this path when a crate already has Cargo features and needs generated,
checked feature documentation.

## Install

```text
cargo install feature-manifest
```

The install provides both entrypoints:

```text
cargo fm
cargo feature-manifest
```

## Initialize a Crate

From your crate root:

```text
cargo fm init --dry-run
cargo fm init
```

The dry run prints the planned edits. The second command scaffolds missing
feature metadata, adds README markers, and writes the current feature table into
that section.

Add `--ci` when you also want a starter GitHub Actions workflow.

## Fill in Metadata

Open `Cargo.toml` and replace generated TODO descriptions with real text:

```toml
[package.metadata.feature-manifest.features]
serde = { description = "Enable Serialize and Deserialize support.", category = "serialization" }
tokio = { description = "Enable Tokio-backed async APIs.", category = "runtime" }
```

## Validate

```text
cargo fm
```

Check repository wiring as well as feature metadata:

```text
cargo fm doctor --explain
```

Use strict mode when CI should fail on setup warnings:

```text
cargo fm doctor --strict
```

## Keep Docs Fresh

Regenerate the README section:

```text
cargo fm md -i README.md
```

Fail CI when it drifts:

```text
cargo fm md --check -i README.md
```
