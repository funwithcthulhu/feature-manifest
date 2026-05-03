# Architecture

`feature-manifest` keeps the Cargo-facing logic in the library and the command
line surface in a thin CLI layer.

## Core Modules

| Module | Responsibility |
| --- | --- |
| `discover` | Resolve manifest paths and use `cargo metadata` for workspace/package selection. |
| `model` | Define normalized feature, dependency, metadata, lint, and workspace types. |
| `parse` | Read `Cargo.toml`, parse feature metadata, preview sync rewrites, and write metadata during `sync`. |
| `validate` | Turn a normalized manifest into lint issues with configurable severities and generated lint docs. |
| `source_map` | Map validation issues back to best-effort manifest line/column spans. |
| `render` | Convert normalized data into Markdown, Mermaid, and focused explanations. |
| `json_output` | Emit the versioned metadata JSON surface. |
| `docs_io` | Handle generated-file writes and marker-based README injection. |

## CLI Layer

The `src/cli` tree owns argument parsing, command dispatch, and CLI-only output
formats:

| Module | Responsibility |
| --- | --- |
| `cli::commands` | One module per command, such as `check`, `init`, `doctor`, `sync`, `schema`, and `markdown`. |
| `cli::output` | CI-oriented formats such as GitHub annotations, SARIF, and check JSON. |
| `cli::docs` | Generated CLI and lint references built from code-owned registries. |
| `cli::util` | Small presentation helpers that should not leak into core logic. |

## Rewrite Flow

All writing commands should pass through a previewable path first:

```text
preview_sync_manifest -> SyncPreview -> sync_manifest/write OR sync --diff/init --dry-run
```

This keeps `sync --diff`, `init --dry-run`, and real writes aligned. If the
preview says a file would change, the applied rewrite should be byte-for-byte
the same content.

## Diagnostics Flow

Validation produces structured issues without output-format assumptions. Output
layers then decide how to present them:

```text
validate -> Issue -> text/json/github/sarif
                    -> ManifestSourceMap for line/column spans
```

The source map is best-effort by design. It should prefer stable, predictable
locations over clever TOML reconstruction.

## Generated Docs

Generated documentation has tests because it is part of the adoption surface:

- `docs/cli.md` is generated from Clap.
- `docs/lints.md` is generated from the lint registry.
- JSON schemas are validated as JSON Schema documents.

If a developer changes CLI flags, lint descriptions, or output schemas without
regenerating docs, tests should fail.

## Public API Shape

The public API re-exports stable entry points from `lib.rs`, such as
`load_workspace`, `load_manifest`, `validate`, `render_markdown`,
`render_mermaid`, `render_json`, `preview_sync_manifest`, and `sync_manifest`.

Internal module layout can keep improving without forcing users to import from
deep paths. That is intentional: CLI polish and implementation structure should
not become accidental public API.
