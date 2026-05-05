use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use anyhow::{Result, bail};

use crate::model::{FeatureManifest, FeatureRef, LintLevel, LintPreset};

/// Lint codes recognized by validation and CLI override parsing.
pub const KNOWN_LINT_CODES: &[&str] = &[
    "missing-metadata",
    "missing-description",
    "sensitive-default",
    "unknown-reference",
    "unknown-feature-reference",
    "unknown-metadata",
    "unknown-default-member",
    "unknown-default-reference",
    "small-group",
    "duplicate-group-member",
    "unknown-group-member",
    "mutually-exclusive-default",
    "dependency-not-found",
    "dependency-not-optional",
    "private-enabled-by-public",
    "feature-cycle",
];

pub const LINT_DOCS: &[LintDoc] = &[
    LintDoc {
        code: "missing-metadata",
        default_severity: Severity::Error,
        summary: "Feature exists in `[features]` but has no metadata entry.",
        guidance: "Add an entry under `[package.metadata.feature-manifest.features]`, or run `cargo fm sync` to scaffold TODO descriptions.",
    },
    LintDoc {
        code: "missing-description",
        default_severity: Severity::Error,
        summary: "Metadata exists but has no usable description.",
        guidance: "Fill in `description` with text that explains why the feature exists.",
    },
    LintDoc {
        code: "sensitive-default",
        default_severity: Severity::Error,
        summary: "A private, deprecated, or unstable feature is default-enabled without acknowledgement.",
        guidance: "Remove the feature from `default`, or set `allow_default = true` when the default is intentional.",
    },
    LintDoc {
        code: "unknown-reference",
        default_severity: Severity::Warning,
        summary: "A feature entry contains syntax that feature-manifest cannot classify.",
        guidance: "Use local features, `dep:name`, `name/feature`, or `name?/feature` when possible.",
    },
    LintDoc {
        code: "unknown-feature-reference",
        default_severity: Severity::Error,
        summary: "A feature enables a plain name that is neither a declared feature nor an optional dependency.",
        guidance: "Add the missing feature, make the dependency optional, switch to `dep:name`, or remove the stale reference.",
    },
    LintDoc {
        code: "unknown-metadata",
        default_severity: Severity::Error,
        summary: "Metadata exists for a feature that is not declared in `[features]`.",
        guidance: "Delete the stale metadata, re-add the feature, or run `cargo fm sync --remove-stale`.",
    },
    LintDoc {
        code: "unknown-default-member",
        default_severity: Severity::Error,
        summary: "`features.default` contains a missing default member.",
        guidance: "Remove the missing default member, add the feature to `[features]`, or make the referenced dependency optional.",
    },
    LintDoc {
        code: "unknown-default-reference",
        default_severity: Severity::Warning,
        summary: "`features.default` contains syntax that feature-manifest cannot classify.",
        guidance: "Use local feature names in `default` when possible.",
    },
    LintDoc {
        code: "small-group",
        default_severity: Severity::Warning,
        summary: "A group has fewer than two members.",
        guidance: "Add at least one more member or remove the group.",
    },
    LintDoc {
        code: "duplicate-group-member",
        default_severity: Severity::Error,
        summary: "A group repeats the same member more than once.",
        guidance: "Deduplicate the `members` array for the group.",
    },
    LintDoc {
        code: "unknown-group-member",
        default_severity: Severity::Error,
        summary: "A group references a feature that does not exist.",
        guidance: "Remove the missing group member or add the feature to `[features]`.",
    },
    LintDoc {
        code: "mutually-exclusive-default",
        default_severity: Severity::Error,
        summary: "A mutually exclusive group has multiple default-enabled members.",
        guidance: "Keep at most one member of the group in the default feature set.",
    },
    LintDoc {
        code: "dependency-not-found",
        default_severity: Severity::Error,
        summary: "A dependency-based feature reference points at a missing dependency.",
        guidance: "Fix the dependency key, add the dependency, or remove the stale `dep:`/dependency-feature reference.",
    },
    LintDoc {
        code: "dependency-not-optional",
        default_severity: Severity::Error,
        summary: "`dep:name` or `name?/feature` is used for a dependency that is not optional.",
        guidance: "Mark the dependency `optional = true`, or use a plain dependency feature reference when the dependency is always enabled.",
    },
    LintDoc {
        code: "private-enabled-by-public",
        default_severity: Severity::Warning,
        summary: "A public feature enables a private feature.",
        guidance: "Make the dependency feature public too, or document why the public feature intentionally exposes that internal toggle.",
    },
    LintDoc {
        code: "feature-cycle",
        default_severity: Severity::Error,
        summary: "Local features form a cycle.",
        guidance: "Break the cycle by removing one local feature reference from the loop.",
    },
];

