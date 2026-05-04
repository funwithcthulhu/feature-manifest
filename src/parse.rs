use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, anyhow, bail};
use serde::Deserialize;
use toml_edit::{Array, DocumentMut, InlineTable, Item, Table, Value};

use crate::model::{
    DependencyInfo, Feature, FeatureGroup, FeatureManifest, FeatureMetadata, FeatureRef, LintLevel,
    LintPreset, MetadataLayout,
};

pub const FEATURE_MANIFEST_METADATA_TABLE: &str = "feature-manifest";
pub const FEATURE_DOCS_METADATA_TABLE: &str = "feature-docs";

/// Options controlling how `sync_manifest` rewrites metadata.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SyncOptions {
    pub check_only: bool,
    pub remove_stale: bool,
    pub style: Option<MetadataLayout>,
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

/// Non-writing result from a manifest synchronization preview.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SyncPreview {
    pub report: SyncReport,
    pub rewritten: Option<String>,
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
    #[serde(default)]
    dependencies: BTreeMap<String, RawDependency>,
    #[serde(default)]
    target: BTreeMap<String, RawTarget>,
}

#[derive(Debug, Deserialize)]
struct RawPackage {
    name: Option<String>,
    metadata: Option<toml::Table>,
}

#[derive(Debug, Deserialize)]
struct RawTarget {
    #[serde(default)]
    dependencies: BTreeMap<String, RawDependency>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
enum RawDependency {
    Version(String),
    Detailed(RawDependencyDetail),
}

#[derive(Debug, Clone, Deserialize)]
struct RawDependencyDetail {
    package: Option<String>,
    #[serde(default)]
    workspace: bool,
    optional: Option<bool>,
}

impl RawDependency {
    fn to_dependency_info(&self, key: &str) -> DependencyInfo {
        match self {
            Self::Version(version) => {
                let _ = version;
                DependencyInfo {
                    key: key.to_owned(),
                    package: key.to_owned(),
                    optional: false,
                }
            }
            Self::Detailed(details) => DependencyInfo {
                key: key.to_owned(),
                package: details.package.clone().unwrap_or_else(|| key.to_owned()),
                optional: details.optional.unwrap_or(details.workspace),
            },
        }
    }
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

    let (metadata_features, groups, metadata_table, metadata_layout, lint_overrides, lint_preset) =
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

    let dependencies = collect_manifest_dependency_info(&raw);
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
        dependencies,
        lint_overrides,
        lint_preset,
    })
}

fn collect_manifest_dependency_info(raw: &RawManifest) -> BTreeMap<String, DependencyInfo> {
    let mut dependencies = BTreeMap::new();

    for (key, dependency) in &raw.dependencies {
        dependencies.insert(key.clone(), dependency.to_dependency_info(key));
    }

    for target in raw.target.values() {
        for (key, dependency) in &target.dependencies {
            dependencies.insert(key.clone(), dependency.to_dependency_info(key));
        }
    }

    dependencies
}

/// Adds missing metadata scaffolding to a manifest in place.
pub fn sync_manifest(path: impl AsRef<Path>, options: &SyncOptions) -> Result<SyncReport> {
    let path = path.as_ref();
    let preview = preview_sync_manifest(path, options)?;

    if !options.check_only {
        if let Some(rewritten) = &preview.rewritten {
            fs::write(path, rewritten)
                .with_context(|| format!("failed to write manifest `{}`", path.display()))?;
        }
    }

    Ok(preview.report)
}

