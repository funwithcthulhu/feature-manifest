# Lint Reference

Generated from the feature-manifest lint registry.

Update this file with:

```text
cargo fm lints --markdown > docs/lints.md
```

## `missing-metadata`

Default: `error`

Feature exists in `[features]` but has no metadata entry.

Fix: Add an entry under `[package.metadata.feature-manifest.features]`, or run
`cargo fm sync` to scaffold TODO descriptions.

## `missing-description`

Default: `error`

Metadata exists but has no usable description.

Fix: Fill in `description` with text that explains why the feature exists.

## `sensitive-default`

Default: `error`

A private, deprecated, or unstable feature is default-enabled without acknowledgement.

Fix: Remove the feature from `default`, or set `allow_default = true` when the default
is intentional.

## `unknown-reference`

Default: `warning`

A feature entry contains syntax that feature-manifest cannot classify.

Fix: Use local features, `dep:name`, `name/feature`, or `name?/feature` when possible.

## `unknown-feature-reference`

Default: `error`

A feature enables a plain name that is neither a declared feature nor an optional
dependency.

Fix: Add the missing feature, make the dependency optional, switch to `dep:name`, or
remove the stale reference.

## `unknown-metadata`

Default: `error`

Metadata exists for a feature that is not declared in `[features]`.

Fix: Delete the stale metadata, re-add the feature, or run
`cargo fm sync --remove-stale`.

## `unknown-default-member`

Default: `error`

`features.default` contains a missing default member.

Fix: Remove the missing default member, add the feature to `[features]`, or make the
referenced dependency optional.

## `unknown-default-reference`

Default: `warning`

`features.default` contains syntax that feature-manifest cannot classify.

Fix: Use local feature names in `default` when possible.

## `small-group`

Default: `warning`

A group has fewer than two members.

Fix: Add at least one more member or remove the group.

## `duplicate-group-member`

Default: `error`

A group repeats the same member more than once.

Fix: Deduplicate the `members` array for the group.

## `unknown-group-member`

Default: `error`

A group references a feature that does not exist.

Fix: Remove the missing group member or add the feature to `[features]`.

## `mutually-exclusive-default`

Default: `error`

A mutually exclusive group has multiple default-enabled members.

Fix: Keep at most one member of the group in the default feature set.

## `dependency-not-found`

Default: `error`

A dependency-based feature reference points at a missing dependency.

Fix: Fix the dependency key, add the dependency, or remove the stale
`dep:`/dependency-feature reference.

## `dependency-not-optional`

Default: `error`

`dep:name` or `name?/feature` is used for a dependency that is not optional.

Fix: Mark the dependency `optional = true`, or use a plain dependency feature reference
when the dependency is always enabled.

## `private-enabled-by-public`

Default: `warning`

A public feature enables a private feature.

Fix: Make the dependency feature public too, or document why the public feature
intentionally exposes that internal toggle.

## `feature-cycle`

Default: `error`

Local features form a cycle.

Fix: Break the cycle by removing one local feature reference from the loop.