/// Documentation for a supported lint code.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LintDoc {
    /// Stable lint code used in output formats and configuration.
    pub code: &'static str,
    /// Default severity before presets or overrides are applied.
    pub default_severity: Severity,
    /// Short lint description.
    pub summary: &'static str,
    /// Suggested fix or next action.
    pub guidance: &'static str,
}

/// Severity level attached to a validation issue.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Severity {
    /// Non-blocking validation finding.
    Warning,
    /// Blocking validation finding.
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

/// Configuration applied during validation.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ValidateOptions {
    /// Per-run lint level overrides keyed by lint code.
    pub cli_lints: BTreeMap<String, LintLevel>,
    /// Per-run lint preset.
    pub cli_preset: Option<LintPreset>,
}

impl ValidateOptions {
    /// Builds options from CLI-style lint overrides.
    pub fn with_cli_lint_overrides(entries: impl IntoIterator<Item = (String, LintLevel)>) -> Self {
        Self {
            cli_lints: entries.into_iter().collect(),
            cli_preset: None,
        }
    }

    /// Applies a CLI-style preset to these options.
    pub fn with_cli_preset(mut self, preset: Option<LintPreset>) -> Self {
        self.cli_preset = preset;
        self
    }
}

/// A single validation finding produced by [`validate`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Issue {
    /// Effective severity after presets and overrides.
    pub severity: Severity,
    /// Severity before presets and overrides.
    pub default_severity: Severity,
    /// Stable lint code.
    pub code: &'static str,
    /// Associated feature or group name when applicable.
    pub feature: Option<String>,
    /// Human-readable explanation.
    pub message: String,
}

impl Issue {
    fn error(code: &'static str, feature: Option<String>, message: impl Into<String>) -> Self {
        Self {
            severity: Severity::Error,
            default_severity: Severity::Error,
            code,
            feature,
            message: message.into(),
        }
    }

    fn warning(code: &'static str, feature: Option<String>, message: impl Into<String>) -> Self {
        Self {
            severity: Severity::Warning,
            default_severity: Severity::Warning,
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
    /// Validation issues in deterministic order.
    pub issues: Vec<Issue>,
}

impl ValidationReport {
    /// Returns `true` when any issue is an error.
    pub fn has_errors(&self) -> bool {
        self.issues
            .iter()
            .any(|issue| issue.severity == Severity::Error)
    }

    /// Counts error-level issues.
    pub fn error_count(&self) -> usize {
        self.issues
            .iter()
            .filter(|issue| issue.severity == Severity::Error)
            .count()
    }

    /// Counts warning-level issues.
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

/// Parses a lint override string like `missing-description=warn`.
pub fn parse_lint_override(raw: &str) -> Result<(String, LintLevel)> {
    let (code, level) = raw
        .split_once('=')
        .ok_or_else(|| anyhow::anyhow!("expected `<lint>=<allow|warn|deny>`, found `{raw}`"))?;

    let code = code.trim().to_owned();
    if !KNOWN_LINT_CODES.contains(&code.as_str()) {
        bail!(
            "unknown lint code `{code}`; known codes: {}",
            KNOWN_LINT_CODES.join(", ")
        );
    }

    Ok((code, level.trim().parse()?))
}

/// Returns the known lint codes in stable order.
pub fn known_lint_codes() -> &'static [&'static str] {
    KNOWN_LINT_CODES
}

/// Returns generated documentation for all known lint codes.
pub fn lint_docs() -> &'static [LintDoc] {
    LINT_DOCS
}

/// Validates a manifest for missing docs, stale metadata, and risky defaults.
pub fn validate(manifest: &FeatureManifest) -> ValidationReport {
    validate_with_options(manifest, &ValidateOptions::default())
}

/// Validates a manifest with CLI lint overrides applied on top of manifest config.
pub fn validate_with_options(
    manifest: &FeatureManifest,
    options: &ValidateOptions,
) -> ValidationReport {
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

        for reference in &feature.enables {
            match reference {
                FeatureRef::Dependency { name } if !manifest.dependencies.is_empty() => {
                    validate_dependency_reference(
                        manifest,
                        &feature.name,
                        name,
                        None,
                        false,
                        &mut issues,
                    );
                }
                FeatureRef::DependencyFeature {
                    dependency,
                    feature: dependency_feature,
                    weak,
                } if !manifest.dependencies.is_empty() => {
                    validate_dependency_reference(
                        manifest,
                        &feature.name,
                        dependency,
                        Some(dependency_feature),
                        *weak,
                        &mut issues,
                    );
                }
                FeatureRef::Feature { name } => {
                    validate_plain_feature_reference(
                        manifest,
                        &feature.name,
                        feature.metadata.public,
                        name,
                        &mut issues,
                    );
                }
                FeatureRef::Dependency { .. }
                | FeatureRef::DependencyFeature { .. }
                | FeatureRef::Unknown { .. } => {}
            }
        }
    }

