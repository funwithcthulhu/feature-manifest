# Dogfooding

This repository runs `cargo fm` against itself, but the root crate currently has
no Cargo features:

```toml
[package]
name = "feature-manifest"
```

Because there is no `[features]` table, there are no real public features to
describe under `[package.metadata.feature-manifest]`. Adding feature metadata to
the root `Cargo.toml` would be misleading until the crate grows real Cargo
features.

The self-check is still useful because it verifies the installed subcommand,
workspace discovery for this repository, and README marker handling:

```text
cargo fm check
```

Expected result for the current root crate:

```text
validated 0 feature(s) and 0 group(s): 0 error(s), 0 warning(s)
```

The README generated section is checked with:

```text
cargo fm markdown --check --insert-into README.md
```

Expected result when the checked-in README is current:

```text
`README.md` injected region is up to date
```

Richer feature layouts are covered by fixtures instead of by invented root-crate
features:

- `fixtures/realistic` covers `dep:` optional dependency references,
  `crate/feature` references, a default feature set, a feature group, a private
  feature, stale metadata, and missing metadata.
- `fixtures/workspace-selection` covers explicit package selection in a
  workspace and Markdown output for the selected package.
- `fixtures/messy` covers lint failures and source locations used by GitHub
  Actions and SARIF output tests.

Those fixtures are exercised by `tests/cli.rs`, `tests/compatibility.rs`, and
`tests/snapshots.rs`.
