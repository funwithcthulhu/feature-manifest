# Security Policy

## Supported Versions

Security fixes are applied to the latest released version of `feature-manifest`.
Because this is a developer tool rather than a runtime dependency, use:

```text
cargo install feature-manifest --locked --force
```

## Reporting a Vulnerability

Please do not file public issues for suspected vulnerabilities.

Report security concerns through GitHub private vulnerability reporting when it
is available for the repository. If that is unavailable, open a minimal public
issue asking for a private contact path without including exploit details.

Helpful report details:

- affected `feature-manifest` version,
- operating system and Rust toolchain,
- exact command and manifest shape involved,
- whether the issue affects local output only, CI behavior, generated docs, or
  published artifacts.

## Scope

In scope:

- command execution or path handling bugs in the CLI,
- malicious or malformed manifest input causing unsafe behavior,
- release, provenance, or dependency-chain issues in this repository.

Out of scope:

- feature-graph policy disagreements,
- generated documentation wording,
- vulnerabilities in downstream crates discovered through feature metadata.
