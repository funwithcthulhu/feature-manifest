use std::collections::BTreeSet;

use anyhow::{Result, bail};

use crate::model::{Feature, FeatureManifest, FeatureMetadata, FeatureRef, WorkspaceManifest};

/// Renders the selected packages as Markdown.
pub fn render_markdown(workspace: &WorkspaceManifest, include_private: bool) -> String {
    if workspace.is_single_package() {
        return render_markdown_package(&workspace.packages[0], include_private, 1);
    }

    let mut lines = vec!["# Workspace feature manifest".to_owned(), String::new()];

    let package_list = workspace
        .package_names()
        .iter()
        .map(|name| format!("`{name}`"))
        .collect::<Vec<_>>()
        .join(", ");

    lines.push(format!("Selected packages: {package_list}"));
    lines.push(String::new());

    for (index, package) in workspace.packages.iter().enumerate() {
        if index > 0 {
            lines.push(String::new());
        }
        lines.push(render_markdown_package(package, include_private, 2));
    }

    lines.join("\n")
}

/// Renders selected packages as a Mermaid graph.
pub fn render_mermaid(workspace: &WorkspaceManifest, include_private: bool) -> String {
    if workspace.is_single_package() {
        return render_mermaid_package(&workspace.packages[0], include_private, None);
    }

    let mut lines = vec!["graph TD".to_owned()];

    for package in &workspace.packages {
        let package_name = package
            .package_name
            .as_deref()
            .unwrap_or("unknown-package")
            .to_owned();
        let package_prefix = sanitize_id(&package_name);
        lines.push(format!(
            "    subgraph package_{package_prefix}[\"{}\"]",
            escape_mermaid_label(&package_name)
        ));

        for line in render_mermaid_package(package, include_private, Some(&package_prefix))
            .lines()
            .skip(1)
        {
            lines.push(format!("    {line}"));
        }

        lines.push("    end".to_owned());
    }

    lines.join("\n")
}

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

