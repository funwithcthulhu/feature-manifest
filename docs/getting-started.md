# Getting Started

This is the shortest path from a crate with Cargo features to documented,
checked, generated feature docs.

## Install

```text
cargo install feature-manifest
```

That installs both entrypoints:

```text
cargo fm
cargo feature-manifest
```

## Initialize A Crate

From your crate root:

```text
cargo fm init --ci
```

This scaffolds missing feature metadata, adds README markers, writes the current
feature table into that README section, and creates a GitHub Actions workflow.

## Fill In Metadata

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

Use `doctor` when you want to check project wiring too:

```text
cargo fm doctor
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
