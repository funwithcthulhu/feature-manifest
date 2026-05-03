use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, anyhow, bail};
use serde::Deserialize;
use toml_edit::{DocumentMut, InlineTable, Item, Table, Value};

use crate::model::{Feature, FeatureGroup, FeatureManifest, FeatureMetadata, FeatureRef};

pub const FEATURE_MANIFEST_METADATA_TABLE: &str = "feature-manifest";
pub const FEATURE_DOCS_METADATA_TABLE: &str = "feature-docs";

/// Summary of a manifest synchronization pass.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SyncReport {
    pub manifest_path: PathBuf,
    pub package_name: Option<String>,
    pub metadata_table: String,
    pub added_features: Vec<String>,
}

impl SyncReport {
    pub fn changed(&self) -> bool {
        !self.added_features.is_empty()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SyncLayout {
    Flat,
    Structured,
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

/// Loads and parses a manifest from disk.
pub fn load_manifest(path: impl AsRef<Path>) -> Result<FeatureManifest> {
    let path = path.as_ref();
    let contents = fs::read_to_string(path)
        .with_context(|| format!("failed to read manifest `{}`", path.display()))?;
    parse_manifest_str(&contents, path)
}

/// Parses a manifest from a TOML string and normalizes its feature metadata.
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

    let default_members = raw
        .features
        .get("default")
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .map(|value| FeatureRef::parse(&value))
        .collect::<Vec<_>>();
    let default_features = default_members
        .iter()
        .filter_map(FeatureRef::local_feature_name)
        .map(str::to_owned)
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

    for (name, entries) in raw.features {
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
                enables: entries
                    .into_iter()
                    .map(|entry| FeatureRef::parse(&entry))
                    .collect(),
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
        default_members,
        default_features,
        groups,
    })
}

/// Adds missing metadata scaffolding to a manifest in place.
pub fn sync_manifest(path: impl AsRef<Path>) -> Result<SyncReport> {
    let path = path.as_ref();
    let contents = fs::read_to_string(path)
        .with_context(|| format!("failed to read manifest `{}`", path.display()))?;
    let manifest = parse_manifest_str(&contents, path)?;

    let mut added_features = manifest
        .features
        .values()
        .filter(|feature| !feature.has_metadata)
        .map(|feature| feature.name.clone())
        .collect::<Vec<_>>();
    added_features.sort();

    let metadata_table = manifest
        .metadata_table
        .clone()
        .unwrap_or_else(|| FEATURE_MANIFEST_METADATA_TABLE.to_owned());

    if added_features.is_empty() {
        return Ok(SyncReport {
            manifest_path: path.to_path_buf(),
            package_name: manifest.package_name,
            metadata_table,
            added_features,
        });
    }

    let mut document = contents.parse::<DocumentMut>().with_context(|| {
        format!(
            "failed to parse TOML document for synchronization from `{}`",
            path.display()
        )
    })?;

    let package_table = ensure_child_table(document.as_table_mut(), "package")?;
    let metadata_parent = ensure_child_table(package_table, "metadata")?;
    let feature_manifest_table = ensure_child_table(metadata_parent, &metadata_table)?;
    let layout = detect_sync_layout(feature_manifest_table);
    let target_table = match layout {
        SyncLayout::Flat => feature_manifest_table,
        SyncLayout::Structured => ensure_child_table(feature_manifest_table, "features")?,
    };

    for feature_name in &added_features {
        insert_scaffold_entry(target_table, feature_name);
    }

    fs::write(path, document.to_string())
        .with_context(|| format!("failed to write manifest `{}`", path.display()))?;

    Ok(SyncReport {
        manifest_path: path.to_path_buf(),
        package_name: manifest.package_name,
        metadata_table,
        added_features,
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

fn ensure_child_table<'a>(parent: &'a mut Table, key: &str) -> Result<&'a mut Table> {
    if !parent.contains_key(key) {
        parent.insert(key, Item::Table(Table::new()));
    }

    parent[key]
        .as_table_mut()
        .ok_or_else(|| anyhow!("expected `{key}` to be a TOML table while editing the manifest"))
}

fn detect_sync_layout(table: &Table) -> SyncLayout {
    if table
        .get("features")
        .and_then(Item::as_table)
        .is_some_and(|_| true)
    {
        return SyncLayout::Structured;
    }

    if table
        .iter()
        .any(|(name, _)| name != "groups" && name != "features")
    {
        SyncLayout::Flat
    } else {
        SyncLayout::Structured
    }
}

fn insert_scaffold_entry(table: &mut Table, feature_name: &str) {
    let mut inline = InlineTable::new();
    inline.insert(
        "description",
        Value::from(format!("TODO: describe `{feature_name}`.")),
    );
    table.insert(feature_name, Item::Value(Value::InlineTable(inline)));
}