    for cycle in detect_feature_cycles(manifest) {
        let cycle_summary = cycle.join(" -> ");
        let cycle_features = cycle.into_iter().collect::<BTreeSet<_>>();
        for feature_name in cycle_features {
            issues.push(Issue::error(
                "feature-cycle",
                Some(feature_name),
                format!("feature is part of a local cycle: {cycle_summary}."),
            ));
        }
    }

    for name in manifest.metadata_only.keys() {
        issues.push(Issue::error(
            "unknown-metadata",
            Some(name.clone()),
            "metadata exists for a feature that is not declared in `[features]`.",
        ));
    }

    for reference in &manifest.default_members {
        match reference {
            FeatureRef::Feature { name }
                if !declared_feature_or_optional_dependency(manifest, name) =>
            {
                issues.push(Issue::error(
                    "unknown-default-member",
                    Some(name.clone()),
                    "entry appears in `features.default` but is not a declared feature or optional dependency.",
                ));
            }
            FeatureRef::Feature { .. } => {}
            FeatureRef::Dependency { name } if !manifest.dependencies.is_empty() => {
                validate_dependency_reference(manifest, "default", name, None, false, &mut issues);
            }
            FeatureRef::DependencyFeature {
                dependency,
                feature,
                weak,
            } if !manifest.dependencies.is_empty() => {
                validate_dependency_reference(
                    manifest,
                    "default",
                    dependency,
                    Some(feature),
                    *weak,
                    &mut issues,
                );
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

    ValidationReport {
        issues: apply_lint_overrides(issues, manifest, options),
    }
}

fn validate_dependency_reference(
    manifest: &FeatureManifest,
    source_feature: &str,
    dependency: &str,
    dependency_feature: Option<&str>,
    weak: bool,
    issues: &mut Vec<Issue>,
) {
    match manifest.dependencies.get(dependency) {
        Some(info) => {
            if dependency_feature.is_none() && !info.optional {
                issues.push(Issue::error(
                    "dependency-not-optional",
                    Some(source_feature.to_owned()),
                    format!(
                        "`dep:{dependency}` requires `{dependency}` to be an optional dependency."
                    ),
                ));
            } else if weak && !info.optional {
                let dependency_feature = dependency_feature.unwrap_or_default();
                issues.push(Issue::error(
                    "dependency-not-optional",
                    Some(source_feature.to_owned()),
                    format!(
                        "`{dependency}?/{dependency_feature}` requires `{dependency}` to be optional."
                    ),
                ));
            }
        }
        None => {
            let reference = match dependency_feature {
                Some(feature) => {
                    let separator = if weak { "?/" } else { "/" };
                    format!("`{dependency}{separator}{feature}`")
                }
                None => format!("`dep:{dependency}`"),
            };
            issues.push(Issue::error(
                "dependency-not-found",
                Some(source_feature.to_owned()),
                format!("{reference} references a dependency that does not exist."),
            ));
        }
    }
}

fn validate_plain_feature_reference(
    manifest: &FeatureManifest,
    source_feature: &str,
    source_is_public: bool,
    target_name: &str,
    issues: &mut Vec<Issue>,
) {
    if let Some(target) = manifest.features.get(target_name) {
        if source_is_public && !target.metadata.public {
            issues.push(Issue::warning(
                "private-enabled-by-public",
                Some(source_feature.to_owned()),
                format!(
                    "public feature enables private feature `{target_name}`, which may surprise downstream users."
                ),
            ));
        }
        return;
    }

    if manifest
        .dependencies
        .get(target_name)
        .is_some_and(|dependency| dependency.optional)
    {
        return;
    }

    issues.push(Issue::error(
        "unknown-feature-reference",
        Some(source_feature.to_owned()),
        format!("`{target_name}` is not a declared feature or optional dependency."),
    ));
}

fn declared_feature_or_optional_dependency(manifest: &FeatureManifest, name: &str) -> bool {
    manifest.features.contains_key(name)
        || manifest
            .dependencies
            .get(name)
            .is_some_and(|dependency| dependency.optional)
}

fn detect_feature_cycles(manifest: &FeatureManifest) -> Vec<Vec<String>> {
    let mut cycles = Vec::new();
    let mut path = Vec::new();
    let mut path_set = BTreeSet::new();
    let mut seen = BTreeSet::new();

    for feature_name in manifest.features.keys() {
        visit_feature(
            manifest,
            feature_name,
            &mut seen,
            &mut path,
            &mut path_set,
            &mut cycles,
        );
    }

    cycles.sort();
    cycles.dedup();
    cycles
}

fn visit_feature(
    manifest: &FeatureManifest,
    feature_name: &str,
    seen: &mut BTreeSet<String>,
    path: &mut Vec<String>,
    path_set: &mut BTreeSet<String>,
    cycles: &mut Vec<Vec<String>>,
) {
    if path_set.contains(feature_name) {
        if let Some(position) = path.iter().position(|entry| entry == feature_name) {
            let mut cycle = path[position..].to_vec();
            cycle.push(feature_name.to_owned());
            cycles.push(cycle);
        }
        return;
    }

    if !seen.insert(feature_name.to_owned()) {
        return;
    }

    let Some(feature) = manifest.features.get(feature_name) else {
        return;
    };

    path.push(feature_name.to_owned());
    path_set.insert(feature_name.to_owned());

    for reference in &feature.enables {
        if let Some(next_feature) = reference.local_feature_name() {
            visit_feature(manifest, next_feature, seen, path, path_set, cycles);
        }
    }

    path.pop();
    path_set.remove(feature_name);
}

fn apply_lint_overrides(
    issues: Vec<Issue>,
    manifest: &FeatureManifest,
    options: &ValidateOptions,
) -> Vec<Issue> {
    issues
        .into_iter()
        .filter_map(|mut issue| {
            let override_level = options
                .cli_lints
                .get(issue.code)
                .copied()
                .or_else(|| manifest.lint_overrides.get(issue.code).copied())
                .or_else(|| preset_level(options.cli_preset, issue.code))
                .or_else(|| preset_level(manifest.lint_preset, issue.code));

            match override_level {
                Some(LintLevel::Allow) => None,
                Some(LintLevel::Warn) => {
                    issue.severity = Severity::Warning;
                    Some(issue)
                }
                Some(LintLevel::Deny) => {
                    issue.severity = Severity::Error;
                    Some(issue)
                }
                None => Some(issue),
            }
        })
        .collect()
}

fn preset_level(preset: Option<LintPreset>, code: &str) -> Option<LintLevel> {
    match preset {
        Some(LintPreset::Adopt) => match code {
            "missing-metadata"
            | "missing-description"
            | "unknown-metadata"
            | "small-group"
            | "private-enabled-by-public" => Some(LintLevel::Warn),
            _ => None,
        },
        Some(LintPreset::Strict) => match code {
            "unknown-reference"
            | "unknown-default-reference"
            | "small-group"
            | "private-enabled-by-public" => Some(LintLevel::Deny),
            _ => None,
        },
        None => None,
    }
}
