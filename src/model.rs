use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::path::PathBuf;
use std::str::FromStr;

use anyhow::{Result, bail};
use serde::{Deserialize, Serialize};

/// A workspace-aware view of one or more Cargo packages selected for analysis.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkspaceManifest {
    /// Root workspace manifest used as the base for relative output paths.
    pub root_manifest_path: PathBuf,
    /// Selected package manifests in deterministic display order.
    pub packages: Vec<FeatureManifest>,
}

impl WorkspaceManifest {
    /// Returns `true` when exactly one package was selected.
    pub fn is_single_package(&self) -> bool {
        self.packages.len() == 1
    }

    /// Returns the selected package names in display order.
    pub fn package_names(&self) -> Vec<&str> {
        self.packages
            .iter()
            .filter_map(|package| package.package_name.as_deref())
            .collect()
    }
}

/// A normalized view of Cargo features plus structured feature metadata.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct FeatureManifest {
    /// Path to the package manifest that was parsed.
    pub manifest_path: PathBuf,
    /// Cargo package name, when the manifest has a `[package]` section.
    pub package_name: Option<String>,
    /// Metadata table that was found, such as `feature-manifest`.
    pub metadata_table: Option<String>,
    /// Metadata layout used by the manifest.
    pub metadata_layout: MetadataLayout,
    /// Declared Cargo features keyed by feature name.
    pub features: BTreeMap<String, Feature>,
    /// Metadata entries that do not match a declared feature.
    pub metadata_only: BTreeMap<String, FeatureMetadata>,
    /// Raw typed members from the `default` feature set.
    pub default_members: Vec<FeatureRef>,
    /// Local feature names enabled by the `default` feature set.
    pub default_features: BTreeSet<String>,
    /// Feature groups declared in metadata.
    pub groups: Vec<FeatureGroup>,
    /// Dependencies known from the manifest or `cargo metadata`.
    pub dependencies: BTreeMap<String, DependencyInfo>,
    /// Manifest-defined lint level overrides.
    pub lint_overrides: BTreeMap<String, LintLevel>,
    /// Manifest-defined lint preset.
    pub lint_preset: Option<LintPreset>,
}

impl FeatureManifest {
    /// Returns the features in stable display order.
    pub fn ordered_features(&self) -> Vec<&Feature> {
        self.features.values().collect()
    }

    /// Returns the groups that contain the named feature.
    pub fn groups_for_feature(&self, feature_name: &str) -> Vec<&FeatureGroup> {
        self.groups
            .iter()
            .filter(|group| group.members.iter().any(|member| member == feature_name))
            .collect()
    }

    /// Returns the features that directly reference the named feature.
    pub fn reverse_dependencies(&self, feature_name: &str) -> Vec<&Feature> {
        self.features
            .values()
            .filter(|feature| {
                feature
                    .enables
                    .iter()
                    .any(|reference| reference.local_feature_name() == Some(feature_name))
            })
            .collect()
    }
}

/// A single Cargo feature and its associated metadata.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Feature {
    /// Cargo feature name.
    pub name: String,
    /// Metadata associated with the feature, or defaults if missing.
    pub metadata: FeatureMetadata,
    /// Whether the feature had an explicit metadata entry.
    pub has_metadata: bool,
    /// Feature references activated by this feature.
    pub enables: Vec<FeatureRef>,
    /// Whether this feature is included in the default feature set.
    pub default_enabled: bool,
}

/// Dependency details relevant to feature validation.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct DependencyInfo {
    /// Dependency key used in `Cargo.toml`.
    pub key: String,
    /// Underlying package name after rename handling.
    pub package: String,
    /// Whether Cargo marks the dependency as optional.
    pub optional: bool,
}

/// Layout used for feature metadata inside `package.metadata`.
#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MetadataLayout {
    /// Feature metadata is stored directly under `[package.metadata.feature-manifest]`.
    Flat,
    /// Feature metadata is stored under `[package.metadata.feature-manifest.features]`.
    Structured,
}

impl fmt::Display for MetadataLayout {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Flat => formatter.write_str("flat"),
            Self::Structured => formatter.write_str("structured"),
        }
    }
}

impl FromStr for MetadataLayout {
    type Err = anyhow::Error;

    fn from_str(value: &str) -> Result<Self> {
        match value {
            "flat" => Ok(Self::Flat),
            "structured" => Ok(Self::Structured),
            _ => bail!("expected `flat` or `structured`, found `{value}`"),
        }
    }
}

/// Severity override policy for a specific lint code.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum LintLevel {
    /// Suppress the lint.
    Allow,
    /// Report the lint without failing validation.
    Warn,
    /// Report the lint as an error.
    Deny,
}

impl fmt::Display for LintLevel {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Allow => formatter.write_str("allow"),
            Self::Warn => formatter.write_str("warn"),
            Self::Deny => formatter.write_str("deny"),
        }
    }
}

impl FromStr for LintLevel {
    type Err = anyhow::Error;

    fn from_str(value: &str) -> Result<Self> {
        match value {
            "allow" => Ok(Self::Allow),
            "warn" | "warning" => Ok(Self::Warn),
            "deny" | "error" => Ok(Self::Deny),
            _ => bail!("expected `allow`, `warn`, or `deny`, found `{value}`"),
        }
    }
}

