use std::fs;
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
        let source = fs::read_to_string(&package.manifest_path).ok();
        for issue in &report.issues {
            let level = match issue.severity {
                Severity::Warning => "warning",
                Severity::Error => "error",
            };
            let path = portable_relative_path(root_directory, &package.manifest_path);
            let line = source
                .as_deref()
                .and_then(|source| issue_line(source, issue))
                .map(|line| format!(",line={line}"))
                .unwrap_or_default();
            println!(
                "::{level} file={path}{line},title=feature-manifest {code}::{message}",
                code = issue.code,
                message = escape_github_workflow_message(&issue_message(package, issue))
            );
        }
    }
}

fn issue_line(source: &str, issue: &Issue) -> Option<usize> {
    let name = issue.feature.as_deref()?;

    match issue.code {
        "small-group"
        | "duplicate-group-member"
        | "unknown-group-member"
        | "mutually-exclusive-default" => find_group_name_line(source, name),
        "unknown-metadata" => find_metadata_key_line(source, name),
        _ => find_feature_key_line(source, name).or_else(|| find_metadata_key_line(source, name)),
    }
}

fn find_feature_key_line(source: &str, name: &str) -> Option<usize> {
    find_key_line_in_section(source, "[features]", name)
}

fn find_metadata_key_line(source: &str, name: &str) -> Option<usize> {
    [
        "[package.metadata.feature-manifest.features]",
        "[package.metadata.feature-manifest]",
        "[package.metadata.feature-docs.features]",
        "[package.metadata.feature-docs]",
    ]
    .into_iter()
    .find_map(|section| find_key_line_in_section(source, section, name))
}

fn find_key_line_in_section(source: &str, section: &str, name: &str) -> Option<usize> {
    let mut in_section = false;

    for (index, line) in source.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') {
            in_section = trimmed == section;
            continue;
        }

        if in_section && toml_key(line).as_deref() == Some(name) {
            return Some(index + 1);
        }
    }

    None
}

fn find_group_name_line(source: &str, group_name: &str) -> Option<usize> {
    source.lines().enumerate().find_map(|(index, line)| {
        if toml_key(line).as_deref() == Some("name") && line_has_string_value(line, group_name) {
            Some(index + 1)
        } else {
            None
        }
    })
}

fn toml_key(line: &str) -> Option<String> {
    let key = line.split_once('=')?.0.trim();
    if key.starts_with('#') || key.is_empty() {
        return None;
    }

    Some(key.trim_matches('"').trim_matches('\'').trim().to_owned())
}

fn line_has_string_value(line: &str, value: &str) -> bool {
    line.contains(&format!("\"{value}\"")) || line.contains(&format!("'{value}'"))
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
