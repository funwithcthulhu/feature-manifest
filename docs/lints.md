# Lint Reference

Generated from the feature-manifest lint registry. Update this file with `cargo fm lints --markdown > docs/lints.md`.

| Code | Default | Meaning | Fix |
| --- | --- | --- | --- |
| `missing-metadata` | `error` | Feature exists in `[features]` but has no metadata entry. | Add an entry under `[package.metadata.feature-manifest.features]`, or run `cargo fm sync` to scaffold TODO descriptions. |
| `missing-description` | `error` | Metadata exists but has no usable description. | Fill in a human-facing `description` so generated docs explain why the feature exists. |
| `sensitive-default` | `error` | A private, deprecated, or unstable feature is default-enabled without acknowledgement. | Remove the feature from `default`, or set `allow_default = true` when the default is intentional. |
| `unknown-reference` | `warning` | A feature entry contains syntax that feature-manifest cannot classify. | Prefer local features, `dep:name`, `name/feature`, or `name?/feature` so tooling can reason about the reference. |
| `unknown-feature-reference` | `error` | A feature enables a plain name that is neither a declared feature nor an optional dependency. | Add the missing feature, make the dependency optional, switch to `dep:name`, or remove the stale reference. |
| `unknown-metadata` | `error` | Metadata exists for a feature that is not declared in `[features]`. | Delete the stale metadata, re-add the feature, or run `cargo fm sync --remove-stale`. |
| `unknown-default-member` | `error` | `features.default` contains a missing default member. | Remove the missing default member, add the feature to `[features]`, or make the referenced dependency optional. |
| `unknown-default-reference` | `warning` | `features.default` contains syntax that feature-manifest cannot classify. | Keep the default set to local feature names when possible so generated summaries stay precise. |
| `small-group` | `warning` | A group has fewer than two members. | Add at least one more member or remove the group until there is a meaningful feature family to document. |
| `duplicate-group-member` | `error` | A group repeats the same member more than once. | Deduplicate the `members` array for the group. |
| `unknown-group-member` | `error` | A group references a feature that does not exist. | Remove the missing group member or add the feature to `[features]`. |
| `mutually-exclusive-default` | `error` | A mutually exclusive group has multiple default-enabled members. | Keep at most one member of the group in the default feature set. |
| `dependency-not-found` | `error` | A dependency-based feature reference points at a missing dependency. | Fix the dependency key, add the dependency, or remove the stale `dep:`/dependency-feature reference. |
| `dependency-not-optional` | `error` | `dep:name` or `name?/feature` is used for a dependency that is not optional. | Mark the dependency `optional = true`, or use a plain dependency feature reference when the dependency is always enabled. |
| `private-enabled-by-public` | `warning` | A public feature enables a private feature. | Make the dependency feature public too, or document why the public feature intentionally exposes that internal toggle. |
| `feature-cycle` | `error` | Local features form a cycle. | Break the cycle by removing one local feature reference from the loop. |
