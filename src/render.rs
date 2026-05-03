use std::collections::BTreeSet;

use crate::manifest::{Feature, FeatureManifest, FeatureMetadata};

pub fn render_markdown(manifest: &FeatureManifest, include_private: bool) -> String {
    let mut lines = Vec::new();
    let title = manifest
        .package_name
        .as_deref()
        .map(|name| format!("# {name} feature manifest"))
        .unwrap_or_else(|| "# Feature manifest".to_owned());
    lines.push(title);
    lines.push(String::new());
    lines.push("| Feature | Default | Visibility | Status | Enables | Description |".to_owned());
    lines.push("| --- | --- | --- | --- | --- | --- |".to_owned());

    let mut hidden_count = 0usize;

    for feature in manifest.features.values() {
        if !include_private && !feature.metadata.public {
            hidden_count += 1;
            continue;
        }

        lines.push(format!(
            "| `{}` | {} | {} | {} | {} | {} |",
            feature.name,
            yes_no(feature.default_enabled),
            visibility_label(&feature.metadata),
            status_summary(&feature.metadata),
            dependency_summary(feature),
            description_summary(feature),
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
        lines.push("## Groups".to_owned());
        lines.push(String::new());

        for group in &manifest.groups {
            let mut line = format!(
                "- `{}`: {}",
                group.name,
                group
                    .description
                    .as_deref()
                    .unwrap_or("No description provided.")
            );
            if group.mutually_exclusive {
                line.push_str(" Mutually exclusive.");
            }
            line.push_str(&format!(
                " Members: {}.",
                group
                    .members
                    .iter()
                    .map(|member| format!("`{member}`"))
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
            lines.push(line);
        }
    }

    lines.join("\n")
}

pub fn render_mermaid(manifest: &FeatureManifest, include_private: bool) -> String {
    let visible_features = manifest
        .features
        .values()
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
        lines.push("    default_[\"default\"]".to_owned());
    }

    for feature in &visible_features {
        lines.push(format!(
            "    {}[\"{}\"]",
            feature_node_id(&feature.name),
            feature_label(feature),
        ));
    }

    let mut external_nodes = BTreeSet::new();

    for feature in &visible_features {
        if feature.default_enabled {
            lines.push(format!(
                "    default_ --> {}",
                feature_node_id(&feature.name)
            ));
        }

        for dependency in &feature.dependencies {
            if visible_names.contains(dependency.as_str()) {
                lines.push(format!(
                    "    {} --> {}",
                    feature_node_id(&feature.name),
                    feature_node_id(dependency)
                ));
                continue;
            }

            let external_id = reference_node_id(dependency);
            if external_nodes.insert(dependency.clone()) {
                lines.push(format!("    {}[\"{}\"]", external_id, dependency));
            }
            lines.push(format!(
                "    {} --> {}",
                feature_node_id(&feature.name),
                external_id
            ));
        }
    }

    lines.join("\n")
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

fn dependency_summary(feature: &Feature) -> String {
    if feature.dependencies.is_empty() {
        return "—".to_owned();
    }

    feature
        .dependencies
        .iter()
        .map(|dependency| format!("`{dependency}`"))
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

fn feature_node_id(name: &str) -> String {
    format!("feature_{}", sanitize_id(name))
}

fn reference_node_id(reference: &str) -> String {
    format!("ref_{}", sanitize_id(reference))
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
