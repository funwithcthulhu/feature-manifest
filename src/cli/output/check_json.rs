use serde::Serialize;

use crate::FeatureManifest;
use crate::ValidationReport;
use crate::WorkspaceManifest;
use crate::cli::commands::check::{PackageReport, Summary};
use crate::cli::util::portable_relative_path;

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

pub fn render(
    workspace: &WorkspaceManifest,
    package_reports: &[PackageReport<'_>],
    summary: &Summary,
) -> Result<String, serde_json::Error> {
    serde_json::to_string_pretty(&report(workspace, package_reports, summary))
}

fn report(
    workspace: &WorkspaceManifest,
    package_reports: &[(&FeatureManifest, ValidationReport)],
    summary: &Summary,
) -> JsonCheckReport {
    let root_directory = workspace
        .root_manifest_path
        .parent()
        .unwrap_or_else(|| std::path::Path::new("."));

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
