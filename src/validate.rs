use std::collections::BTreeSet;
use std::fmt;

use crate::model::{FeatureManifest, FeatureRef};

/// Severity level attached to a validation issue.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Severity {
    Warning,
    Error,
}

impl fmt::Display for Severity {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Warning => formatter.write_str("warning"),
            Self::Error => formatter.write_str("error"),
        }
    }
}

/// A single validation finding produced by [`validate`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Issue {
    pub severity: Severity,
    pub code: &'static str,
    pub feature: Option<String>,
    pub message: String,
}

impl Issue {
    fn error(code: &'static str, feature: Option<String>, message: impl Into<String>) -> Self {
        Self {
            severity: Severity::Error,
            code,
            feature,
            message: message.into(),
        }
    }

    fn warning(code: &'static str, feature: Option<String>, message: impl Into<String>) -> Self {
        Self {
            severity: Severity::Warning,
            code,
            feature,
            message: message.into(),
        }
    }
}

impl fmt::Display for Issue {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.feature {
            Some(feature) => write!(
                formatter,
                "{}[{}] `{}`: {}",
                self.severity, self.code, feature, self.message
            ),
            None => write!(
                formatter,
                "{}[{}]: {}",
                self.severity, self.code, self.message
            ),
        }
    }
}

/// Aggregated output from a validation run.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ValidationReport {
    pub issues: Vec<Issue>,
}

impl ValidationReport {
    /// Returns `true` when any issue is an error.
    pub fn has_errors(&self) -> bool {
        self.issues
            .iter()
            .any(|issue| issue.severity == Severity::Error)
    }

    pub fn error_count(&self) -> usize {
        self.issues
            .iter()
            .filter(|issue| issue.severity == Severity::Error)
            .count()
    }

    pub fn warning_count(&self) -> usize {
        self.issues
            .iter()
            .filter(|issue| issue.severity == Severity::Warning)
            .count()
    }

    /// Returns a compact summary line suitable for CLI output.
    pub fn summary(&self, feature_count: usize, group_count: usize) -> String {
        format!(
            "validated {feature_count} feature(s) and {group_count} group(s): {} error(s), {} warning(s)",
            self.error_count(),
            self.warning_count()
        )
    }
}

/// Validates a manifest for missing docs, stale metadata, and risky defaults.
pub fn validate(manifest: &FeatureManifest) -> ValidationReport {
    let mut issues = Vec::new();

    for feature in manifest.ordered_features() {
        if !feature.has_metadata {
            issues.push(Issue::error(
                "missing-metadata",
                Some(feature.name.clone()),
                "feature is defined in `[features]` but missing metadata; add an entry under `[package.metadata.feature-manifest]`.",
            ));
        }

        let missing_description = feature
            .metadata
            .description
            .as_deref()
            .map(str::trim)
            .map(str::is_empty)
            .unwrap_or(true);

        if missing_description {
            issues.push(Issue::error(
                "missing-description",
                Some(feature.name.clone()),
                "feature metadata needs a non-empty `description`.",
            ));
        }

        if feature.default_enabled
            && (feature.metadata.unstable
                || feature.metadata.deprecated
                || !feature.metadata.public)
            && !feature.metadata.allow_default
        {
            issues.push(Issue::error(
                "sensitive-default",
                Some(feature.name.clone()),
                "feature is enabled by default while marked unstable, deprecated, or private; set `allow_default = true` to acknowledge the default.",
            ));
        }

        for reference in &feature.enables {
            if let FeatureRef::Unknown { raw } = reference {
                issues.push(Issue::warning(
                    "unknown-reference",
                    Some(feature.name.clone()),
                    format!("feature contains an unrecognized reference syntax: `{raw}`."),
                ));
            }
        }
    }

    for (name, _) in &manifest.metadata_only {
        issues.push(Issue::error(
            "unknown-metadata",
            Some(name.clone()),
            "metadata exists for a feature that is not declared in `[features]`.",
        ));
    }

    for reference in &manifest.default_members {
        match reference {
            FeatureRef::Feature { name } => {
                if !manifest.features.contains_key(name) {
                    issues.push(Issue::error(
                        "unknown-default-member",
                        Some(name.clone()),
                        "entry appears in `features.default` but is not a declared feature.",
                    ));
                }
            }
            FeatureRef::Unknown { raw } => {
                issues.push(Issue::warning(
                    "unknown-default-reference",
                    Some(raw.clone()),
                    "default feature set contains an unrecognized reference syntax.",
                ));
            }
            _ => {}
        }
    }

    for group in &manifest.groups {
        if group.members.len() < 2 {
            issues.push(Issue::warning(
                "small-group",
                Some(group.name.clone()),
                "groups are most useful with at least two members.",
            ));
        }

        let mut seen = BTreeSet::new();
        let mut default_members = Vec::new();

        for member in &group.members {
            if !seen.insert(member) {
                issues.push(Issue::error(
                    "duplicate-group-member",
                    Some(group.name.clone()),
                    format!("group includes `{member}` more than once."),
                ));
            }

            let Some(feature) = manifest.features.get(member) else {
                issues.push(Issue::error(
                    "unknown-group-member",
                    Some(group.name.clone()),
                    format!("group references missing feature `{member}`."),
                ));
                continue;
            };

            if feature.default_enabled {
                default_members.push(member.clone());
            }
        }

        if group.mutually_exclusive && default_members.len() > 1 {
            issues.push(Issue::error(
                "mutually-exclusive-default",
                Some(group.name.clone()),
                format!(
                    "mutually exclusive group has multiple default-enabled members: {}.",
                    default_members
                        .iter()
                        .map(|member| format!("`{member}`"))
                        .collect::<Vec<_>>()
                        .join(", ")
                ),
            ));
        }
    }

    ValidationReport { issues }
}
