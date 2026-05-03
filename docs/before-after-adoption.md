# Before and After Adoption

This guide shows the smallest useful migration from undocumented Cargo features
to checked, generated feature documentation.

## Before

A typical crate starts with a `[features]` table that is correct for Cargo but
opaque to people reading the project:

```toml
[features]
default = ["std", "serde"]
std = []
serde = ["dep:serde"]
tokio = ["dep:tokio", "std"]
rustls = ["dep:rustls"]
native-tls = ["dep:native-tls"]
unstable = []
```

Consumers can see which features exist, but they still have to infer intent:

- Is `std` safe to disable?
- Are TLS backends mutually exclusive?
- Is `unstable` part of the public API?
- Which features should appear in README/docs.rs?

## Add Metadata

Run:

```text
cargo fm init --ci
```

Then fill in the generated metadata:

```toml
[package.metadata.feature-manifest]
preset = "adopt"

[package.metadata.feature-manifest.features]
std = { description = "Enables APIs that require the Rust standard library.", category = "platform" }
serde = { description = "Enables Serialize and Deserialize implementations.", category = "integration" }
tokio = { description = "Enables async APIs backed by Tokio.", category = "runtime", requires = ["std"] }
rustls = { description = "Uses rustls for TLS.", category = "tls" }
native-tls = { description = "Uses platform-native TLS.", category = "tls" }
unstable = { description = "Enables experimental APIs; semver is not guaranteed.", unstable = true }

[[package.metadata.feature-manifest.groups]]
name = "tls"
description = "TLS backend selection."
members = ["rustls", "native-tls"]
mutually_exclusive = true
```

## After

Now the project has:

- `cargo fm` for validation.
- `cargo fm doctor` for setup health checks.
- `cargo fm md -i README.md` for generated docs.
- `cargo fm md --check -i README.md` for CI drift detection.
- `cargo fm c --format sarif` for code-scanning pipelines.
- `cargo fm j` for editor or automation experiments.

Once adoption issues are resolved, switch from gradual to strict checks:

```toml
[package.metadata.feature-manifest]
preset = "strict"
```

The result is still plain Cargo. The crate does not need runtime code changes,
and users can understand feature intent without spelunking through build flags.
