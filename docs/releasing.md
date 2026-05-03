# Release Process

This repository uses one workflow for continuous integration and one manual
workflow for release publishing.

## CI

`.github/workflows/ci.yml` runs on pushes and pull requests. It is expected to:

- run tests on Linux, macOS, and Windows,
- keep formatting checks centralized,
- run supply-chain policy checks with `cargo deny`,
- run `cargo publish --dry-run` before releases.

## Release Workflow

`.github/workflows/release.yml` is intentionally manual-only. Creating GitHub
release tags after a local publish should not trigger a second `cargo publish`
attempt for an already-published version.

Expected repository secret:

- `CARGO_REGISTRY_TOKEN`

## Recommended Release Steps

1. Update `CHANGELOG.md`.
2. Confirm `Cargo.toml` has the intended version.
3. Regenerate the CLI reference:

```text
cargo fm help-markdown > docs/cli.md
```

4. Regenerate the lint reference when lint docs or registry entries changed:

```text
cargo fm lints --markdown > docs/lints.md
```

5. Confirm JSON schema changes are intentional when JSON output changes.
6. Run the local publish checks:

```text
cargo fmt
cargo test --all-targets
cargo deny check advisories bans licenses sources
cargo publish --dry-run
```

7. Commit the release changes.
8. Push `main`.
9. Publish manually:

```text
cargo publish --locked
```

10. Create the matching Git tag and GitHub release:

```text
gh release create vX.Y.Z --target <FULL_COMMIT_SHA> --title "feature-manifest X.Y.Z" --generate-notes
```

Do not push a version tag before publishing unless you intentionally want to
manage the release manually afterward. The tag itself is release metadata; it is
not the publish trigger.

## Manual Fallback

If you want to publish through GitHub Actions instead of the local terminal, run
the manual `Release` workflow from the Actions tab after the release commit is
on `main`, and provide the intended tag such as `vX.Y.Z`. Do not also run
`cargo publish` locally for the same version.

```text
gh workflow run release.yml --ref main -f tag=vX.Y.Z
```

## Post-Release Checks

- Confirm the new version appears on crates.io.
- Confirm docs.rs built the new documentation.
- Confirm the GitHub tag, release, changelog, and crates.io version match.
- Smoke-test installation:

```text
cargo install feature-manifest
```

That install now provides both `cargo-feature-manifest` and the shorter
`cargo-fm` entrypoint.
