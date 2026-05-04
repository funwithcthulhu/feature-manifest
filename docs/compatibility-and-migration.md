# Compatibility And Migration

`feature-manifest` treats undocumented or stale metadata as a problem, but it
also provides presets and preview commands for staged adoption.

## Metadata Table Names

The preferred table is:

```toml
[package.metadata.feature-manifest]
```

The older alias is still accepted:

```toml
[package.metadata.feature-docs]
```

Use `feature-manifest` for new projects. The alias exists so early
experiments do not need an all-at-once migration.

## Flat Vs Structured Layout

Both of these are valid:

```toml
[package.metadata.feature-manifest]
serde = { description = "Enable serde support." }
```

```toml
[package.metadata.feature-manifest.features]
serde = { description = "Enable serde support." }
```

If you want one canonical shape across a repository, use:

```text
cargo fm s --diff --style structured
cargo fm s --style structured
```

or:

```text
cargo fm s --diff --style flat
cargo fm s --style flat
```

## Migrating From Hand-Maintained Tables

If your README or docs contain a manually maintained feature table:

1. Add metadata entries to `Cargo.toml`.
2. Run `cargo fm` until descriptions and visibility are in good shape.
3. Replace the manual section with injection markers.
4. Use `cargo fm md -i README.md` going forward.

## Introducing The Tool Gradually

For existing crates, a staged rollout usually looks like this:

1. Run `cargo fm s --diff`.
2. Run `cargo fm s` after reviewing the diff.
3. Fill in descriptions for public features first.
4. Mark internal features with `public = false`.
5. Use `preset = "adopt"` while onboarding.
6. Add `cargo fm` to CI.
7. Tighten lint levels over time.

Example:

```toml
[package.metadata.feature-manifest.lints]
missing-description = "warn"
private-enabled-by-public = "warn"
```

Later, after the metadata is clean:

```toml
[package.metadata.feature-manifest]
preset = "strict"

[package.metadata.feature-manifest.lints]
missing-description = "deny"
private-enabled-by-public = "deny"
```

## Schema Consumers

If you are consuming `cargo fm j` or `cargo fm c --format json`
programmatically:

- treat `schema_version` as the compatibility boundary,
- ignore unknown fields,
- avoid depending on exact human-readable messages,
- prefer stable codes such as `missing-description` and `feature-cycle`.

## Compatibility Fixtures

The `fixtures/compat` directory contains curated manifest layouts that should
continue to parse and validate:

- small crates with `std` and `serde`,
- workspace packages with inherited fields,
- TLS backend groups,
- async runtime backend groups,
- `std`/`alloc`/`no_std` surfaces.

Add a fixture when a real crate layout exposes a parser, validation, or docs
edge case. The fixture can be synthetic, but it should preserve the structure
that made the real-world layout interesting.
