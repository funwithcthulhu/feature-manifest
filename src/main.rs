use std::ffi::OsString;
use std::path::PathBuf;
use std::process;

use anyhow::{Result, bail};
use clap::{ArgAction, Parser, Subcommand};
use feature_manifest::{
    load_manifest, render_markdown, render_mermaid, resolve_manifest_path, validate,
};

#[derive(Debug, Parser)]
#[command(
    name = "cargo-feature-manifest",
    bin_name = "cargo-feature-manifest",
    version,
    about = "Document, validate, and render Cargo feature metadata.",
    after_help = "Examples:\n  cargo feature-manifest check\n  cargo feature-manifest markdown --manifest-path path/to/crate\n  cargo feature-manifest graph --include-private"
)]
struct Cli {
    #[arg(
        long,
        global = true,
        value_name = "PATH",
        help = "Path to Cargo.toml or a crate directory."
    )]
    manifest_path: Option<PathBuf>,

    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Validate feature metadata and CI-oriented rules.
    Check,
    /// Render a Markdown feature table.
    Markdown {
        #[arg(long, action = ArgAction::SetTrue, help = "Include private/internal features in the output.")]
        include_private: bool,
    },
    /// Emit normalized feature metadata as JSON.
    Json,
    /// Render a Mermaid graph of feature relationships.
    Graph {
        #[arg(long, action = ArgAction::SetTrue, help = "Include private/internal features in the output.")]
        include_private: bool,
    },
}

fn main() {
    if let Err(error) = run() {
        eprintln!("error: {error:#}");
        process::exit(1);
    }
}

fn run() -> Result<()> {
    let cli = Cli::parse_from(normalize_args(std::env::args_os()));
    let manifest_path = resolve_manifest_path(cli.manifest_path.as_deref())?;
    let manifest = load_manifest(&manifest_path)?;

    match cli.command.unwrap_or(Command::Check) {
        Command::Check => run_check(&manifest),
        Command::Markdown { include_private } => {
            println!("{}", render_markdown(&manifest, include_private));
            Ok(())
        }
        Command::Json => {
            println!("{}", serde_json::to_string_pretty(&manifest)?);
            Ok(())
        }
        Command::Graph { include_private } => {
            println!("{}", render_mermaid(&manifest, include_private));
            Ok(())
        }
    }
}

fn run_check(manifest: &feature_manifest::FeatureManifest) -> Result<()> {
    let report = validate(manifest);

    if report.issues.is_empty() {
        println!(
            "{}",
            report.summary(manifest.features.len(), manifest.groups.len())
        );
        return Ok(());
    }

    for issue in &report.issues {
        eprintln!("{issue}");
    }
    eprintln!(
        "{}",
        report.summary(manifest.features.len(), manifest.groups.len())
    );

    if report.has_errors() {
        bail!("validation failed");
    }

    Ok(())
}

fn normalize_args(args: impl IntoIterator<Item = OsString>) -> Vec<OsString> {
    let mut args = args.into_iter().collect::<Vec<_>>();

    if args
        .get(1)
        .and_then(|argument| argument.to_str())
        .is_some_and(|argument| argument == "feature-manifest" || argument == "feature_manifest")
    {
        args.remove(1);
    }

    args
}
