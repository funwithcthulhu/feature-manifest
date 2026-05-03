# Compatibility And Migration

`feature-manifest` is designed to be strict about feature quality while still
being gentle about adoption.

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
cargo feature-manifest sync --style structured
```

or:

```text
cargo feature-manifest sync --style flat
```

## Migrating From Hand-Maintained Tables

If your README or docs contain a manually maintained feature table:

1. Add metadata entries to `Cargo.toml`.
2. Run `cargo feature-manifest check` until descriptions and visibility are in good shape.
3. Replace the manual section with injection markers.
4. Use `markdown --insert-into` going forward.

## Introducing The Tool Gradually

A low-friction adoption path usually looks like this:

1. Run `cargo feature-manifest sync`.
2. Fill in descriptions for public features first.
3. Mark internal features with `public = false`.
4. Add `cargo feature-manifest check` to CI.
5. Tighten lint levels over time.

Example:

```toml
[package.metadata.feature-manifest.lints]
missing-description = "warn"
private-enabled-by-public = "warn"
```

Later, after the metadata is clean:

```toml
[package.metadata.feature-manifest.lints]
missing-description = "deny"
private-enabled-by-public = "deny"
```

## Schema Consumers

If you are consuming `cargo feature-manifest json` or
`cargo feature-manifest check --format json` programmatically:

- treat `schema_version` as the compatibility boundary,
- ignore unknown fields,
- avoid depending on exact human-readable messages,
- prefer stable codes such as `missing-description` and `feature-cycle`.
