# Architecture

`feature-manifest` puts Cargo parsing, validation, and rendering in the library.
The command-line layer stays thin and handles argument parsing plus terminal and
CI output.

## Core Modules

| Module | Responsibility |
| --- | --- |
| `discover` | Resolve manifest paths and use `cargo metadata` for workspace/package selection. |
| `model` | Define normalized feature, dependency, metadata, lint, and workspace types. |
| `parse` | Read `Cargo.toml`, parse feature metadata, preview sync rewrites, and write metadata during `sync`. |
| `validate` | Turn a normalized manifest into lint issues with configurable severities and generated lint docs. |
| `source_map` | Map validation issues back to best-effort manifest line/column spans. |
| `render` | Convert normalized data into Markdown, Mermaid, and feature explanations. |
| `json_output` | Emit the versioned metadata JSON output. |
| `docs_io` | Handle generated-file writes and marker-based README injection. |

## CLI Layer

The `src/cli` tree owns argument parsing, command dispatch, and CLI-only output
formats:

| Module | Responsibility |
| --- | --- |
| `cli::commands` | One module per command, such as `check`, `init`, `doctor`, `sync`, `schema`, and `markdown`. |
| `cli::output` | CI-oriented formats such as GitHub annotations, SARIF, and check JSON. |
| `cli::docs` | Generated CLI and lint references built from code-owned registries. |
| `cli::util` | Presentation helpers kept out of core logic. |

## Rewrite Flow

Writing commands pass through a previewable path first:

```text
preview_sync_manifest -> SyncPreview -> sync_manifest/write OR sync --diff/init --dry-run
```

This keeps `sync --diff`, `init --dry-run`, and real writes aligned. If a
preview says a file would change, the applied rewrite is expected to write the
same bytes.

## Diagnostics Flow

Validation produces structured issues without output-format assumptions. Output
layers then decide how to present them:

```text
validate -> Issue -> text/json/github/sarif
                    -> ManifestSourceMap for line/column spans
```

The source map is best-effort by design. Prefer stable, predictable locations
over complex TOML reconstruction.

## Generated Docs

Generated documentation has tests because users rely on it:

- `docs/cli.md` is generated from Clap.
- `docs/lints.md` is generated from the lint registry.
- JSON schemas are validated as JSON Schema documents.

If CLI flags, lint descriptions, or output schemas change without regenerated
docs, tests fail.

## Public API Shape

The public API re-exports stable entry points from `lib.rs`, such as
`load_workspace`, `load_manifest`, `validate`, `render_markdown`,
`render_mermaid`, `render_json`, `preview_sync_manifest`, and `sync_manifest`.

Internal module layout can change without forcing users to import from deep
paths. CLI ergonomics and implementation details should not become accidental
public API.
