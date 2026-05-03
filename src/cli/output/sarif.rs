use std::fs;
use std::path::Path;

use serde_json::json;

use crate::cli::commands::check::PackageReport;
use crate::cli::output::github::issue_message;
use crate::cli::util::portable_relative_path;
use crate::{ManifestSourceMap, Severity, WorkspaceManifest, lint_docs};

pub fn render(
    workspace: &WorkspaceManifest,
    package_reports: &[PackageReport<'_>],
) -> serde_json::Value {
    let root_directory = workspace
        .root_manifest_path
        .parent()
        .unwrap_or_else(|| Path::new("."));

    let rules = lint_docs()
        .iter()
        .map(|lint| {
            json!({
                "id": lint.code,
                "name": lint.code,
                "shortDescription": { "text": lint.summary },
                "help": { "text": lint.guidance },
            })
        })
        .collect::<Vec<_>>();

    let results = package_reports
        .iter()
        .flat_map(|(package, report)| {
            let source = fs::read_to_string(&package.manifest_path).ok();
            report.issues.iter().map(move |issue| {
                let mut physical_location = json!({
                    "artifactLocation": {
                        "uri": portable_relative_path(root_directory, &package.manifest_path),
                    }
                });

                if let Some(span) = source
                    .as_deref()
                    .and_then(|source| ManifestSourceMap::new(source).span_for_issue(issue))
                {
                    physical_location["region"] = json!({
                        "startLine": span.line,
                        "startColumn": span.column,
                    });
                }

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
                            "physicalLocation": physical_location,
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
