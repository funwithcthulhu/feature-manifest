use std::ffi::OsString;
use std::path::PathBuf;
use std::process;

use anyhow::{Result, bail};
use clap::{ArgAction, Parser, Subcommand};
use feature_manifest::{
    PackageSelection, WorkspaceManifest, load_workspace, render_explain, render_json,
    render_markdown, render_mermaid, resolve_manifest_path, sync_manifest, validate,
};

#[derive(Debug, Parser)]
#[command(
    name = "cargo-feature-manifest",
    bin_name = "cargo-feature-manifest",
    version,
    about = "Document, validate, and render Cargo feature metadata.",
    after_help = "Examples:\n  cargo feature-manifest check\n  cargo feature-manifest --workspace check\n  cargo feature-manifest --package cli explain serde\n  cargo feature-manifest sync --manifest-path path/to/crate\n  cargo feature-manifest json --workspace"
)]
struct Cli {
    #[arg(
        long,
        global = true,
        value_name = "PATH",
        help = "Path to Cargo.toml or a crate directory."
    )]
    manifest_path: Option<PathBuf>,

    #[arg(
        long,
        global = true,
        action = ArgAction::SetTrue,
        help = "Operate on every workspace member."
    )]
    workspace: bool,

    #[arg(
        long,
        global = true,
        value_name = "NAME",
        help = "Select a specific package within a workspace."
    )]
    package: Option<String>,

    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Validate feature metadata and CI-oriented rules.
    Check,
    /// Render a Markdown feature table.
    Markdown {
        #[arg(
            long,
            action = ArgAction::SetTrue,
            help = "Include private/internal features in the output."
        )]
        include_private: bool,
    },
    /// Emit normalized machine-readable feature metadata as JSON.
    Json,
    /// Render a Mermaid graph of feature relationships.
    Graph {
        #[arg(
            long,
            action = ArgAction::SetTrue,
            help = "Include private/internal features in the output."
        )]
        include_private: bool,
    },
    /// Scaffold missing metadata entries directly into Cargo.toml.
    Sync,
    /// Explain one feature in human-readable form.
    Explain {
        feature: String,
        #[arg(
            long,
            action = ArgAction::SetTrue,
            help = "Include private/internal features when searching for matches."
        )]
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
    let selection = selection_from_cli(&cli)?;
    let manifest_path = resolve_manifest_path(cli.manifest_path.as_deref())?;
    let workspace = load_workspace(&manifest_path, selection)?;

    match cli.command.unwrap_or(Command::Check) {
        Command::Check => run_check(&workspace),
        Command::Markdown { include_private } => {
            println!("{}", render_markdown(&workspace, include_private));
            Ok(())
        }
        Command::Json => {
            println!("{}", render_json(&workspace)?);
            Ok(())
        }
        Command::Graph { include_private } => {
            println!("{}", render_mermaid(&workspace, include_private));
            Ok(())
        }
        Command::Sync => run_sync(&workspace),
        Command::Explain {
            feature,
            include_private,
        } => {
            println!("{}", render_explain(&workspace, &feature, include_private)?);
            Ok(())
        }
    }
}

fn run_check(workspace: &WorkspaceManifest) -> Result<()> {
    let mut total_errors = 0usize;
    let mut total_warnings = 0usize;
    let mut total_features = 0usize;
    let mut total_groups = 0usize;

    for package in &workspace.packages {
        let report = validate(package);
        total_errors += report.error_count();
        total_warnings += report.warning_count();
        total_features += package.features.len();
        total_groups += package.groups.len();

        if workspace.is_single_package() {
            emit_package_report(None, package, &report);
            continue;
        }

        emit_package_report(package.package_name.as_deref(), package, &report);
    }

    if !workspace.is_single_package() {
        eprintln!(
            "workspace summary: validated {} package(s), {total_features} feature(s), {total_groups} group(s): {total_errors} error(s), {total_warnings} warning(s)",
            workspace.packages.len()
        );
    }

    if total_errors > 0 {
        bail!("validation failed");
    }

    Ok(())
}

fn emit_package_report(
    package_name: Option<&str>,
    package: &feature_manifest::FeatureManifest,
    report: &feature_manifest::ValidationReport,
) {
    let summary = report.summary(package.features.len(), package.groups.len());

    if report.issues.is_empty() {
        if package_name.is_some() {
            println!("package `{}`", package_name.unwrap_or("unknown-package"));
            println!("  {summary}");
        } else {
            println!("{summary}");
        }
        return;
    }

    if let Some(package_name) = package_name {
        eprintln!("package `{package_name}`");
    }

    for issue in &report.issues {
        if package_name.is_some() {
            eprintln!("  {issue}");
        } else {
            eprintln!("{issue}");
        }
    }

    if package_name.is_some() {
        eprintln!("  {summary}");
    } else {
        eprintln!("{summary}");
    }
}

fn run_sync(workspace: &WorkspaceManifest) -> Result<()> {
    let mut changed_packages = 0usize;

    for package in &workspace.packages {
        let report = sync_manifest(&package.manifest_path)?;
        let package_name = report.package_name.as_deref().unwrap_or("unknown-package");

        if report.changed() {
            changed_packages += 1;
            println!(
                "synced `{package_name}`: added {} metadata entr{} under `[package.metadata.{}]`",
                report.added_features.len(),
                if report.added_features.len() == 1 {
                    "y"
                } else {
                    "ies"
                },
                report.metadata_table
            );
            for feature in &report.added_features {
                println!("  - {feature}");
            }
        } else {
            println!("`{package_name}` is already in sync");
        }
    }

    if changed_packages > 0 {
        println!("updated {changed_packages} package(s)");
    }

    Ok(())
}

fn selection_from_cli(cli: &Cli) -> Result<PackageSelection> {
    if cli.workspace && cli.package.is_some() {
        bail!("`--workspace` and `--package` cannot be used together");
    }

    if cli.workspace {
        return Ok(PackageSelection::Workspace);
    }

    if let Some(package_name) = &cli.package {
        return Ok(PackageSelection::Package(package_name.clone()));
    }

    Ok(PackageSelection::Default)
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
