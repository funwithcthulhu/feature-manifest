# Supply-Chain Trust

`feature-manifest` is usually installed as a CI/dev tool, so the safest setup is
pinned and explicit.

## Pin Tool Installation

Use `--locked` so CI builds the exact dependency graph from the published
`Cargo.lock`:

```text
cargo install feature-manifest --locked
```

For fully pinned CI, include a version:

```text
cargo install feature-manifest --version X.Y.Z --locked
```

## Verify Releases

Each published crates.io version should have:

- a matching Git tag, such as `vX.Y.Z`,
- a GitHub release with human-readable notes,
- a green CI run for the tagged commit,
- a successful `cargo publish --dry-run --locked` before publish.

## Tag Policy

Release tags use `vX.Y.Z` and should point at the exact commit that was used to
publish the crates.io package. The tag, GitHub release, changelog entry, and
`Cargo.toml` version should all agree.

Inspect a release locally:

```text
git fetch --tags origin
git show vX.Y.Z --stat
cargo install feature-manifest --version X.Y.Z --locked --force
cargo fm --version
```

## Crate and Tag Comparison

For higher-assurance release verification, download the crate and compare it to
the matching tag:

```text
cargo download feature-manifest --version X.Y.Z
cargo package --list --allow-dirty
```

The packaged file list should match the files expected for the tag, excluding
normal Cargo packaging metadata.

## Checksums

Cargo verifies registry checksums automatically through the crates.io index.
For local release artifacts, generate checksums before attaching or sharing
files:

```text
sha256sum target/package/feature-manifest-X.Y.Z.crate
```

On Windows PowerShell:

```text
Get-FileHash target/package/feature-manifest-X.Y.Z.crate -Algorithm SHA256
```

## CI Recommendations

Run these checks before a release:

```text
cargo fmt --check
cargo test --all-targets
cargo deny check advisories bans licenses sources
cargo publish --dry-run --locked
```

In downstream projects, use:

```text
cargo fm
cargo fm md --check -i README.md
```

If generated JSON is consumed by automation, pin the schema file:

```text
cargo fm schema metadata -o feature-manifest.v1.schema.json
cargo fm schema check-report -o check-report.v1.schema.json
```

## What to Trust

The CLI is a maintainer aid. It does not participate in dependency resolution,
compile code into downstream crates, or replace Cargo's feature resolver. Treat
its output as release metadata and CI policy, not as runtime security policy.

## Dependency Automation

This repository uses Dependabot for Cargo and GitHub Actions updates. Dependency
PRs should still pass the full CI matrix, `cargo deny`, and publish dry-run
before merge.
