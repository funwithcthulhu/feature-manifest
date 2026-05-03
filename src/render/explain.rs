use anyhow::{Result, bail};

use crate::model::{Feature, FeatureManifest, WorkspaceManifest};

use super::shared::{reference_summary, status_summary, visibility_label, yes_no};

/// Renders a human-readable explanation for a named feature.
pub fn render_explain(
    workspace: &WorkspaceManifest,
    feature_name: &str,
    include_private: bool,
) -> Result<String> {
    let matching_packages = workspace
        .packages
        .iter()
        .filter_map(|package| {
            package.features.get(feature_name).and_then(|feature| {
                if include_private || feature.metadata.public {
                    Some((package, feature))
                } else {
                    None
                }
            })
        })
        .collect::<Vec<_>>();

    if matching_packages.is_empty() {
        bail!("feature `{feature_name}` was not found in the selected package set");
    }

    let mut sections = Vec::new();

    for (index, (package, feature)) in matching_packages.into_iter().enumerate() {
        if workspace.is_single_package() {
            sections.push(render_feature_explanation(package, feature));
            continue;
        }

        if index > 0 {
            sections.push(String::new());
        }
        sections.push(format!(
            "## {}",
            package.package_name.as_deref().unwrap_or("unknown-package")
        ));
        sections.push(String::new());
        sections.push(render_feature_explanation(package, feature));
    }

    Ok(sections.join("\n"))
}

fn render_feature_explanation(manifest: &FeatureManifest, feature: &Feature) -> String {
    let groups = manifest
        .groups_for_feature(&feature.name)
        .into_iter()
        .map(|group| group.name.clone())
        .collect::<Vec<_>>();
    let reverse_dependencies = manifest
        .reverse_dependencies(&feature.name)
        .into_iter()
        .map(|candidate| candidate.name.clone())
        .collect::<Vec<_>>();

    let mut lines = vec![format!("Feature: `{}`", feature.name)];
    if let Some(package_name) = &manifest.package_name {
        lines.push(format!("Package: {package_name}"));
    }
    lines.push(format!(
        "Description: {}",
        feature
            .metadata
            .description
            .as_deref()
            .unwrap_or("No description provided.")
    ));
    lines.push(format!(
        "Default enabled: {}",
        yes_no(feature.default_enabled)
    ));
    lines.push(format!(
        "Visibility: {}",
        visibility_label(&feature.metadata)
    ));
    lines.push(format!("Status: {}", status_summary(&feature.metadata)));
    lines.push(format!(
        "Metadata table: {}",
        manifest
            .metadata_table
            .as_deref()
            .unwrap_or("package.metadata.feature-manifest")
    ));
    lines.push(format!("Enables: {}", reference_summary(&feature.enables)));

    if manifest.default_features.contains(&feature.name) {
        lines.push("Included in default feature set: yes".to_owned());
    } else {
        lines.push("Included in default feature set: no".to_owned());
    }

    lines.push(format!(
        "Groups: {}",
        if groups.is_empty() {
            "none".to_owned()
        } else {
            groups.join(", ")
        }
    ));
    lines.push(format!(
        "Required by: {}",
        if reverse_dependencies.is_empty() {
            "no feature references".to_owned()
        } else {
            reverse_dependencies.join(", ")
        }
    ));

    if let Some(note) = &feature.metadata.note {
        lines.push(format!("Note: {note}"));
    }
    if let Some(category) = &feature.metadata.category {
        lines.push(format!("Category: {category}"));
    }
    if let Some(since) = &feature.metadata.since {
        lines.push(format!("Since: {since}"));
    }
    if let Some(docs) = &feature.metadata.docs {
        lines.push(format!("Docs: {docs}"));
    }
    if let Some(tracking_issue) = &feature.metadata.tracking_issue {
        lines.push(format!("Tracking issue: {tracking_issue}"));
    }
    if !feature.metadata.requires.is_empty() {
        lines.push(format!(
            "Requires: {}",
            feature.metadata.requires.join(", ")
        ));
    }

    lines.join("\n")
}
