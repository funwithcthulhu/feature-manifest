# Real-World Patterns

These examples show how maintainers often map common Rust feature styles into
feature-manifest metadata.

The repository also keeps compatibility fixtures for these patterns under
`fixtures/compat`. They are intentionally small, but they mirror layouts seen in
published crates: workspace-inherited fields, TLS backend families, async
runtime toggles, optional serialization, and `no_std` surfaces.

## Case Study: Small Library

A small crate usually needs very little ceremony. Start with public features,
keep the default set obvious, and use `category` only when it helps readers scan
the generated table.

```toml
[features]
default = ["std"]
std = []
serde = ["dep:serde"]

[package.metadata.feature-manifest.features]
std = { description = "Enable APIs that depend on the Rust standard library.", category = "platform" }
serde = { description = "Enable Serialize and Deserialize support.", category = "serialization" }
```

## Case Study: Workspace Package

Workspace packages often inherit dependency versions and package fields. The
metadata still belongs in the package that owns the features:

```toml
[package]
version.workspace = true
edition.workspace = true

[dependencies]
tokio = { workspace = true, optional = true }

[features]
cli = ["dep:tokio", "tokio/rt"]

[package.metadata.feature-manifest.features]
cli = { description = "Enable command-line helpers backed by Tokio.", category = "cli" }
```

Use `cargo fm -w` when generated output should cover every workspace member, or
`cargo fm -p package-name` when one member owns the public feature surface.

## Runtime Backends

```toml
[features]
tokio = ["dep:tokio"]
async-std = ["dep:async-std"]

[package.metadata.feature-manifest.features]
tokio = { description = "Enable Tokio-backed async APIs.", category = "runtime" }
async-std = { description = "Enable async-std-backed async APIs.", category = "runtime" }

[[package.metadata.feature-manifest.groups]]
name = "runtime"
description = "Choose one async runtime backend."
members = ["tokio", "async-std"]
mutually_exclusive = true
```

Use a mutually exclusive group when enabling more than one runtime at once would
produce confusing behavior or duplicate integration surfaces.

## TLS Backends

```toml
[features]
rustls = []
native-tls = []

[package.metadata.feature-manifest.features]
rustls = { description = "Use the Rustls TLS backend.", category = "tls" }
native-tls = { description = "Use the platform-native TLS backend.", category = "tls" }

[[package.metadata.feature-manifest.groups]]
name = "tls"
description = "Choose one TLS backend."
members = ["rustls", "native-tls"]
mutually_exclusive = true
```

If a TLS backend is default-enabled, keep only that backend in `default`. The
`mutually-exclusive-default` lint catches accidental default combinations before
a release.

## `std`, `alloc`, and `no_std`

```toml
[features]
default = ["std"]
std = []
alloc = []

[package.metadata.feature-manifest.features]
std = { description = "Enable APIs that depend on the Rust standard library.", category = "platform" }
alloc = { description = "Enable APIs that require allocation but not full std.", category = "platform" }
```

If `std` is intentionally default-enabled, the default state is already clear
from the feature graph. Use `allow_default = true` only when the feature is
also private, deprecated, or unstable.

For `no_std` crates, document `alloc` separately when allocation unlocks APIs
that are still available without full `std`.

## Optional Integrations

```toml
[dependencies]
serde = { version = "1", optional = true }

[features]
serde = ["dep:serde"]

[package.metadata.feature-manifest.features]
serde = { description = "Enable Serialize and Deserialize support.", category = "serialization", docs = "https://docs.rs/serde" }
```

Using `dep:serde` lets feature-manifest verify that the dependency exists and is
marked optional.

## Experimental Or Internal Toggles

```toml
[features]
unstable = []
internal-codegen = []

[package.metadata.feature-manifest.features]
unstable = { description = "Experimental APIs; semver stability is not guaranteed.", category = "experimental", unstable = true, tracking_issue = "https://github.com/example/project/issues/123" }
internal-codegen = { description = "Internal code generation support.", category = "internal", public = false }
```

This keeps public output focused while still documenting maintainer intent.

Run `cargo fm md --include-private` when reviewing private feature docs during a
release, and omit the flag for README/docs.rs output.

## Format Families

```toml
[features]
json = []
yaml = []
toml = []

[package.metadata.feature-manifest.features]
json = { description = "Enable JSON support.", category = "format" }
yaml = { description = "Enable YAML support.", category = "format" }
toml = { description = "Enable TOML support.", category = "format" }

[[package.metadata.feature-manifest.groups]]
name = "formats"
description = "Optional format integrations."
members = ["json", "yaml", "toml"]
mutually_exclusive = false
```

Not every group is exclusive. Non-exclusive groups still help readers and JSON
consumers understand feature families.
