# CLI Reference

Generated from the Clap command definitions.

Update this file with:

```text
cargo fm help-markdown > docs/cli.md
```

## `cargo fm`

```text
Document, validate, and render Cargo feature metadata.

Usage: cargo fm [OPTIONS] [COMMAND]

Commands:
  init         Set up feature-manifest metadata, README markers, and optional CI
  doctor       Check project wiring, generated docs, CI config, and metadata health
  check        Validate feature metadata and CI-oriented rules [aliases: c, chk]
  markdown     Render a Markdown feature table [aliases: md, m]
  json         Emit normalized machine-readable feature metadata as JSON [aliases: j]
  graph        Render a Mermaid graph of feature relationships [aliases: g, viz]
  sync         Scaffold missing metadata entries directly into Cargo.toml [aliases: s]
  explain      Explain one feature in human-readable form [aliases: show, x]
  list-lints   List the lint codes supported by `check` [aliases: lints]
  schema       Emit bundled JSON Schema documents [aliases: schemas]
  completions  Generate shell completions for cargo-fm
  help         Print this message or the help of the given subcommand(s)

Options:
  -m, --manifest-path <PATH>
          Path to Cargo.toml or a crate directory.

  -w, --workspace
          Operate on every workspace member.

  -p, --package <NAME>
          Select a specific package within a workspace.

  -h, --help
          Print help

  -V, --version
          Print version

Examples:
  cargo fm check
  cargo fm init --dry-run
  cargo fm doctor --explain
  cargo fm check --format sarif
  cargo fm --workspace check --lint missing-description=warn
  cargo fm markdown --write FEATURES.md
  cargo fm markdown --check --insert-into README.md
  cargo fm sync --diff --remove-stale --style structured
  cargo fm --package cli explain serde

Short aliases such as `c`, `md`, `s`, and `show` are supported.
The original `cargo feature-manifest ...` command remains supported.
```

## `cargo fm init`

```text
Set up feature-manifest metadata, README markers, and optional CI

Usage: cargo fm init [OPTIONS]

Options:
      --dry-run
          Print setup actions without writing files.

      --readme <PATH>
          README path to create or update.

      --no-readme
          Skip README marker setup.

      --ci
          Add a GitHub Actions workflow.

  -s, --style <STYLE>
          Choose the metadata layout to write.
          
          [possible values: flat, structured]

  -h, --help
          Print help
```

## `cargo fm doctor`

```text
Check project wiring, generated docs, CI config, and metadata health

Usage: cargo fm doctor [OPTIONS]

Options:
      --readme <PATH>
          README path to inspect.

      --strict
          Exit non-zero when doctor reports warnings as well as errors.

      --explain
          Print suggested next actions for each doctor finding.

  -h, --help
          Print help
```

## `cargo fm check`

```text
Validate feature metadata and CI-oriented rules

Usage: cargo fm check [OPTIONS]

Options:
  -f, --format <FORMAT>
          [default: text]
          [possible values: text, json, github, sarif]

  -l, --lint <CODE=LEVEL>
          Override one lint level for this run.

      --preset <PRESET>
          Apply a lint preset for this run.
          
          [possible values: adopt, strict]

  -h, --help
          Print help
```

## `cargo fm markdown`

```text
Render a Markdown feature table

Usage: cargo fm markdown [OPTIONS]

Options:
  -a, --include-private
          Include private/internal features in the output.

  -o, --write <PATH>
          Write Markdown to a file.

  -i, --insert-into <PATH>
          Inject Markdown between markers in an existing file.

  -c, --check
          Exit non-zero when generated Markdown is stale.

      --start-marker <TEXT>
          Start marker used by `--insert-into`.
          
          [default: "<!-- feature-manifest:start -->"]

      --end-marker <TEXT>
          End marker used by `--insert-into`.
          
          [default: "<!-- feature-manifest:end -->"]

  -h, --help
          Print help
```

## `cargo fm json`

```text
Emit normalized machine-readable feature metadata as JSON

Usage: cargo fm json

Options:
  -h, --help
          Print help
```

## `cargo fm graph`

```text
Render a Mermaid graph of feature relationships

Usage: cargo fm graph [OPTIONS]

Options:
  -a, --include-private
          Include private/internal features in the output.

  -h, --help
          Print help
```

## `cargo fm sync`

```text
Scaffold missing metadata entries directly into Cargo.toml

Usage: cargo fm sync [OPTIONS]

Options:
  -c, --check
          Exit non-zero if changes would be needed, without rewriting files.

  -r, --remove-stale
          Remove stale metadata entries for missing features.

  -s, --style <STYLE>
          Choose the metadata layout to write back.
          
          [possible values: flat, structured]

      --diff
          Print a unified diff of the rewrite without changing files.

  -h, --help
          Print help
```

## `cargo fm explain`

```text
Explain one feature in human-readable form

Usage: cargo fm explain [OPTIONS] <FEATURE>

Arguments:
  <FEATURE>
          

Options:
  -a, --include-private
          Include private/internal features when searching for matches.

  -h, --help
          Print help
```

## `cargo fm list-lints`

```text
List the lint codes supported by `check`

Usage: cargo fm list-lints [OPTIONS]

Options:
      --markdown
          Render the generated lint reference as Markdown.

  -h, --help
          Print help
```

## `cargo fm schema`

```text
Emit bundled JSON Schema documents

Usage: cargo fm schema [OPTIONS] [SCHEMA]

Arguments:
  [SCHEMA]
          [default: metadata]
          [possible values: metadata, check-report]

Options:
  -o, --write <PATH>
          Write the schema to a file instead of stdout.

  -h, --help
          Print help
```

## `cargo fm completions`

```text
Generate shell completions for cargo-fm

Usage: cargo fm completions <SHELL>

Arguments:
  <SHELL>
          [possible values: bash, elvish, fish, powershell, zsh]

Options:
  -h, --help
          Print help
```

