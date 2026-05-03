use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, anyhow, bail};
use serde::{Deserialize, Serialize};

pub const FEATURE_MANIFEST_METADATA_TABLE: &str = "feature-manifest";
pub const FEATURE_DOCS_METADATA_TABLE: &str = "feature-docs";

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct FeatureManifest {
    pub manifest_path: PathBuf,
    pub package_name: Option<String>,
    pub metadata_table: Option<String>,
    pub features: BTreeMap<String, Feature>,
    pub metadata_only: BTreeMap<String, FeatureMetadata>,
    pub default_features: BTreeSet<String>,
    pub groups: Vec<FeatureGroup>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Feature {
    pub name: String,
    pub metadata: FeatureMetadata,
    pub has_metadata: bool,
    pub dependencies: Vec<String>,
    pub default_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct FeatureGroup {
    pub name: String,
    pub members: Vec<String>,
    #[serde(default)]
    pub mutually_exclusive: bool,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct FeatureMetadata {
    pub description: Option<String>,
    #[serde(default = "default_public")]
    pub public: bool,
    #[serde(default)]
    pub unstable: bool,
    #[serde(default)]
    pub deprecated: bool,
    #[serde(default)]
    pub allow_default: bool,
    pub note: Option<String>,
}

impl Default for FeatureMetadata {
    fn default() -> Self {
        Self {
            description: None,
            public: true,
            unstable: false,
            deprecated: false,
            allow_default: false,
            note: None,
        }
    }
}

impl FeatureMetadata {
    pub fn status_labels(&self) -> Vec<&'static str> {
        let mut labels = Vec::new();
        if self.deprecated {
            labels.push("deprecated");
        }
        if self.unstable {
            labels.push("unstable");
        }
        if !self.public {
            labels.push("private");
        }
        if labels.is_empty() {
            labels.push("stable");
        }
        labels
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
enum RawFeatureMetadata {
    Description(String),
    Detailed(FeatureMetadata),
}

impl RawFeatureMetadata {
    fn into_metadata(self) -> FeatureMetadata {
        match self {
            Self::Description(description) => FeatureMetadata {
                description: Some(description),
                ..FeatureMetadata::default()
            },
            Self::Detailed(metadata) => metadata,
        }
    }
}

#[derive(Debug, Deserialize)]
struct RawManifest {
    package: Option<RawPackage>,
    #[serde(default)]
    features: BTreeMap<String, Vec<String>>,
}

#[derive(Debug, Deserialize)]
struct RawPackage {
    name: Option<String>,
    metadata: Option<toml::Table>,
}

pub fn resolve_manifest_path(path: Option<&Path>) -> Result<PathBuf> {
    let base_path = match path {
        Some(path) => path.to_path_buf(),
        None => std::env::current_dir()
            .context("failed to determine the current directory")?
            .join("Cargo.toml"),
    };

    let manifest_path = if base_path.is_dir() {
        base_path.join("Cargo.toml")
    } else {
        base_path
    };

    if !manifest_path.exists() {
        bail!("could not find Cargo.toml at `{}`", manifest_path.display());
    }

    Ok(manifest_path)
}

pub fn load_manifest(path: impl AsRef<Path>) -> Result<FeatureManifest> {
    let path = path.as_ref();
    let contents = fs::read_to_string(path)
        .with_context(|| format!("failed to read manifest `{}`", path.display()))?;
    parse_manifest_str(&contents, path)
}

pub fn parse_manifest_str(
    manifest_source: &str,
    manifest_path: impl Into<PathBuf>,
) -> Result<FeatureManifest> {
    let manifest_path = manifest_path.into();
    let raw: RawManifest = toml::from_str(manifest_source).with_context(|| {
        format!(
            "failed to parse manifest TOML from `{}`",
            manifest_path.display()
        )
    })?;

    let default_features = raw
        .features
        .get("default")
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .collect::<BTreeSet<_>>();

    let (metadata_features, groups, metadata_table) = extract_metadata(
        raw.package
            .as_ref()
            .and_then(|package| package.metadata.as_ref()),
    )
    .with_context(|| {
        format!(
            "failed to parse feature metadata from `{}`",
            manifest_path.display()
        )
    })?;

    let package_name = raw.package.and_then(|package| package.name);
    let mut metadata_only = metadata_features.clone();
    let mut features = BTreeMap::new();

    for (name, dependencies) in raw.features {
        if name == "default" {
            continue;
        }

        let metadata = metadata_only.remove(&name).unwrap_or_default();
        let has_metadata = metadata_features.contains_key(&name);
        let default_enabled = default_features.contains(&name);

        features.insert(
            name.clone(),
            Feature {
                name,
                metadata,
                has_metadata,
                dependencies,
                default_enabled,
            },
        );
    }

    Ok(FeatureManifest {
        manifest_path,
        package_name,
        metadata_table,
        features,
        metadata_only,
        default_features,
        groups,
    })
}

fn extract_metadata(
    metadata: Option<&toml::Table>,
) -> Result<(
    BTreeMap<String, FeatureMetadata>,
    Vec<FeatureGroup>,
    Option<String>,
)> {
    let Some(metadata) = metadata else {
        return Ok((BTreeMap::new(), Vec::new(), None));
    };

    let (table_name, table_value) =
        if let Some(value) = metadata.get(FEATURE_MANIFEST_METADATA_TABLE) {
            (FEATURE_MANIFEST_METADATA_TABLE.to_owned(), value)
        } else if let Some(value) = metadata.get(FEATURE_DOCS_METADATA_TABLE) {
            (FEATURE_DOCS_METADATA_TABLE.to_owned(), value)
        } else {
            return Ok((BTreeMap::new(), Vec::new(), None));
        };

    let table = table_value.as_table().ok_or_else(|| {
        anyhow!("`[package.metadata.{table_name}]` must be a TOML table, not a scalar value")
    })?;

    let mut features = BTreeMap::new();

    if let Some(structured_features) = table.get("features") {
        let structured_features = structured_features.as_table().ok_or_else(|| {
            anyhow!("`[package.metadata.{table_name}.features]` must be a TOML table")
        })?;

        for (name, value) in structured_features {
            insert_feature_metadata(&mut features, name, value, &table_name)?;
        }
    }

    for (name, value) in table {
        if name == "features" || name == "groups" {
            continue;
        }

        insert_feature_metadata(&mut features, name, value, &table_name)?;
    }

    let groups = match table.get("groups") {
        Some(groups) => groups
            .clone()
            .try_into()
            .context("`groups` must be an array of tables")?,
        None => Vec::new(),
    };

    Ok((features, groups, Some(table_name)))
}

fn insert_feature_metadata(
    features: &mut BTreeMap<String, FeatureMetadata>,
    name: &str,
    value: &toml::Value,
    table_name: &str,
) -> Result<()> {
    let raw_metadata: RawFeatureMetadata = value.clone().try_into().with_context(|| {
        format!("feature `{name}` in `[package.metadata.{table_name}]` must be a string or table")
    })?;
    let metadata = raw_metadata.into_metadata();

    if features.insert(name.to_owned(), metadata).is_some() {
        bail!("feature `{name}` is defined more than once in `[package.metadata.{table_name}]`");
    }

    Ok(())
}

fn default_public() -> bool {
    true
}