fn render_markdown_package(
    manifest: &FeatureManifest,
    include_private: bool,
    heading_level: usize,
) -> String {
    let heading = "#".repeat(heading_level);
    let title = manifest
        .package_name
        .as_deref()
        .map(|name| format!("{heading} {name} feature manifest"))
        .unwrap_or_else(|| format!("{heading} Feature manifest"));

    let mut lines = vec![title, String::new()];
    lines.push(default_feature_summary(manifest));
    lines.push(String::new());
    lines.push("| Feature | Default | Visibility | Status | Enables | Description |".to_owned());
    lines.push("| --- | --- | --- | --- | --- | --- |".to_owned());

    let mut hidden_count = 0usize;

    for feature in manifest.ordered_features() {
        if !include_private && !feature.metadata.public {
            hidden_count += 1;
            continue;
        }

        lines.push(format!(
            "| `{}` | {} | {} | {} | {} | {} |",
            escape_markdown_inline(&feature.name),
            yes_no(feature.default_enabled),
            visibility_label(&feature.metadata),
            escape_markdown_inline(&status_summary(&feature.metadata)),
            escape_markdown_cell(&reference_summary(&feature.enables)),
            escape_markdown_cell(&description_summary(feature)),
        ));
    }

    if hidden_count > 0 && !include_private {
        lines.push(String::new());
        lines.push(format!(
            "_{hidden_count} internal/private feature(s) hidden. Use `--include-private` to render all._"
        ));
    }

    if !manifest.groups.is_empty() {
        lines.push(String::new());
        lines.push(format!("{heading}# Groups"));
        lines.push(String::new());

        for group in &manifest.groups {
            let mut line = format!(
                "- `{}`: {}",
                escape_markdown_inline(&group.name),
                escape_markdown_cell(
                    group
                        .description
                        .as_deref()
                        .unwrap_or("No description provided.")
                )
            );
            if group.mutually_exclusive {
                line.push_str(" Mutually exclusive.");
            }
            line.push_str(&format!(
                " Members: {}.",
                group
                    .members
                    .iter()
                    .map(|member| format!("`{}`", escape_markdown_inline(member)))
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
            lines.push(line);
        }
    }

    lines.join("\n")
}

fn render_mermaid_package(
    manifest: &FeatureManifest,
    include_private: bool,
    package_prefix: Option<&str>,
) -> String {
    let visible_features = manifest
        .ordered_features()
        .into_iter()
        .filter(|feature| include_private || feature.metadata.public)
        .collect::<Vec<_>>();

    if visible_features.is_empty() {
        return "graph TD\n    empty[\"No public features declared\"]".to_owned();
    }

    let visible_names = visible_features
        .iter()
        .map(|feature| feature.name.as_str())
        .collect::<BTreeSet<_>>();
    let mut lines = vec!["graph TD".to_owned()];

    if visible_features
        .iter()
        .any(|feature| feature.default_enabled)
    {
        lines.push(format!(
            "    {}[\"default\"]",
            default_node_id(package_prefix)
        ));
    }

    for feature in &visible_features {
        lines.push(format!(
            "    {}[\"{}\"]",
            feature_node_id(package_prefix, &feature.name),
            escape_mermaid_label(&feature_label(feature)),
        ));
    }

    let mut external_nodes = BTreeSet::new();

    for feature in &visible_features {
        if feature.default_enabled {
            lines.push(format!(
                "    {} --> {}",
                default_node_id(package_prefix),
                feature_node_id(package_prefix, &feature.name)
            ));
        }

        for reference in &feature.enables {
            if let Some(local_feature_name) = reference.local_feature_name() {
                if visible_names.contains(local_feature_name) {
                    lines.push(format!(
                        "    {} --> {}",
                        feature_node_id(package_prefix, &feature.name),
                        feature_node_id(package_prefix, local_feature_name)
                    ));
                    continue;
                }
            }

            let external_id = reference_node_id(package_prefix, reference);
            if external_nodes.insert(external_id.clone()) {
                lines.push(format!(
                    "    {}[\"{}\"]",
                    external_id,
                    escape_mermaid_label(&reference.to_string())
                ));
            }
            lines.push(format!(
                "    {} --> {}",
                feature_node_id(package_prefix, &feature.name),
                external_id
            ));
        }
    }

    lines.join("\n")
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

    lines.join("\n")
}

fn default_feature_summary(manifest: &FeatureManifest) -> String {
    if manifest.default_members.is_empty() {
        "Default feature set: _none_".to_owned()
    } else {
        format!(
            "Default feature set: {}",
            manifest
                .default_members
                .iter()
                .map(|member| format!("`{}`", escape_markdown_inline(&member.to_string())))
                .collect::<Vec<_>>()
                .join(", ")
        )
    }
}

fn description_summary(feature: &Feature) -> String {
    let description = feature
        .metadata
        .description
        .as_deref()
        .unwrap_or("No description provided.");

    match &feature.metadata.note {
        Some(note) => format!("{description} Note: {note}"),
        None => description.to_owned(),
    }
}

fn reference_summary(references: &[FeatureRef]) -> String {
    if references.is_empty() {
        return "—".to_owned();
    }

    references
        .iter()
        .map(|reference| format!("`{}`", escape_markdown_inline(&reference.to_string())))
        .collect::<Vec<_>>()
        .join(", ")
}

fn feature_label(feature: &Feature) -> String {
    let status = status_summary(&feature.metadata);
    if feature.default_enabled {
        format!("{}\\n{}, default", feature.name, status)
    } else {
        format!("{}\\n{}", feature.name, status)
    }
}

fn feature_node_id(package_prefix: Option<&str>, name: &str) -> String {
    prefixed_node_id(package_prefix, "feature", name)
}

fn reference_node_id(package_prefix: Option<&str>, reference: &FeatureRef) -> String {
    prefixed_node_id(package_prefix, "ref", &reference.to_string())
}

fn default_node_id(package_prefix: Option<&str>) -> String {
    match package_prefix {
        Some(prefix) => format!("default_{prefix}"),
        None => "default_".to_owned(),
    }
}

fn prefixed_node_id(package_prefix: Option<&str>, kind: &str, raw: &str) -> String {
    match package_prefix {
        Some(prefix) => format!("{kind}_{prefix}_{}", sanitize_id(raw)),
        None => format!("{kind}_{}", sanitize_id(raw)),
    }
}

fn sanitize_id(raw: &str) -> String {
    raw.chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() {
                character
            } else {
                '_'
            }
        })
        .collect()
}

fn status_summary(metadata: &FeatureMetadata) -> String {
    metadata.status_labels().join(", ")
}

fn visibility_label(metadata: &FeatureMetadata) -> &'static str {
    if metadata.public { "public" } else { "private" }
}

fn yes_no(value: bool) -> &'static str {
    if value { "yes" } else { "no" }
}

fn escape_markdown_inline(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('|', "\\|")
        .replace('\r', "")
        .replace('\n', "<br>")
}

fn escape_markdown_cell(value: &str) -> String {
    escape_markdown_inline(value)
}

fn escape_mermaid_label(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\r', "")
        .replace('\n', "\\n")
}
