# CI Setup

`feature-manifest` works well as a normal shell command in CI.

## GitHub Actions

```yaml
name: Feature Manifest

on:
  push:
    branches:
      - main
  pull_request:

jobs:
  feature-manifest:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v6
      - uses: dtolnay/rust-toolchain@stable
      - name: Install feature-manifest
        run: cargo install feature-manifest --locked
      - name: Check feature metadata
        run: cargo fm
      - name: Check generated README section
        run: cargo fm md --check -i README.md
```

This uses `actions/checkout@v6` so new projects avoid the GitHub Actions Node
20 deprecation warning.

For GitHub annotations:

```text
cargo fm c -f github
```

Annotations include `Cargo.toml` line numbers when the related feature,
metadata entry, or group can be located.

For SARIF:

```text
cargo fm c -f sarif > feature-manifest.sarif
```

## GitLab CI

```yaml
feature_manifest:
  image: rust:latest
  script:
    - cargo install feature-manifest --locked
    - cargo fm
    - cargo fm md --check -i README.md
```

## Generic Shell

```text
cargo install feature-manifest --locked
cargo fm
cargo fm md --check -i README.md
```

Use `cargo fm doctor --strict` when you want project wiring warnings to fail CI
alongside validation errors.

## Safer Rewrite Checks

Use preview modes in review workflows so maintainers can see exactly what a
rewrite would do:

```text
cargo fm init --dry-run --ci
cargo fm s --diff --remove-stale --style structured
```

`sync --diff` exits non-zero when drift exists, which makes it useful as a CI
guard as well as a local preview.

## Supply-Chain Checks

This repository runs `cargo deny` in CI for advisories, license policy, duplicate
dependency warnings, and unknown sources:

```text
cargo deny check advisories bans licenses sources
```

Downstream projects that pin `feature-manifest` as a release tool can copy the
same pattern when they want dependency and license policy in the same CI surface
as feature metadata checks.
