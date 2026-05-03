# Release Process

This repository uses one workflow for continuous integration and one workflow
for tagged releases.

## CI

`.github/workflows/ci.yml` runs on pushes and pull requests. It is expected to:

- run tests on Linux, macOS, and Windows,
- keep formatting checks centralized,
- run `cargo publish --dry-run` before releases.

## Release Workflow

`.github/workflows/release.yml` is intended to run on version tags such as
`vX.Y.Z` and on manual dispatch.

Expected repository secret:

- `CARGO_REGISTRY_TOKEN`

## Recommended Release Steps

1. Update `CHANGELOG.md`.
2. Confirm `Cargo.toml` has the intended version.
3. Regenerate the CLI reference:

```text
cargo fm help-markdown > docs/cli.md
```

4. Confirm JSON schema changes are intentional when JSON output changes.
5. Run the local publish checks:

```text
cargo fmt
cargo test --all-targets
cargo publish --dry-run
```

6. Commit the release changes.
7. Push `main`.
8. Tag the release:

```text
git tag vX.Y.Z
git push origin vX.Y.Z
```

9. Let GitHub Actions publish to crates.io and create the GitHub release.

Do not manually run `cargo publish` before pushing the release tag unless the
automated workflow is unavailable. If you do publish manually, create the
matching Git tag and GitHub release afterward, but do not rerun the tag workflow
for that already-published version.

## Manual Fallback

If automated publishing is unavailable, publish manually:

```text
cargo publish
```

Then create a GitHub release from the matching tag.

```text
gh release create vX.Y.Z --title "feature-manifest X.Y.Z" --generate-notes
```

## Post-Release Checks

- Confirm the new version appears on crates.io.
- Confirm docs.rs built the new documentation.
- Smoke-test installation:

```text
cargo install feature-manifest
```

That install now provides both `cargo-feature-manifest` and the shorter
`cargo-fm` entrypoint.
