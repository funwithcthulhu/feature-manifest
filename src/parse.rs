use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, anyhow, bail};
use serde::Deserialize;
use toml_edit::{DocumentMut, InlineTable, Item, Table, Value};

use crate::model::{
    Feature, FeatureGroup, FeatureManifest, FeatureMetadata, FeatureRef, LintLevel, MetadataLayout,
};

pub const FEATURE_MANIFEST_METADATA_TABLE: &str = "feature-manifest";
pub const FEATURE_DOCS_METADATA_TABLE: &str = "feature-docs";

/// Options controlling how `sync_manifest` rewrites metadata.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SyncOptions {
    pub check_only: bool,
    pub remove_stale: bool,
    pub style: Option<MetadataLayout>,
}

impl Default for SyncOptions {
    fn default() -> Self {
        Self {
            check_only: false,
            remove_stale: false,
            style: None,
        }
    }
}

/// Summary of a manifest synchronization pass.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SyncReport {
    pub manifest_path: PathBuf,
    pub package_name: Option<String>,
    pub metadata_table: String,
    pub style: MetadataLayout,
    pub added_features: Vec<String>,
    pub removed_features: Vec<String>,
    pub would_change: bool,
}

impl SyncReport {
    pub fn changed(&self) -> bool {
        self.would_change
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

    let (metadata_features, groups, metadata_table, metadata_layout, lint_overrides) =
        extract_metadata(
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
        metadata_layout,
        features,
        metadata_only,
        default_members,
        default_features,
        groups,
        dependencies: BTreeMap::new(),
        lint_overrides,
    })
}

/// Adds missing metadata scaffolding to a manifest in place.
pub fn sync_manifest(path: impl AsRef<Path>, options: &SyncOptions) -> Result<SyncReport> {
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

    let mut removed_features = if options.remove_stale {
        manifest.metadata_only.keys().cloned().collect::<Vec<_>>()
    } else {
        Vec::new()
    };
    removed_features.sort();

    let metadata_table = manifest
        .metadata_table
        .clone()
        .unwrap_or_else(|| FEATURE_MANIFEST_METADATA_TABLE.to_owned());
    let style = options.style.unwrap_or(manifest.metadata_layout);

    let would_change = !added_features.is_empty()
        || !removed_features.is_empty()
        || options
            .style
            .is_some_and(|requested| requested != manifest.metadata_layout);

    if !would_change || options.check_only {
        return Ok(SyncReport {
            manifest_path: path.to_path_buf(),
            package_name: manifest.package_name,
            metadata_table,
            style,
            added_features,
            removed_features,
            would_change,
        });
    }

    let mut document = contents.parse::<DocumentMut>().with_context(|| {
        format!(
            "failed to parse TOML document for synchronization from `{}`",
            path.display()
        )
    })?;

    rewrite_feature_metadata(
        &mut document,
        &manifest,
        &metadata_table,
        style,
        &added_features,
    )?;

    fs::write(path, document.to_string())
        .with_context(|| format!("failed to write manifest `{}`", path.display()))?;

    Ok(SyncReport {
        manifest_path: path.to_path_buf(),
        package_name: manifest.package_name,
        metadata_table,
        style,
        added_features,
        removed_features,
        would_change,
    })
}

