# Changelog

All notable changes to this project will be documented in this file.

The format is loosely based on Keep a Changelog, and the project follows
Semantic Versioning where it makes sense for a Cargo tool.

## [Unreleased]

- Nothing yet.

## [0.1.0] - 2026-05-02

- Initial release scaffold for the `feature-manifest` crate and
  `cargo-feature-manifest` binary.
- Added manifest parsing for `[features]` plus
  `[package.metadata.feature-manifest]` and `[package.metadata.feature-docs]`.
- Added `check`, `markdown`, `json`, and `graph` commands.
- Added validation for missing or stale metadata, missing descriptions,
  sensitive default-enabled features, and mutually exclusive groups.
- Added Markdown and Mermaid renderers, a fixture crate, and CI coverage.