/// A named lint policy intended to make adoption and strict CI setup easier.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum LintPreset {
    /// Downgrade common rollout issues while adopting metadata.
    Adopt,
    /// Treat subjective warnings as release-blocking errors.
    Strict,
}

impl fmt::Display for LintPreset {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Adopt => formatter.write_str("adopt"),
            Self::Strict => formatter.write_str("strict"),
        }
    }
}

impl FromStr for LintPreset {
    type Err = anyhow::Error;

    fn from_str(value: &str) -> Result<Self> {
        match value {
            "adopt" => Ok(Self::Adopt),
            "strict" => Ok(Self::Strict),
            _ => bail!("expected `adopt` or `strict`, found `{value}`"),
        }
    }
}

/// A typed reference inside a feature definition.
#[derive(Debug, Clone, Serialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum FeatureRef {
    /// Reference to another local Cargo feature.
    Feature {
        /// Referenced feature name.
        name: String,
    },
    /// Reference to an optional dependency using `dep:name`.
    Dependency {
        /// Dependency key.
        name: String,
    },
    /// Reference to a dependency feature such as `tokio/rt`.
    DependencyFeature {
        /// Dependency key.
        dependency: String,
        /// Dependency feature name.
        feature: String,
        /// Whether the reference uses Cargo's weak `name?/feature` syntax.
        weak: bool,
    },
    /// Reference syntax that could not be classified.
    Unknown {
        /// Original reference text.
        raw: String,
    },
}

impl FeatureRef {
    /// Parses a raw Cargo feature entry into a typed reference.
    pub fn parse(raw: &str) -> Self {
        if let Some(name) = raw.strip_prefix("dep:") {
            return Self::Dependency {
                name: name.to_owned(),
            };
        }

        if let Some((dependency, feature)) = raw.split_once("?/") {
            return Self::DependencyFeature {
                dependency: dependency.to_owned(),
                feature: feature.to_owned(),
                weak: true,
            };
        }

        if let Some((dependency, feature)) = raw.split_once('/') {
            return Self::DependencyFeature {
                dependency: dependency.to_owned(),
                feature: feature.to_owned(),
                weak: false,
            };
        }

        if raw.trim().is_empty() {
            return Self::Unknown {
                raw: raw.to_owned(),
            };
        }

        Self::Feature {
            name: raw.to_owned(),
        }
    }

    /// Returns the local feature name when this is a local feature reference.
    pub fn local_feature_name(&self) -> Option<&str> {
        match self {
            Self::Feature { name } => Some(name.as_str()),
            _ => None,
        }
    }

    /// Returns the dependency key for dependency-based references.
    pub fn dependency_name(&self) -> Option<&str> {
        match self {
            Self::Dependency { name } => Some(name.as_str()),
            Self::DependencyFeature { dependency, .. } => Some(dependency.as_str()),
            _ => None,
        }
    }

    /// Returns the reference in Cargo feature syntax.
    pub fn raw(&self) -> String {
        self.to_string()
    }
}

impl fmt::Display for FeatureRef {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Feature { name } => formatter.write_str(name),
            Self::Dependency { name } => write!(formatter, "dep:{name}"),
            Self::DependencyFeature {
                dependency,
                feature,
                weak,
            } => {
                if *weak {
                    write!(formatter, "{dependency}?/{feature}")
                } else {
                    write!(formatter, "{dependency}/{feature}")
                }
            }
            Self::Unknown { raw } => formatter.write_str(raw),
        }
    }
}

/// A logical grouping of related features.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct FeatureGroup {
    /// Group identifier shown in reports and generated output.
    pub name: String,
    /// Feature names that belong to the group.
    pub members: Vec<String>,
    #[serde(default)]
    /// Whether multiple default-enabled members should be reported as an error.
    pub mutually_exclusive: bool,
    /// Optional group description shown in generated output.
    pub description: Option<String>,
}

/// Additional author-provided metadata for a feature.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct FeatureMetadata {
    /// Description shown in generated docs.
    pub description: Option<String>,
    /// Optional family label such as `runtime`, `tls`, or `serialization`.
    pub category: Option<String>,
    /// Version or release label where the feature became available.
    pub since: Option<String>,
    /// URL for feature-specific documentation.
    pub docs: Option<String>,
    /// URL for an issue tracking unstable or planned work.
    pub tracking_issue: Option<String>,
    #[serde(default)]
    /// Prerequisite labels or related feature names.
    pub requires: Vec<String>,
    #[serde(default = "default_public")]
    /// Whether the feature should appear in public generated output.
    pub public: bool,
    #[serde(default)]
    /// Whether the feature is experimental.
    pub unstable: bool,
    #[serde(default)]
    /// Whether the feature is deprecated.
    pub deprecated: bool,
    #[serde(default)]
    /// Acknowledges intentional default enablement of private/deprecated/unstable features.
    pub allow_default: bool,
    /// Extra context appended to Markdown and explain output.
    pub note: Option<String>,
}

impl Default for FeatureMetadata {
    fn default() -> Self {
        Self {
            description: None,
            category: None,
            since: None,
            docs: None,
            tracking_issue: None,
            requires: Vec::new(),
            public: true,
            unstable: false,
            deprecated: false,
            allow_default: false,
            note: None,
        }
    }
}

impl FeatureMetadata {
    /// Returns a human-readable list of status labels for display output.
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

fn default_public() -> bool {
    true
}