fn extract_metadata(
    metadata: Option<&toml::Table>,
) -> Result<(
    BTreeMap<String, FeatureMetadata>,
    Vec<FeatureGroup>,
    Option<String>,
    MetadataLayout,
    BTreeMap<String, LintLevel>,
)> {
    let Some(metadata) = metadata else {
        return Ok((
            BTreeMap::new(),
            Vec::new(),
            None,
            MetadataLayout::Structured,
            BTreeMap::new(),
        ));
    };

    let (table_name, table_value) =
        if let Some(value) = metadata.get(FEATURE_MANIFEST_METADATA_TABLE) {
            (FEATURE_MANIFEST_METADATA_TABLE.to_owned(), value)
        } else if let Some(value) = metadata.get(FEATURE_DOCS_METADATA_TABLE) {
            (FEATURE_DOCS_METADATA_TABLE.to_owned(), value)
        } else {
            return Ok((
                BTreeMap::new(),
                Vec::new(),
                None,
                MetadataLayout::Structured,
                BTreeMap::new(),
            ));
        };

    let table = table_value.as_table().ok_or_else(|| {
        anyhow!("`[package.metadata.{table_name}]` must be a TOML table, not a scalar value")
    })?;

    let metadata_layout = if table
        .get("features")
        .and_then(|item| item.as_table())
        .is_some()
    {
        MetadataLayout::Structured
    } else if table
        .iter()
        .any(|(name, _)| name != "groups" && name != "features" && name != "lints")
    {
        MetadataLayout::Flat
    } else {
        MetadataLayout::Structured
    };

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
        if name == "features" || name == "groups" || name == "lints" {
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

    let lint_overrides = match table.get("lints") {
        Some(lints) => lints
            .clone()
            .try_into()
            .context("`lints` must be a table of lint names to levels")?,
        None => BTreeMap::new(),
    };

    Ok((
        features,
        groups,
        Some(table_name),
        metadata_layout,
        lint_overrides,
    ))
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

fn rewrite_feature_metadata(
    document: &mut DocumentMut,
    manifest: &FeatureManifest,
    metadata_table_name: &str,
    style: MetadataLayout,
    added_features: &[String],
) -> Result<()> {
    let package_table = ensure_child_table(document.as_table_mut(), "package")?;
    let metadata_parent = ensure_child_table(package_table, "metadata")?;
    let feature_manifest_table = ensure_child_table(metadata_parent, metadata_table_name)?;

    let mut feature_entries = manifest
        .features
        .values()
        .filter(|feature| feature.has_metadata)
        .map(|feature| (feature.name.clone(), feature.metadata.clone()))
        .collect::<BTreeMap<_, _>>();

    for feature_name in added_features {
        feature_entries.insert(
            feature_name.clone(),
            FeatureMetadata {
                description: Some(format!("TODO: describe `{feature_name}`.")),
                ..FeatureMetadata::default()
            },
        );
    }

    remove_existing_feature_metadata(feature_manifest_table)?;

    match style {
        MetadataLayout::Flat => {
            feature_manifest_table.remove("features");
            for (feature_name, metadata) in &feature_entries {
                feature_manifest_table.insert(
                    feature_name,
                    Item::Value(metadata_to_inline_value(metadata, feature_name)),
                );
            }
        }
        MetadataLayout::Structured => {
            let features_table = ensure_child_table(feature_manifest_table, "features")?;
            for (feature_name, metadata) in &feature_entries {
                features_table.insert(
                    feature_name,
                    Item::Value(metadata_to_inline_value(metadata, feature_name)),
                );
            }
        }
    }

    Ok(())
}

fn remove_existing_feature_metadata(table: &mut Table) -> Result<()> {
    let feature_keys = table
        .iter()
        .filter_map(|(name, _)| {
            if name == "groups" || name == "features" || name == "lints" {
                None
            } else {
                Some(name.to_owned())
            }
        })
        .collect::<Vec<_>>();

    for key in feature_keys {
        table.remove(&key);
    }

    if let Some(features_item) = table.get_mut("features") {
        let features_table = features_item
            .as_table_mut()
            .ok_or_else(|| anyhow!("expected `features` to be a TOML table while editing"))?;
        let nested_keys = features_table
            .iter()
            .map(|(name, _)| name.to_owned())
            .collect::<Vec<_>>();
        for key in nested_keys {
            features_table.remove(&key);
        }
    }

    Ok(())
}

fn metadata_to_inline_value(metadata: &FeatureMetadata, feature_name: &str) -> Value {
    let mut inline = InlineTable::new();
    inline.insert(
        "description",
        Value::from(
            metadata
                .description
                .clone()
                .unwrap_or_else(|| format!("TODO: describe `{feature_name}`.")),
        ),
    );

    if !metadata.public {
        inline.insert("public", Value::from(false));
    }
    if metadata.unstable {
        inline.insert("unstable", Value::from(true));
    }
    if metadata.deprecated {
        inline.insert("deprecated", Value::from(true));
    }
    if metadata.allow_default {
        inline.insert("allow_default", Value::from(true));
    }
    if let Some(note) = &metadata.note {
        inline.insert("note", Value::from(note.clone()));
    }

    Value::InlineTable(inline)
}

fn ensure_child_table<'a>(parent: &'a mut Table, key: &str) -> Result<&'a mut Table> {
    if !parent.contains_key(key) {
        parent.insert(key, Item::Table(Table::new()));
    }

    parent[key]
        .as_table_mut()
        .ok_or_else(|| anyhow!("expected `{key}` to be a TOML table while editing the manifest"))
}
