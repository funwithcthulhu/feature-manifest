use std::path::{Path, PathBuf};

use serde::Serialize;

use crate::model::{
    DependencyInfo, FeatureGroup, FeatureManifest, FeatureMetadata, FeatureRef, LintLevel,
    LintPreset, MetadataLayout, WorkspaceManifest,
};

const JSON_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Serialize)]
pub struct JsonReport {
    pub schema_version: u32,
    pub packages: Vec<JsonPackage>,
}

#[derive(Debug, Serialize)]
pub struct JsonPackage {
    pub package_name: Option<String>,
    pub manifest_path: String,
    pub metadata_table: Option<String>,
    pub metadata_layout: MetadataLayout,
    pub default_feature_set: Vec<String>,
    pub dependencies: Vec<JsonDependency>,
    pub lint_overrides: Vec<JsonLintOverride>,
    pub lint_preset: Option<LintPreset>,
    pub features: Vec<JsonFeature>,
    pub groups: Vec<JsonGroup>,
}

#[derive(Debug, Serialize)]
pub struct JsonFeature {
    pub name: String,
    pub default_enabled: bool,
    pub has_metadata: bool,
    pub enables: Vec<JsonFeatureRef>,
    pub groups: Vec<String>,
    pub metadata: FeatureMetadata,
}

#[derive(Debug, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum JsonFeatureRef {
    Feature {
        name: String,
    },
    Dependency {
        name: String,
    },
    DependencyFeature {
        dependency: String,
        feature: String,
        weak: bool,
    },
    Unknown {
        raw: String,
    },
}

#[derive(Debug, Serialize)]
pub struct JsonGroup {
    pub name: String,
    pub description: Option<String>,
    pub mutually_exclusive: bool,
    pub members: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct JsonDependency {
    pub key: String,
    pub package: String,
    pub optional: bool,
}

#[derive(Debug, Serialize)]
pub struct JsonLintOverride {
    pub code: String,
    pub level: LintLevel,
}

/// Renders the selected workspace/package metadata as pretty-printed JSON.
pub fn render_json(workspace: &WorkspaceManifest) -> Result<String, serde_json::Error> {
    serde_json::to_string_pretty(&workspace_to_json(workspace))
}

fn workspace_to_json(workspace: &WorkspaceManifest) -> JsonReport {
    let root_directory = workspace
        .root_manifest_path
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("."));

    JsonReport {
        schema_version: JSON_SCHEMA_VERSION,
        packages: workspace
            .packages
            .iter()
            .map(|package| package_to_json(&root_directory, package))
            .collect(),
    }
}

fn package_to_json(root_directory: &Path, package: &FeatureManifest) -> JsonPackage {
    JsonPackage {
        package_name: package.package_name.clone(),
        manifest_path: portable_relative_path(root_directory, &package.manifest_path),
        metadata_table: package.metadata_table.clone(),
        metadata_layout: package.metadata_layout,
        default_feature_set: package
            .default_members
            .iter()
            .map(FeatureRef::raw)
            .collect(),
        dependencies: package
            .dependencies
            .values()
            .map(dependency_to_json)
            .collect(),
        lint_overrides: package
            .lint_overrides
            .iter()
            .map(|(code, level)| JsonLintOverride {
                code: code.clone(),
                level: *level,
            })
            .collect(),
        lint_preset: package.lint_preset,
        features: package
            .ordered_features()
            .into_iter()
            .map(|feature| JsonFeature {
                name: feature.name.clone(),
                default_enabled: feature.default_enabled,
                has_metadata: feature.has_metadata,
                enables: feature.enables.iter().map(feature_ref_to_json).collect(),
                groups: package
                    .groups_for_feature(&feature.name)
                    .into_iter()
                    .map(|group| group.name.clone())
                    .collect(),
                metadata: feature.metadata.clone(),
            })
            .collect(),
        groups: package.groups.iter().map(group_to_json).collect(),
    }
}

fn dependency_to_json(dependency: &DependencyInfo) -> JsonDependency {
    JsonDependency {
        key: dependency.key.clone(),
        package: dependency.package.clone(),
        optional: dependency.optional,
    }
}

fn feature_ref_to_json(reference: &FeatureRef) -> JsonFeatureRef {
    match reference {
        FeatureRef::Feature { name } => JsonFeatureRef::Feature { name: name.clone() },
        FeatureRef::Dependency { name } => JsonFeatureRef::Dependency { name: name.clone() },
        FeatureRef::DependencyFeature {
            dependency,
            feature,
            weak,
        } => JsonFeatureRef::DependencyFeature {
            dependency: dependency.clone(),
            feature: feature.clone(),
            weak: *weak,
        },
        FeatureRef::Unknown { raw } => JsonFeatureRef::Unknown { raw: raw.clone() },
    }
}

fn group_to_json(group: &FeatureGroup) -> JsonGroup {
    JsonGroup {
        name: group.name.clone(),
        description: group.description.clone(),
        mutually_exclusive: group.mutually_exclusive,
        members: group.members.clone(),
    }
}

fn portable_relative_path(root_directory: &Path, path: &Path) -> String {
    let relative = path.strip_prefix(root_directory).unwrap_or(path);
    relative
        .components()
        .map(|component| component.as_os_str().to_string_lossy().into_owned())
        .collect::<Vec<_>>()
        .join("/")
}
