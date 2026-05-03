use std::path::Path;

use crate::cli::commands::check::PackageReport;
use crate::cli::util::{escape_github_workflow_message, portable_relative_path};
use crate::{FeatureManifest, Issue, Severity, WorkspaceManifest};

pub fn emit(workspace: &WorkspaceManifest, package_reports: &[PackageReport<'_>]) {
    let root_directory = workspace
        .root_manifest_path
        .parent()
        .unwrap_or_else(|| Path::new("."));

    for (package, report) in package_reports {
        for issue in &report.issues {
            let level = match issue.severity {
                Severity::Warning => "warning",
                Severity::Error => "error",
            };
            let path = portable_relative_path(root_directory, &package.manifest_path);
            println!(
                "::{level} file={path},title=feature-manifest {code}::{message}",
                code = issue.code,
                message = escape_github_workflow_message(&issue_message(package, issue))
            );
        }
    }
}

pub fn issue_message(package: &FeatureManifest, issue: &Issue) -> String {
    let package_name = package.package_name.as_deref().unwrap_or("unknown-package");
    match &issue.feature {
        Some(feature) => format!(
            "package `{package_name}` feature `{feature}`: {}",
            issue.message
        ),
        None => format!("package `{package_name}`: {}", issue.message),
    }
}
