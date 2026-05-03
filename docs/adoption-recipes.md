# Adoption Recipes

These snippets are meant to be copied into projects that already have Cargo
features and want feature metadata checks without a long rollout.

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

## GitLab CI

```yaml
feature-manifest:
  image: rust:latest
  stage: test
  before_script:
    - cargo install feature-manifest --locked
  script:
    - cargo fm
    - cargo fm md --check -i README.md
```

## Pre-commit

```yaml
repos:
  - repo: local
    hooks:
      - id: feature-manifest
        name: feature-manifest
        entry: cargo fm
        language: system
        pass_filenames: false
      - id: feature-manifest-docs
        name: feature-manifest generated docs
        entry: cargo fm md --check -i README.md
        language: system
        pass_filenames: false
```

## Shell Completions

Generate completions for local installation scripts:

```text
cargo fm completions bash > cargo-fm.bash
cargo fm completions zsh > _cargo-fm
cargo fm completions fish > cargo-fm.fish
cargo fm completions powershell > _cargo-fm.ps1
```

## README or Docs.rs Section

Add markers where the generated feature table should appear:

```markdown
<!-- feature-manifest:start -->
<!-- feature-manifest:end -->
```

Then generate or check the section:

```text
cargo fm md -i README.md
cargo fm md --check -i README.md
```

For a standalone page:

```text
cargo fm md -o FEATURES.md
cargo fm md --check -o FEATURES.md
```

## Gradual Rollout

For a crate with many existing features, start with the adoption preset:

```toml
[package.metadata.feature-manifest]
preset = "adopt"
```

Then tighten CI after the metadata is filled in:

```text
cargo fm c --preset strict
```

Preview scaffolding before writing:

```text
cargo fm init --dry-run --ci
cargo fm s --diff
```

Once the preview looks right, rerun without `--dry-run` or `--diff`.

## Maintainer Health Check

Use doctor explanations when onboarding a repository or reviewing a release PR:

```text
cargo fm doctor --explain
cargo fm doctor --strict --explain
```

The first form is friendly for local setup; the strict form is useful when CI
should fail on missing README/CI wiring as well as validation errors.
