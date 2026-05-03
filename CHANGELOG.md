# Changelog

All notable changes to this project will be documented in this file.

The format is loosely based on Keep a Changelog, and the project follows
Semantic Versioning where it makes sense for a Cargo tool.

## [Unreleased]

- Bumped the crate version to `0.2.0` for the next unpublished iteration.
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

## [0.1.0] - 2026-05-02

- Initial release scaffold for the `feature-manifest` crate and
  `cargo-feature-manifest` binary.
- Added manifest parsing for `[features]` plus
  `[package.metadata.feature-manifest]` and `[package.metadata.feature-docs]`.
- Added `check`, `markdown`, `json`, and `graph` commands.
- Added validation for missing or stale metadata, missing descriptions,
  sensitive default-enabled features, and mutually exclusive groups.
- Added Markdown and Mermaid renderers, a fixture crate, and CI coverage.
