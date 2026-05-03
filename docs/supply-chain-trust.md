# Supply-Chain Trust

`feature-manifest` is usually installed as a CI/dev tool, so the safest setup is
boring and explicit.

## Pin Tool Installation

Use `--locked` so CI builds the exact dependency graph from the published
`Cargo.lock`:

```text
cargo install feature-manifest --locked
```

For fully pinned CI, include a version:

```text
cargo install feature-manifest --version 0.5.0 --locked
```

## Verify Releases

Each published crates.io version should have:

- a matching Git tag, such as `v0.5.0`,
- a GitHub release with human-readable notes,
- a green CI run for the tagged commit,
- a successful `cargo publish --dry-run --locked` before publish.

## CI Recommendations

Run these checks before a release:

```text
cargo fmt --check
cargo test --all-targets
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
