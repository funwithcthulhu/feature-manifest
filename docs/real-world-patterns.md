# Real-World Patterns

These examples show how maintainers often map common Rust feature styles into
feature-manifest metadata.

## Runtime Backends

```toml
[features]
tokio = ["dep:tokio"]
async-std = ["dep:async-std"]

[package.metadata.feature-manifest.features]
tokio = { description = "Enable Tokio-backed async APIs." }
async-std = { description = "Enable async-std-backed async APIs." }

[[package.metadata.feature-manifest.groups]]
name = "runtime"
description = "Choose one async runtime backend."
members = ["tokio", "async-std"]
mutually_exclusive = true
```

## TLS Backends

```toml
[features]
rustls = []
native-tls = []

[package.metadata.feature-manifest.features]
rustls = { description = "Use the Rustls TLS backend." }
native-tls = { description = "Use the platform-native TLS backend." }

[[package.metadata.feature-manifest.groups]]
name = "tls"
description = "Choose one TLS backend."
members = ["rustls", "native-tls"]
mutually_exclusive = true
```

## `std`, `alloc`, and `no_std`

```toml
[features]
default = ["std"]
std = []
alloc = []

[package.metadata.feature-manifest.features]
std = { description = "Enable APIs that depend on the Rust standard library." }
alloc = { description = "Enable APIs that require allocation but not full std." }
```

If `std` is intentionally default-enabled, the default state is already clear
from the feature graph. Use `allow_default = true` only when the feature is
also private, deprecated, or unstable.

## Optional Integrations

```toml
[dependencies]
serde = { version = "1", optional = true }

[features]
serde = ["dep:serde"]

[package.metadata.feature-manifest.features]
serde = { description = "Enable Serialize and Deserialize support." }
```

Using `dep:serde` lets feature-manifest verify that the dependency exists and is
marked optional.

## Experimental Or Internal Toggles

```toml
[features]
unstable = []
internal-codegen = []

[package.metadata.feature-manifest.features]
unstable = { description = "Experimental APIs; semver stability is not guaranteed.", unstable = true }
internal-codegen = { description = "Internal code generation support.", public = false }
```

This keeps public output focused while still documenting maintainer intent.

## Format Families

```toml
[features]
json = []
yaml = []
toml = []

[package.metadata.feature-manifest.features]
json = { description = "Enable JSON support." }
yaml = { description = "Enable YAML support." }
toml = { description = "Enable TOML support." }

[[package.metadata.feature-manifest.groups]]
name = "formats"
description = "Optional format integrations."
members = ["json", "yaml", "toml"]
mutually_exclusive = false
```

Not every group is exclusive. Non-exclusive groups still help readers and JSON
consumers understand feature families.
