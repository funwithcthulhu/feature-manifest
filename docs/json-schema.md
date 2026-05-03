# JSON Schema

`feature-manifest` currently exposes two machine-readable JSON surfaces:

- `cargo fm j`
- `cargo fm c --format json`

Both are versioned independently through a top-level `schema_version` field. The
current schema version for both outputs is `1`.

The long-form commands still work; the examples here use the preferred shorthand.

## `cargo fm j`

This command emits normalized package metadata for the selected crate or
workspace.

Top-level shape:

```json
{
  "schema_version": 1,
  "packages": []
}
```

Package shape:

| Field | Type | Meaning |
| --- | --- | --- |
| `package_name` | `string \| null` | Cargo package name when known. |
| `manifest_path` | `string` | Portable path relative to the selected root manifest. |
| `metadata_table` | `string \| null` | The metadata table that was used, such as `package.metadata.feature-manifest`. |
| `metadata_layout` | `"flat" \| "structured"` | How feature metadata is stored in the manifest. |
| `default_feature_set` | `string[]` | Raw entries from `features.default`. |
| `dependencies` | `Dependency[]` | Known dependencies discovered from Cargo metadata. |
| `lint_overrides` | `LintOverride[]` | Manifest-defined lint overrides. |
| `features` | `Feature[]` | Normalized feature entries. |
| `groups` | `Group[]` | Feature group definitions. |

Dependency shape:

| Field | Type | Meaning |
| --- | --- | --- |
| `key` | `string` | Dependency key used in `Cargo.toml`. |
| `package` | `string` | Underlying package name. |
| `optional` | `bool` | Whether Cargo marks the dependency as optional. |

Lint override shape:

| Field | Type | Meaning |
| --- | --- | --- |
| `code` | `string` | Lint code, such as `missing-description`. |
| `level` | `"allow" \| "warn" \| "deny"` | Effective override level from the manifest. |

Feature shape:

| Field | Type | Meaning |
| --- | --- | --- |
| `name` | `string` | Feature name. |
| `default_enabled` | `bool` | Whether the feature is enabled by the `default` feature set. |
| `has_metadata` | `bool` | Whether the feature has metadata in the selected table. |
| `enables` | `FeatureRef[]` | Typed references activated by this feature. |
| `groups` | `string[]` | Group names that include this feature. |
| `metadata` | `FeatureMetadata` | Normalized metadata payload. |

Feature reference shape:

```json
{ "kind": "feature", "name": "serde" }
{ "kind": "dependency", "name": "serde" }
{ "kind": "dependency_feature", "dependency": "tokio", "feature": "rt", "weak": true }
{ "kind": "unknown", "raw": "target-conditional-syntax" }
```

Feature metadata shape:

| Field | Type | Meaning |
| --- | --- | --- |
| `description` | `string \| null` | Human-facing description. |
| `public` | `bool` | Whether the feature is meant for public-facing output. |
| `unstable` | `bool` | Whether the feature is experimental. |
| `deprecated` | `bool` | Whether the feature is deprecated. |
| `allow_default` | `bool` | Whether sensitive default enablement is explicitly acknowledged. |
| `note` | `string \| null` | Extra maintainer-facing context. |

Group shape:

| Field | Type | Meaning |
| --- | --- | --- |
| `name` | `string` | Group identifier. |
| `description` | `string \| null` | Human-facing description. |
| `mutually_exclusive` | `bool` | Whether multiple default-enabled members are invalid. |
| `members` | `string[]` | Feature names in the group. |

## `cargo fm c --format json`

This command emits a validation report plus a summary.

Top-level shape:

```json
{
  "schema_version": 1,
  "packages": [],
  "summary": {}
}
```

Per-package report:

| Field | Type | Meaning |
| --- | --- | --- |
| `package_name` | `string \| null` | Cargo package name when known. |
| `manifest_path` | `string` | Portable path relative to the selected root manifest. |
| `errors` | `number` | Number of error-level issues after lint overrides. |
| `warnings` | `number` | Number of warning-level issues after lint overrides. |
| `issues` | `Issue[]` | Validation issues for the package. |

Issue shape:

| Field | Type | Meaning |
| --- | --- | --- |
| `code` | `string` | Stable lint code. |
| `severity` | `"error" \| "warning"` | Effective severity after overrides. |
| `feature` | `string \| null` | Associated feature name when applicable. |
| `message` | `string` | Human-readable explanation. |

Summary shape:

| Field | Type | Meaning |
| --- | --- | --- |
| `packages` | `number` | Number of packages in the report. |
| `features` | `number` | Total number of feature definitions considered. |
| `groups` | `number` | Total number of groups considered. |
| `errors` | `number` | Total error count. |
| `warnings` | `number` | Total warning count. |

## Stability Notes

- Field names are intended to be stable within a given `schema_version`.
- New fields may be added in future versions.
- Consumers should ignore unknown fields and prefer feature detection over
  positional assumptions.
- Human-readable text like `message` may improve over time even when the schema
  version stays the same.