/// Computes the synchronization result and rewritten TOML without writing it.
pub fn preview_sync_manifest(path: impl AsRef<Path>, options: &SyncOptions) -> Result<SyncPreview> {
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

    let report = SyncReport {
        manifest_path: path.to_path_buf(),
        package_name: manifest.package_name.clone(),
        metadata_table: metadata_table.clone(),
        style,
        added_features,
        removed_features,
        would_change,
    };

    if !would_change {
        return Ok(SyncPreview {
            report,
            rewritten: None,
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
        &report.added_features,
        options.remove_stale,
    )?;

    Ok(SyncPreview {
        report,
        rewritten: Some(document.to_string()),
    })
}

/// Renders a compact unified diff for a manifest preview.
pub fn render_sync_diff(path: &Path, before: &str, after: &str) -> String {
    let path = path.display();
    let mut output = format!("--- a/{path}\n+++ b/{path}\n");
    let before_lines = before.lines().collect::<Vec<_>>();
    let after_lines = after.lines().collect::<Vec<_>>();

    output.push_str(&format!(
        "@@ -1,{} +1,{} @@\n",
        before_lines.len(),
        after_lines.len()
    ));

    for operation in diff_lines(&before_lines, &after_lines) {
        let (prefix, line) = match operation {
            DiffLine::Unchanged(line) => (' ', line),
            DiffLine::Removed(line) => ('-', line),
            DiffLine::Added(line) => ('+', line),
        };
        output.push(prefix);
        output.push_str(line);
        output.push('\n');
    }

    output
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DiffLine<'a> {
    Unchanged(&'a str),
    Removed(&'a str),
    Added(&'a str),
}

fn diff_lines<'a>(before: &'a [&'a str], after: &'a [&'a str]) -> Vec<DiffLine<'a>> {
    let before_len = before.len();
    let after_len = after.len();
    let mut lengths = vec![vec![0usize; after_len + 1]; before_len + 1];

    for before_index in (0..before_len).rev() {
        for after_index in (0..after_len).rev() {
            lengths[before_index][after_index] = if before[before_index] == after[after_index] {
                lengths[before_index + 1][after_index + 1] + 1
            } else {
                lengths[before_index + 1][after_index].max(lengths[before_index][after_index + 1])
            };
        }
    }

    let mut operations = Vec::new();
    let mut before_index = 0usize;
    let mut after_index = 0usize;

    while before_index < before_len || after_index < after_len {
        if before_index < before_len
            && after_index < after_len
            && before[before_index] == after[after_index]
        {
            operations.push(DiffLine::Unchanged(before[before_index]));
            before_index += 1;
            after_index += 1;
        } else if after_index < after_len
            && (before_index == before_len
                || lengths[before_index][after_index + 1] >= lengths[before_index + 1][after_index])
        {
            operations.push(DiffLine::Added(after[after_index]));
            after_index += 1;
        } else if before_index < before_len {
            operations.push(DiffLine::Removed(before[before_index]));
            before_index += 1;
        }
    }

    operations
}

type ExtractedMetadata = (
    BTreeMap<String, FeatureMetadata>,
    Vec<FeatureGroup>,
    Option<String>,
    MetadataLayout,
    BTreeMap<String, LintLevel>,
    Option<LintPreset>,
);

fn empty_metadata() -> ExtractedMetadata {
    (
        BTreeMap::new(),
        Vec::new(),
        None,
        MetadataLayout::Structured,
        BTreeMap::new(),
        None,
    )
}

fn extract_metadata(metadata: Option<&toml::Table>) -> Result<ExtractedMetadata> {
    let Some(metadata) = metadata else {
        return Ok(empty_metadata());
    };

    let (table_name, table_value) =
        if let Some(value) = metadata.get(FEATURE_MANIFEST_METADATA_TABLE) {
            (FEATURE_MANIFEST_METADATA_TABLE.to_owned(), value)
        } else if let Some(value) = metadata.get(FEATURE_DOCS_METADATA_TABLE) {
            (FEATURE_DOCS_METADATA_TABLE.to_owned(), value)
        } else {
            return Ok(empty_metadata());
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
    } else if table.iter().any(|(name, _)| {
        name != "groups" && name != "features" && name != "lints" && name != "preset"
    }) {
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
        if name == "features" || name == "groups" || name == "lints" || name == "preset" {
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

    let lint_preset = match table.get("preset") {
        Some(preset) => Some(
            preset
                .as_str()
                .ok_or_else(|| anyhow!("`preset` must be a string"))?
                .parse()?,
        ),
        None => None,
    };

    Ok((
        features,
        groups,
        Some(table_name),
        metadata_layout,
        lint_overrides,
        lint_preset,
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
    remove_stale: bool,
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

    if !remove_stale {
        feature_entries.extend(
            manifest
                .metadata_only
                .iter()
                .map(|(feature_name, metadata)| (feature_name.clone(), metadata.clone())),
        );
    }

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
            if name == "groups" || name == "features" || name == "lints" || name == "preset" {
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
    if let Some(category) = &metadata.category {
        inline.insert("category", Value::from(category.clone()));
    }
    if let Some(since) = &metadata.since {
        inline.insert("since", Value::from(since.clone()));
    }
    if let Some(docs) = &metadata.docs {
        inline.insert("docs", Value::from(docs.clone()));
    }
    if let Some(tracking_issue) = &metadata.tracking_issue {
        inline.insert("tracking_issue", Value::from(tracking_issue.clone()));
    }
    if !metadata.requires.is_empty() {
        let mut requires = Array::new();
        for requirement in &metadata.requires {
            requires.push(requirement.as_str());
        }
        inline.insert("requires", Value::Array(requires));
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
