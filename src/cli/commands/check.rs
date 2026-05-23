use std::collections::BTreeMap;

use anyhow::{Context, Result, bail};

use crate::cli::output::{check_json, github, sarif};
use crate::{FeatureManifest, LintLevel, LintPreset, ValidationReport, WorkspaceManifest};
use crate::{ValidateOptions, parse_lint_override, validate_with_options};

use super::super::CheckFormat;

pub type PackageReport<'a> = (&'a FeatureManifest, ValidationReport);

#[derive(Debug, Clone, Copy)]
pub struct Summary {
    pub packages: usize,
    pub features: usize,
    pub groups: usize,
    pub errors: usize,
    pub warnings: usize,
}

pub fn run(
    workspace: &WorkspaceManifest,
    format: CheckFormat,
    lint_overrides: &[String],
    preset: Option<LintPreset>,
) -> Result<()> {
    let package_reports = collect_reports(workspace, lint_overrides, preset)?;
    let summary = workspace_summary(workspace, &package_reports);

    match format {
        CheckFormat::Text => emit_text(workspace, &package_reports, &summary),
        CheckFormat::Json => println!(
            "{}",
            check_json::render(workspace, &package_reports, &summary)?
        ),
        CheckFormat::Github => github::emit(workspace, &package_reports),
        CheckFormat::Sarif => println!(
            "{}",
            serde_json::to_string_pretty(&sarif::render(workspace, &package_reports))?
        ),
    }

    if summary.errors > 0 {
        bail!("validation failed");
    }

    Ok(())
}

pub fn collect_reports<'a>(
    workspace: &'a WorkspaceManifest,
    lint_overrides: &[String],
    preset: Option<LintPreset>,
) -> Result<Vec<PackageReport<'a>>> {
    let mut cli_lints = BTreeMap::<String, LintLevel>::new();
    for override_value in lint_overrides {
        let (code, level) = parse_lint_override(override_value)
            .with_context(|| format!("invalid lint override `{override_value}`"))?;
        cli_lints.insert(code, level);
    }

    let validate_options =
        ValidateOptions::with_cli_lint_overrides(cli_lints).with_cli_preset(preset);

    Ok(workspace
        .packages
        .iter()
        .map(|package| (package, validate_with_options(package, &validate_options)))
        .collect())
}

pub fn workspace_summary(
    workspace: &WorkspaceManifest,
    package_reports: &[PackageReport<'_>],
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

fn emit_text(
    workspace: &WorkspaceManifest,
    package_reports: &[PackageReport<'_>],
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
    package: &FeatureManifest,
    report: &ValidationReport,
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
