# Metadata Format

`feature-manifest` reads feature metadata from one of these tables:

- `[package.metadata.feature-manifest]`
- `[package.metadata.feature-docs]`

`feature-manifest` is the canonical name. `feature-docs` remains supported as a compatibility alias for earlier experiments and migration scenarios.

## Layouts

Two layouts are supported.

Structured:

```toml
[package.metadata.feature-manifest.features]
serde = { description = "Enable serde support." }
tokio = { description = "Enable Tokio-backed APIs." }
```

Flat:

```toml
[package.metadata.feature-manifest]
serde = { description = "Enable serde support." }
tokio = { description = "Enable Tokio-backed APIs." }
```

`sync --style structured` and `sync --style flat` can normalize a manifest to either layout.

## Feature Fields

Each feature entry accepts:

| Field | Type | Meaning |
| --- | --- | --- |
| `description` | `string` | Human-facing explanation of the feature. |
| `category` | `string` | Optional family label such as `runtime`, `tls`, or `serialization`. |
| `since` | `string` | Version or release label where the feature became available. |
| `docs` | `string` | URL to feature-specific documentation. |
| `tracking_issue` | `string` | URL to an issue tracking unstable or planned work. |
| `requires` | `string[]` | Human-facing prerequisite labels or related feature names. |
| `public` | `bool` | Whether the feature should appear in public-facing output. Defaults to `true`. |
| `unstable` | `bool` | Marks the feature as experimental. |
| `deprecated` | `bool` | Marks the feature as deprecated. |
| `allow_default` | `bool` | Acknowledges that a private, deprecated, or unstable feature is intentionally default-enabled. |
| `note` | `string` | Extra context appended in Markdown and explain output. |

Shorthand string form is allowed:

```toml
[package.metadata.feature-manifest.features]
serde = "Enable serde support."
```

That expands to:

```toml
[package.metadata.feature-manifest.features]
serde = { description = "Enable serde support." }
```

## Groups

Groups live under `[[package.metadata.feature-manifest.groups]]`:

```toml
[[package.metadata.feature-manifest.groups]]
name = "tls"
description = "Select one TLS backend."
members = ["rustls", "native-tls"]
mutually_exclusive = true
```

Fields:

| Field | Type | Meaning |
| --- | --- | --- |
| `name` | `string` | Group identifier used in reports and output. |
| `description` | `string` | Human-facing explanation of the group. |
| `members` | `string[]` | Feature names that belong to the group. |
| `mutually_exclusive` | `bool` | When `true`, flags invalid default combinations. |

## Lints

Lint overrides live under `[package.metadata.feature-manifest.lints]`:

```toml
[package.metadata.feature-manifest.lints]
missing-description = "deny"
small-group = "allow"
private-enabled-by-public = "warn"
```

Accepted levels:

- `allow`
- `warn`
- `deny`

CLI overrides use the same values:

```text
cargo fm c -l missing-description=warn
```

CLI overrides win over manifest configuration for that run.

## Presets

Use a preset when a project needs a temporary adoption policy:

```toml
[package.metadata.feature-manifest]
preset = "adopt"
```

Supported presets:

| Preset | Meaning |
| --- | --- |
| `adopt` | Downgrades common documentation drift issues to warnings while a project is onboarding. |
| `strict` | Upgrades subjective warnings such as unknown references and public-to-private activation to errors. |

CLI presets are supported too:

```text
cargo fm c --preset strict
```

## Supported Lint Codes

The generated reference in [lints.md](lints.md) is the source of truth for
lint meanings and fix guidance.

| Lint | Default | Meaning |
| --- | --- | --- |
| `missing-metadata` | `error` | Feature exists in `[features]` but has no metadata entry. |
| `missing-description` | `error` | Metadata exists but has no usable description. |
| `sensitive-default` | `error` | A private, deprecated, or unstable feature is default-enabled without `allow_default = true`. |
| `unknown-reference` | `warning` | A feature entry contains syntax that feature-manifest cannot classify. |
| `unknown-feature-reference` | `error` | A feature enables a plain name that is neither a declared feature nor an optional dependency. |
| `unknown-metadata` | `error` | Metadata exists for a feature that is not in `[features]`. |
| `unknown-default-member` | `error` | `features.default` contains a missing feature or optional dependency. |
| `unknown-default-reference` | `warning` | `features.default` contains syntax that feature-manifest cannot classify. |
| `small-group` | `warning` | A group has fewer than two members. |
| `duplicate-group-member` | `error` | A group repeats the same member more than once. |
| `unknown-group-member` | `error` | A group references a feature that does not exist. |
| `mutually-exclusive-default` | `error` | A mutually exclusive group has multiple default-enabled members. |
| `dependency-not-found` | `error` | A dependency-based feature reference points at a missing dependency. |
| `dependency-not-optional` | `error` | `dep:name` or `name?/feature` is used for a dependency that is not optional. |
| `private-enabled-by-public` | `warning` | A public feature enables a private feature. |
| `feature-cycle` | `error` | Local features form a cycle. |

## Feature Reference Syntax

`feature-manifest` models references as typed values:

| Syntax | Meaning |
| --- | --- |
| `serde` | Local feature reference |
| `dep:serde` | Optional dependency activation |
| `tokio/rt` | Dependency feature activation |
| `tokio?/rt` | Weak dependency feature activation |

## Sync Behavior

`sync` can:

- add missing metadata entries,
- remove stale entries with `--remove-stale`,
- rewrite to `flat` or `structured` layout,
- fail in CI with `--check` instead of rewriting files.

Examples:

```text
cargo fm s
cargo fm s --check
cargo fm s --remove-stale --style structured
```
