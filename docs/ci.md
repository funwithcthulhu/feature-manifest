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
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - name: Install feature-manifest
        run: cargo install feature-manifest --locked
      - name: Check feature metadata
        run: cargo fm
      - name: Check generated README section
        run: cargo fm md --check -i README.md
```

For GitHub annotations:

```text
cargo fm c -f github
```

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
