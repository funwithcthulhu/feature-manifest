# Changelog

All notable changes to this project will be documented in this file.

The format is loosely based on Keep a Changelog, and the project follows
Semantic Versioning where it makes sense for a Cargo tool.

## [Unreleased]

- No unreleased changes.

## [0.7.6] - 2026-05-05

- Removed maintainer release-checklist content from the public README.
- Kept release-process guidance in contributor and release documentation.

## [0.7.5] - 2026-05-04

- Reworked the README around a concrete before-and-after adoption example.
- Clarified when to use the tool, the minimum setup path, MSRV expectations,
  and pre-1.0 stability boundaries.
- Improved generated CLI and lint reference formatting for raw Markdown
  readability.
- Added missing public API documentation and a rustdoc coverage gate.

## [0.7.4] - 2026-05-04

- Clarified README and documentation wording, reduced repeated setup guidance,
  and refreshed generated lint/SARIF documentation text.

## [0.7.3] - 2026-05-04

- Adjusted default-feature validation so the new GitHub clippy gate stays clean
  on the stable toolchain used by CI.

## [0.7.2] - 2026-05-04

- Revised public-facing documentation to remove stale examples, vague wording,
  and promotional phrasing.
- Clarified the docs.rs crate introduction and public API entry points.
- Updated release and publish-readiness docs to include the clippy gate and
  locked publish dry-runs.
- Added clippy checks to CI and the manual release workflow.

## [0.7.1] - 2026-05-03

- Fixed validation for Cargo's plain optional dependency feature syntax, such as
  `default = ["serde"]` when `serde` is an optional dependency.
- Added `unknown-feature-reference` so stale plain feature references are
  reported when they point at neither a declared feature nor an optional
  dependency.
- Added direct dependency parsing for single-manifest library callers, including
  target-specific dependencies and workspace-inherited dependency declarations.
- Fixed current-toolchain Clippy warnings so `clippy -D warnings` stays clean.
- Aligned direct TOML parser dependencies with `cargo_metadata` to reduce
  duplicate dependency versions.

## [0.7.0] - 2026-05-03

- Released `0.7.0` to crates.io.
- Added `init --dry-run` and `sync --diff` for safer rewrite previews.
- Added `doctor --explain` with concrete next actions for setup findings.
- Added generated lint reference documentation backed by the lint registry.
- Added source-map-backed line and column diagnostics for GitHub Actions and
  SARIF output.
- Added `cargo deny` policy, Dependabot configuration, and CI supply-chain
  checks.
- Added security, contributing, code-of-conduct, and support documentation.
- Added curated compatibility fixtures for small-crate, workspace, TLS,
  runtime, and no_std-style manifest layouts.
- Expanded trust, adoption, compatibility, architecture, and release
  documentation.

## [0.6.0] - 2026-05-03

- Released `0.6.0` to crates.io.
- Added `schema` for printing or writing bundled JSON Schema files.
- Added `completions` for Bash, Zsh, Fish, PowerShell, and Elvish shell
  completions.
- Added `doctor --strict` so setup warnings can fail CI.
- Added line-aware GitHub Actions annotations when feature, metadata, or group
  lines can be located in `Cargo.toml`.
- Added schema validation tests for both JSON output surfaces.
- Added more property coverage for stale removal, layout conversion, lint
  preservation, and quoted feature keys.
- Added supply-chain trust documentation and expanded release/tag guidance.

## [0.5.0] - 2026-05-03

- Released `0.5.0` to crates.io.
- Added generated CLI reference documentation backed by Clap command
  definitions.
- Added versioned JSON Schema files for metadata and check-report JSON output.
- Added property tests for sync idempotence, marker injection, and line-ending
  tolerant generated-doc checks.
- Split renderers into focused Markdown, Mermaid, explain, and shared helper
  modules.
- Added adoption recipes, a before/after migration guide, and architecture docs.
- Updated generated GitHub Actions workflows plus project CI/release workflows
  to `actions/checkout@v6`.

## [0.4.0] - 2026-05-03

- Released `0.4.0` to crates.io.
- Added `init` for first-time setup of metadata, README markers, and optional
  GitHub Actions wiring.
- Added `doctor` for project wiring, generated docs, CI, install-shape, and
  validation health checks.
- Added `md --check` for stale generated Markdown and README injection checks.
- Added lint presets via `preset = "adopt"` or `preset = "strict"`, plus
  `check --preset`.
- Added richer feature metadata fields: `category`, `since`, `docs`,
  `tracking_issue`, and `requires`.
- Split CLI command and output handling into focused internal modules.
- Added edge and messy fixtures, additional reliability tests, and golden
  snapshots for JSON, GitHub Actions, and SARIF check output.
- Added getting-started docs, CI docs, examples, badges, and issue templates.

## [0.3.0] - 2026-05-03

- Added a shorter `cargo fm` entrypoint plus command aliases and short flags for
  the most common workflows.

## [0.2.0] - 2026-05-02

- Added workspace-aware package discovery built on `cargo metadata`.
- Added typed feature-reference modeling for local features, `dep:` references,
  dependency features, and weak dependency features.
- Added `sync` to scaffold missing metadata entries directly into manifests.
- Added `explain` for focused single-feature summaries.
- Added a versioned JSON schema instead of serializing internal structs
  directly.
- Added default feature-set summaries and output escaping for Markdown and
  Mermaid renderers.
- Split the crate into focused `discover`, `parse`, `model`, `render`,
  `validate`, and `json_output` layers.
- Added valid Cargo fixture crates, integration tests, and snapshot tests.
- Added Markdown file writing and marker-based README injection.
- Added configurable lint overrides in both manifest metadata and CLI flags.
- Added `sync --check`, `sync --remove-stale`, and layout normalization flags.
- Added `check` output modes for JSON, GitHub Actions annotations, and SARIF.
- Added validation for missing optional dependencies, public-to-private feature
  activation, and local feature cycles.
- Added companion documentation for metadata format, JSON output, migration,
  workflows, release process, and 1.0 planning.
- Expanded CI to cover Linux, macOS, and Windows, and added a tagged release
  workflow for crates.io publishing plus GitHub releases.

## [0.1.0] - 2026-05-02

- Initial release scaffold for the `feature-manifest` crate and
  `cargo-feature-manifest` binary.
- Added manifest parsing for `[features]` plus
  `[package.metadata.feature-manifest]` and `[package.metadata.feature-docs]`.
- Added `check`, `markdown`, `json`, and `graph` commands.
- Added validation for missing or stale metadata, missing descriptions,
  sensitive default-enabled features, and mutually exclusive groups.
- Added Markdown and Mermaid renderers, a fixture crate, and CI coverage.
