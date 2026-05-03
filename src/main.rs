use std::collections::BTreeMap;
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process;

use anyhow::{Result, bail};
use clap::{ArgAction, Parser, Subcommand, ValueEnum};
use feature_manifest::{
    InjectionMarkers, MetadataLayout, PackageSelection, SyncOptions, WorkspaceManifest,
    inject_between_markers, known_lint_codes, load_workspace, parse_lint_override, render_explain,
    render_json, render_markdown, render_mermaid, resolve_manifest_path, sync_manifest,
    validate_with_options, write_output,
};
use serde::Serialize;
use serde_json::json;

#[derive(Debug, Parser)]
#[command(
    name = "cargo-feature-manifest",
    bin_name = "cargo-feature-manifest",
    version,
    about = "Document, validate, and render Cargo feature metadata.",
    after_help = "Examples:\n  cargo feature-manifest check\n  cargo feature-manifest check --format sarif\n  cargo feature-manifest --workspace check --lint missing-description=warn\n  cargo feature-manifest markdown --write FEATURES.md\n  cargo feature-manifest markdown --insert-into README.md\n  cargo feature-manifest sync --check --remove-stale --style structured\n  cargo feature-manifest --package cli explain serde"
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
    Check {
        #[arg(long, value_enum, default_value_t = CheckFormat::Text)]
        format: CheckFormat,
        #[arg(
            long = "lint",
            value_name = "CODE=LEVEL",
            action = ArgAction::Append,
            help = "Override one lint level for this run. Known codes: emitted in `--help` docs and docs/metadata-format.md."
        )]
        lint_overrides: Vec<String>,
    },
    /// Render a Markdown feature table.
    Markdown {
        #[arg(
            long,
            action = ArgAction::SetTrue,
            help = "Include private/internal features in the output."
        )]
        include_private: bool,
        #[arg(
            long,
            value_name = "PATH",
            help = "Write generated Markdown to a file."
        )]
        write: Option<PathBuf>,
        #[arg(
            long,
            value_name = "PATH",
            help = "Inject generated Markdown between markers in an existing file."
        )]
        insert_into: Option<PathBuf>,
        #[arg(
            long,
            value_name = "TEXT",
            default_value = "<!-- feature-manifest:start -->",
            help = "Start marker used by `--insert-into`."
        )]
        start_marker: String,
        #[arg(
            long,
            value_name = "TEXT",
            default_value = "<!-- feature-manifest:end -->",
            help = "End marker used by `--insert-into`."
        )]
        end_marker: String,
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
    Sync {
        #[arg(
            long,
            action = ArgAction::SetTrue,
            help = "Exit non-zero if changes would be needed, without rewriting files."
        )]
        check: bool,
        #[arg(
            long,
            action = ArgAction::SetTrue,
            help = "Remove stale metadata entries for missing features."
        )]
        remove_stale: bool,
        #[arg(long, value_enum, help = "Choose the metadata layout to write back.")]
        style: Option<SyncStyle>,
    },
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
    /// List the lint codes supported by `check`.
    ListLints,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum CheckFormat {
    Text,
    Json,
    Github,
    Sarif,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum SyncStyle {
    Flat,
    Structured,
}

impl From<SyncStyle> for MetadataLayout {
    fn from(value: SyncStyle) -> Self {
        match value {
            SyncStyle::Flat => MetadataLayout::Flat,
            SyncStyle::Structured => MetadataLayout::Structured,
        }
    }
}

#[derive(Debug, Serialize)]
struct JsonCheckReport {
    schema_version: u32,
    packages: Vec<JsonCheckPackage>,
    summary: JsonCheckSummary,
}

#[derive(Debug, Serialize)]
struct JsonCheckPackage {
    package_name: Option<String>,
    manifest_path: String,
    errors: usize,
    warnings: usize,
    issues: Vec<JsonIssue>,
}

#[derive(Debug, Serialize)]
struct JsonIssue {
    code: String,
    severity: String,
    feature: Option<String>,
    message: String,
}

#[derive(Debug, Serialize)]
struct JsonCheckSummary {
    packages: usize,
    features: usize,
    groups: usize,
    errors: usize,
    warnings: usize,
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

    match cli.command.unwrap_or(Command::Check {
        format: CheckFormat::Text,
        lint_overrides: Vec::new(),
    }) {
        Command::Check {
            format,
            lint_overrides,
        } => run_check(&workspace, format, &lint_overrides),
        Command::Markdown {
            include_private,
            write,
            insert_into,
            start_marker,
            end_marker,
        } => run_markdown(
            &workspace,
            include_private,
            write,
            insert_into,
            InjectionMarkers {
                start: start_marker,
                end: end_marker,
            },
        ),
        Command::Json => {
            println!("{}", render_json(&workspace)?);
            Ok(())
        }
        Command::Graph { include_private } => {
            println!("{}", render_mermaid(&workspace, include_private));
            Ok(())
        }
        Command::Sync {
            check,
            remove_stale,
            style,
        } => run_sync(
            &workspace,
            SyncOptions {
                check_only: check,
                remove_stale,
                style: style.map(Into::into),
            },
        ),
        Command::Explain {
            feature,
            include_private,
        } => {
            println!("{}", render_explain(&workspace, &feature, include_private)?);
            Ok(())
        }
        Command::ListLints => {
            for code in known_lint_codes() {
                println!("{code}");
            }
            Ok(())
        }
    }
}

fn run_check(
    workspace: &WorkspaceManifest,
    format: CheckFormat,
    lint_overrides: &[String],
) -> Result<()> {
    let mut cli_lints = BTreeMap::new();
    for override_value in lint_overrides {
        let (code, level) = parse_lint_override(override_value)?;
        cli_lints.insert(code, level);
    }

    let validate_options = feature_manifest::ValidateOptions::with_cli_lint_overrides(cli_lints);
    let package_reports = workspace
        .packages
        .iter()
        .map(|package| (package, validate_with_options(package, &validate_options)))
        .collect::<Vec<_>>();

    let summary = workspace_summary(workspace, &package_reports);

    match format {
        CheckFormat::Text => emit_text_check_output(workspace, &package_reports, &summary),
        CheckFormat::Json => println!(
            "{}",
            serde_json::to_string_pretty(&json_check_report(
                workspace,
                &package_reports,
                &summary
            ))?
        ),
        CheckFormat::Github => emit_github_check_output(workspace, &package_reports),
        CheckFormat::Sarif => println!(
            "{}",
            serde_json::to_string_pretty(&sarif_report(workspace, &package_reports))?
        ),
    }

    if summary.errors > 0 {
        bail!("validation failed");
    }

    Ok(())
}

fn run_markdown(
    workspace: &WorkspaceManifest,
    include_private: bool,
    write: Option<PathBuf>,
    insert_into: Option<PathBuf>,
    markers: InjectionMarkers,
) -> Result<()> {
    let markdown = render_markdown(workspace, include_private);
    let mut wrote_somewhere = false;

    if let Some(path) = write {
        write_output(&path, &markdown)?;
        println!("wrote Markdown to `{}`", path.display());
        wrote_somewhere = true;
    }

    if let Some(path) = insert_into {
        inject_between_markers(&path, &markdown, &markers)?;
        println!("injected Markdown into `{}`", path.display());
        wrote_somewhere = true;
    }

    if !wrote_somewhere {
        println!("{markdown}");
    }

    Ok(())
}

fn run_sync(workspace: &WorkspaceManifest, options: SyncOptions) -> Result<()> {
    let mut changed_packages = 0usize;
    let mut check_failed = false;

    for package in &workspace.packages {
        let report = sync_manifest(&package.manifest_path, &options)?;
        let package_name = report.package_name.as_deref().unwrap_or("unknown-package");

        if report.changed() {
            changed_packages += 1;
            if options.check_only {
                check_failed = true;
                println!(
                    "sync drift in `{package_name}`: {}",
                    sync_change_summary(&report)
                );
            } else {
                println!("synced `{package_name}`: {}", sync_change_summary(&report));
            }

            for feature in &report.added_features {
                println!("  + {feature}");
            }
            for feature in &report.removed_features {
                println!("  - {feature}");
            }
        } else if options.check_only {
            println!("`{package_name}` is already in sync");
        } else {
            println!("`{package_name}` is already in sync");
        }
    }

    if changed_packages > 0 && !options.check_only {
        println!("updated {changed_packages} package(s)");
    }

    if check_failed {
        bail!("sync drift detected");
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

fn emit_text_check_output(
    workspace: &WorkspaceManifest,
    package_reports: &[(
        &feature_manifest::FeatureManifest,
        feature_manifest::ValidationReport,
    )],
    summary: &Summary,
) {
    for (package, report) in package_reports {
        if workspace.is_single_package() {
            emit_package_report(None, package, report);
            continue;
        }

        emit_package_report(package.package_name.as_deref(), package, report);
    }

    if !workspace.is_single_package() {
        eprintln!(
            "workspace summary: validated {} package(s), {} feature(s), {} group(s): {} error(s), {} warning(s)",
            summary.packages, summary.features, summary.groups, summary.errors, summary.warnings
        );
    }
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

fn emit_github_check_output(
    workspace: &WorkspaceManifest,
    package_reports: &[(
        &feature_manifest::FeatureManifest,
        feature_manifest::ValidationReport,
    )],
) {
    let root_directory = workspace
        .root_manifest_path
        .parent()
        .unwrap_or_else(|| Path::new("."));

    for (package, report) in package_reports {
        for issue in &report.issues {
            let level = match issue.severity {
                feature_manifest::Severity::Warning => "warning",
                feature_manifest::Severity::Error => "error",
            };
            let path = portable_relative_path(root_directory, &package.manifest_path);
            println!(
                "::{level} file={path},title=feature-manifest {code}::{message}",
                code = issue.code,
                message = escape_github_workflow_message(&github_issue_message(package, issue))
            );
        }
    }
}

fn github_issue_message(
    package: &feature_manifest::FeatureManifest,
    issue: &feature_manifest::Issue,
) -> String {
    let package_name = package.package_name.as_deref().unwrap_or("unknown-package");
    match &issue.feature {
        Some(feature) => format!(
            "package `{package_name}` feature `{feature}`: {}",
            issue.message
        ),
        None => format!("package `{package_name}`: {}", issue.message),
    }
}

fn json_check_report(
    workspace: &WorkspaceManifest,
    package_reports: &[(
        &feature_manifest::FeatureManifest,
        feature_manifest::ValidationReport,
    )],
    summary: &Summary,
) -> JsonCheckReport {
    let root_directory = workspace
        .root_manifest_path
        .parent()
        .unwrap_or_else(|| Path::new("."));

    JsonCheckReport {
        schema_version: 1,
        packages: package_reports
            .iter()
            .map(|(package, report)| JsonCheckPackage {
                package_name: package.package_name.clone(),
                manifest_path: portable_relative_path(root_directory, &package.manifest_path),
                errors: report.error_count(),
                warnings: report.warning_count(),
                issues: report
                    .issues
                    .iter()
                    .map(|issue| JsonIssue {
                        code: issue.code.to_owned(),
                        severity: issue.severity.to_string(),
                        feature: issue.feature.clone(),
                        message: issue.message.clone(),
                    })
                    .collect(),
            })
            .collect(),
        summary: JsonCheckSummary {
            packages: summary.packages,
            features: summary.features,
            groups: summary.groups,
            errors: summary.errors,
            warnings: summary.warnings,
        },
    }
}

fn sarif_report(
    workspace: &WorkspaceManifest,
    package_reports: &[(
        &feature_manifest::FeatureManifest,
        feature_manifest::ValidationReport,
    )],
) -> serde_json::Value {
    let root_directory = workspace
        .root_manifest_path
        .parent()
        .unwrap_or_else(|| Path::new("."));

    let rules = known_lint_codes()
        .iter()
        .map(|code| {
            json!({
                "id": code,
                "name": code,
                "shortDescription": { "text": code },
            })
        })
        .collect::<Vec<_>>();

    let results = package_reports
        .iter()
        .flat_map(|(package, report)| {
            report.issues.iter().map(|issue| {
                json!({
                    "ruleId": issue.code,
                    "level": match issue.severity {
                        feature_manifest::Severity::Warning => "warning",
                        feature_manifest::Severity::Error => "error",
                    },
                    "message": {
                        "text": github_issue_message(package, issue),
                    },
                    "locations": [
                        {
                            "physicalLocation": {
                                "artifactLocation": {
                                    "uri": portable_relative_path(root_directory, &package.manifest_path),
                                }
                            }
                        }
                    ]
                })
            })
        })
        .collect::<Vec<_>>();

    json!({
        "$schema": "https://json.schemastore.org/sarif-2.1.0.json",
        "version": "2.1.0",
        "runs": [
            {
                "tool": {
                    "driver": {
                        "name": "feature-manifest",
                        "informationUri": "https://github.com/funwithcthulhu/feature-manifest",
                        "rules": rules,
                    }
                },
                "results": results,
            }
        ]
    })
}

fn workspace_summary(
    workspace: &WorkspaceManifest,
    package_reports: &[(
        &feature_manifest::FeatureManifest,
        feature_manifest::ValidationReport,
    )],
) -> Summary {
    Summary {
        packages: workspace.packages.len(),
        features: workspace
            .packages
            .iter()
            .map(|package| package.features.len())
            .sum(),
        groups: workspace
            .packages
            .iter()
            .map(|package| package.groups.len())
            .sum(),
        errors: package_reports
            .iter()
            .map(|(_, report)| report.error_count())
            .sum(),
        warnings: package_reports
            .iter()
            .map(|(_, report)| report.warning_count())
            .sum(),
    }
}

fn sync_change_summary(report: &feature_manifest::SyncReport) -> String {
    let mut parts = Vec::new();
    if !report.added_features.is_empty() {
        parts.push(format!(
            "added {}",
            pluralized(report.added_features.len(), "feature")
        ));
    }
    if !report.removed_features.is_empty() {
        parts.push(format!(
            "removed {} stale metadata entr{}",
            report.removed_features.len(),
            if report.removed_features.len() == 1 {
                "y"
            } else {
                "ies"
            }
        ));
    }
    if parts.is_empty() {
        parts.push("layout updated".to_owned());
    }
    parts.push(format!("using `{}` layout", report.style));
    parts.join(", ")
}

fn pluralized(count: usize, noun: &str) -> String {
    if count == 1 {
        format!("1 {noun}")
    } else {
        format!("{count} {noun}s")
    }
}

fn portable_relative_path(root_directory: &Path, path: &Path) -> String {
    let relative = path.strip_prefix(root_directory).unwrap_or(path);
    relative
        .components()
        .map(|component| component.as_os_str().to_string_lossy().into_owned())
        .collect::<Vec<_>>()
        .join("/")
}

fn escape_github_workflow_message(message: &str) -> String {
    message
        .replace('%', "%25")
        .replace('\r', "%0D")
        .replace('\n', "%0A")
        .replace(':', "%3A")
        .replace(',', "%2C")
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

#[derive(Debug, Clone, Copy)]
struct Summary {
    packages: usize,
    features: usize,
    groups: usize,
    errors: usize,
    warnings: usize,
}
