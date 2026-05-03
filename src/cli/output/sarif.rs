use std::path::Path;

use serde_json::json;

use crate::cli::commands::check::PackageReport;
use crate::cli::output::github::issue_message;
use crate::cli::util::portable_relative_path;
use crate::{Severity, WorkspaceManifest, known_lint_codes};

pub fn render(
    workspace: &WorkspaceManifest,
    package_reports: &[PackageReport<'_>],
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
                        Severity::Warning => "warning",
                        Severity::Error => "error",
                    },
                    "message": {
                        "text": issue_message(package, issue),
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
