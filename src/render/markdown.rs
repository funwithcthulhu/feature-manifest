use crate::model::{Feature, FeatureManifest, WorkspaceManifest};

use super::shared::{
    escape_markdown_cell, escape_markdown_inline, reference_summary, status_summary,
    visibility_label, yes_no,
};

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
    lines.push(
        "| Feature | Default | Visibility | Status | Category | Enables | Description |".to_owned(),
    );
    lines.push("| --- | --- | --- | --- | --- | --- | --- |".to_owned());

    let mut hidden_count = 0usize;

    for feature in manifest.ordered_features() {
        if !include_private && !feature.metadata.public {
            hidden_count += 1;
            continue;
        }

        lines.push(format!(
            "| `{}` | {} | {} | {} | {} | {} | {} |",
            escape_markdown_inline(&feature.name),
            yes_no(feature.default_enabled),
            visibility_label(&feature.metadata),
            escape_markdown_inline(&status_summary(&feature.metadata)),
            escape_markdown_cell(feature.metadata.category.as_deref().unwrap_or("—")),
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

    let mut details = Vec::new();
    if let Some(note) = &feature.metadata.note {
        details.push(format!("Note: {note}"));
    }
    if let Some(since) = &feature.metadata.since {
        details.push(format!("Since: {since}"));
    }
    if let Some(docs) = &feature.metadata.docs {
        details.push(format!("Docs: {docs}"));
    }
    if let Some(tracking_issue) = &feature.metadata.tracking_issue {
        details.push(format!("Tracking issue: {tracking_issue}"));
    }
    if !feature.metadata.requires.is_empty() {
        details.push(format!(
            "Requires: {}",
            feature.metadata.requires.join(", ")
        ));
    }

    if details.is_empty() {
        description.to_owned()
    } else {
        format!("{description} {}", details.join(" "))
    }
}
