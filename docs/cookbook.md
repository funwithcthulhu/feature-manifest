# Cookbook

This page collects short, copy-pasteable workflows for common maintainer tasks.

## Single Crate, First-Time Setup

1. Add or update features in `Cargo.toml`.
2. Scaffold metadata entries:

```text
cargo feature-manifest sync
```

3. Fill in descriptions and visibility flags.
4. Validate the result:

```text
cargo feature-manifest check
```

5. Generate Markdown for docs:

```text
cargo feature-manifest markdown --write FEATURES.md
```

## Keep a README Section Up to Date

Add markers to `README.md`:

```markdown
<!-- feature-manifest:start -->
<!-- feature-manifest:end -->
```

Then update that section whenever features change:

```text
cargo feature-manifest markdown --insert-into README.md
```

Use custom markers when your repository already reserves a different region:

```text
cargo feature-manifest markdown --insert-into README.md --start-marker "<!-- features:start -->" --end-marker "<!-- features:end -->"
```

## Fail CI When Metadata Drifts

Use `sync --check` to verify that every feature has metadata and that the
selected layout is already normalized:

```text
cargo feature-manifest sync --check --style structured
```

Use `check` to enforce quality rules:

```text
cargo feature-manifest check
```

## Generate Tooling-Friendly Validation Output

For local scripts or editor tooling:

```text
cargo feature-manifest check --format json
```

For GitHub Actions annotations:

```text
cargo feature-manifest check --format github
```

For code-scanning pipelines:

```text
cargo feature-manifest check --format sarif > feature-manifest.sarif
```

## Audit a Workspace

Validate every selected member:

```text
cargo feature-manifest --workspace check --manifest-path Cargo.toml
```

Inspect one package in more detail:

```text
cargo feature-manifest --package my-crate explain serde --manifest-path Cargo.toml
```

## Trim Stale Metadata

Remove entries for deleted features and rewrite into a consistent layout:

```text
cargo feature-manifest sync --remove-stale --style structured
```

Preview whether anything would change without editing the file:

```text
cargo feature-manifest sync --check --remove-stale --style structured
```

## Soften or Tighten a Lint

Persist the rule in `Cargo.toml`:

```toml
[package.metadata.feature-manifest.lints]
missing-description = "warn"
private-enabled-by-public = "deny"
```

Or override one run from CI:

```text
cargo feature-manifest check --lint private-enabled-by-public=deny
```
