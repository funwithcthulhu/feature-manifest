# Architecture

`feature-manifest` keeps the Cargo-facing logic in the library and the command
line surface in a thin CLI layer.

## Core Modules

| Module | Responsibility |
| --- | --- |
| `discover` | Resolve manifest paths and use `cargo metadata` for workspace/package selection. |
| `model` | Define normalized feature, dependency, metadata, lint, and workspace types. |
| `parse` | Read `Cargo.toml`, parse feature metadata, and rewrite metadata during `sync`. |
| `validate` | Turn a normalized manifest into lint issues with configurable severities. |
| `render` | Convert normalized data into Markdown, Mermaid, and focused explanations. |
| `json_output` | Emit the versioned metadata JSON surface. |
| `docs_io` | Handle generated-file writes and marker-based README injection. |

## CLI Layer

The `src/cli` tree owns argument parsing, command dispatch, and CLI-only output
formats:

| Module | Responsibility |
| --- | --- |
| `cli::commands` | One module per command, such as `check`, `init`, `doctor`, `sync`, and `markdown`. |
| `cli::output` | CI-oriented formats such as GitHub annotations, SARIF, and check JSON. |
| `cli::docs` | Generated CLI reference built from the Clap command definitions. |
| `cli::util` | Small presentation helpers that should not leak into core logic. |

## Public API Shape

The public API re-exports stable entry points from `lib.rs`, such as
`load_workspace`, `load_manifest`, `validate`, `render_markdown`,
`render_mermaid`, `render_json`, and `sync_manifest`.

Internal module layout can keep improving without forcing users to import from
deep paths. That is intentional: CLI polish and implementation structure should
not become accidental public API.
