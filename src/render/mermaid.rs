use std::collections::BTreeSet;

use crate::model::{Feature, FeatureManifest, FeatureRef, WorkspaceManifest};

use super::shared::{escape_mermaid_label, sanitize_id, status_summary};

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
