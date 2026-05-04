# Getting Started

This is a minimal setup for a crate that already has Cargo features and wants
checked, generated feature documentation.

## Install

```text
cargo install feature-manifest
```

That installs both entrypoints:

```text
cargo fm
cargo feature-manifest
```

## Initialize a Crate

From your crate root:

```text
cargo fm init --dry-run --ci
cargo fm init --ci
```

This scaffolds missing feature metadata, adds README markers, writes the current
feature table into that README section, and creates a GitHub Actions workflow.
The dry run shows the same setup plan without touching files.

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

Use `doctor` when you want to check project wiring too:

```text
cargo fm doctor --explain
```

Use strict doctor mode when warnings should fail CI:

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
