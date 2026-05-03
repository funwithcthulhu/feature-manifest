use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Result, bail};

use crate::cli::commands::check;
use crate::{
    InjectionMarkers, KNOWN_LINT_CODES, WorkspaceManifest, injected_region_matches,
    inspect_markers, render_markdown,
};

#[derive(Debug, Clone)]
pub struct DoctorOptions {
    pub readme: Option<PathBuf>,
    pub strict: bool,
    pub explain: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DoctorLevel {
    Ok,
    Warn,
    Error,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct DoctorCheck {
    level: DoctorLevel,
    message: String,
    next_action: Option<String>,
}

impl DoctorCheck {
    fn ok(message: impl Into<String>) -> Self {
        Self {
            level: DoctorLevel::Ok,
            message: message.into(),
            next_action: None,
        }
    }

    fn warn(message: impl Into<String>, next_action: impl Into<String>) -> Self {
        Self {
            level: DoctorLevel::Warn,
            message: message.into(),
            next_action: Some(next_action.into()),
        }
    }

    fn error(message: impl Into<String>, next_action: impl Into<String>) -> Self {
        Self {
            level: DoctorLevel::Error,
            message: message.into(),
            next_action: Some(next_action.into()),
        }
    }
}

pub fn run(workspace: &WorkspaceManifest, options: DoctorOptions) -> Result<()> {
    let root_directory = workspace
        .root_manifest_path
        .parent()
        .unwrap_or_else(|| Path::new("."));
    let readme_path = options
        .readme
        .unwrap_or_else(|| root_directory.join("README.md"));

    let mut checks = Vec::new();
    checks.push(install_shape_check());
    checks.extend(validation_checks(workspace));
    checks.extend(lint_compatibility_checks(workspace));
    checks.extend(readme_checks(workspace, &readme_path));
    checks.push(ci_check(root_directory));

    for check in &checks {
        let label = match check.level {
            DoctorLevel::Ok => "ok",
            DoctorLevel::Warn => "warn",
            DoctorLevel::Error => "error",
        };
        println!("{label}: {}", check.message);
        if options.explain {
            if let Some(next_action) = &check.next_action {
                println!("  next: {next_action}");
            }
        }
    }

    if checks.iter().any(|check| check.level == DoctorLevel::Error) {
        bail!("doctor found errors");
    }

    if options.strict && checks.iter().any(|check| check.level == DoctorLevel::Warn) {
        bail!("doctor found warnings in strict mode");
    }

    Ok(())
}

fn install_shape_check() -> DoctorCheck {
    let Ok(current_exe) = std::env::current_exe() else {
        return DoctorCheck::warn(
            "could not inspect current executable".to_owned(),
            "run `cargo install feature-manifest --locked` to install both Cargo entrypoints",
        );
    };
    let Some(directory) = current_exe.parent() else {
        return DoctorCheck::warn(
            "could not inspect executable directory".to_owned(),
            "run `cargo install feature-manifest --locked` to install both Cargo entrypoints",
        );
    };

    let suffix = std::env::consts::EXE_SUFFIX;
    let short = directory.join(format!("cargo-fm{suffix}"));
    let long = directory.join(format!("cargo-feature-manifest{suffix}"));

    if short.exists() && long.exists() {
        DoctorCheck::ok(
            "both cargo-fm and cargo-feature-manifest entrypoints are present".to_owned(),
        )
    } else {
        DoctorCheck::warn(
            "could not find both cargo-fm and cargo-feature-manifest beside the current executable",
            "reinstall with `cargo install feature-manifest --locked` so Cargo can find both `cargo fm` and `cargo feature-manifest`",
        )
    }
}

fn validation_checks(workspace: &WorkspaceManifest) -> Vec<DoctorCheck> {
    let reports = check::collect_reports(workspace, &[], None).unwrap_or_default();
    reports
        .into_iter()
        .map(|(package, report)| {
            let package_name = package.package_name.as_deref().unwrap_or("unknown-package");
            if report.has_errors() {
                DoctorCheck::error(
                    format!(
                        "`{package_name}` has {} validation error(s)",
                        report.error_count()
                    ),
                    "run `cargo fm c` to see the failing lints, then fill in or fix feature metadata",
                )
            } else if report.warning_count() > 0 {
                DoctorCheck::warn(
                    format!(
                        "`{package_name}` has {} validation warning(s)",
                        report.warning_count()
                    ),
                    "run `cargo fm c --preset strict` if you want warnings to behave like release-blocking errors",
                )
            } else {
                DoctorCheck::ok(
                    format!("`{package_name}` feature metadata validates cleanly"),
                )
            }
        })
        .collect()
}

fn lint_compatibility_checks(workspace: &WorkspaceManifest) -> Vec<DoctorCheck> {
    let known = KNOWN_LINT_CODES
        .iter()
        .copied()
        .collect::<std::collections::BTreeSet<_>>();
    let mut checks = Vec::new();

    for package in &workspace.packages {
        let package_name = package.package_name.as_deref().unwrap_or("unknown-package");
        for code in package.lint_overrides.keys() {
            if !known.contains(code.as_str()) {
                checks.push(DoctorCheck::error(
                    format!("`{package_name}` configures unknown lint `{code}`"),
                    "run `cargo fm lints` for supported codes or remove the stale lint override",
                ));
            }
        }
    }

    if checks.is_empty() {
        checks.push(DoctorCheck::ok(
            "lint configuration uses known codes".to_owned(),
        ));
    }

    checks
}

fn readme_checks(workspace: &WorkspaceManifest, readme_path: &Path) -> Vec<DoctorCheck> {
    let markers = InjectionMarkers::default();
    let Ok(report) = inspect_markers(readme_path, &markers) else {
        return vec![DoctorCheck::warn(
            format!(
                "README markers were not found at `{}`",
                readme_path.display()
            ),
            format!(
                "run `cargo fm init --readme {}` or add the feature-manifest markers manually",
                readme_path.display()
            ),
        )];
    };

    if !report.ready() {
        return vec![DoctorCheck::error(
            format!(
                "README markers in `{}` are partial, duplicated, or out of order",
                readme_path.display()
            ),
            "keep exactly one start marker before exactly one end marker, then rerun `cargo fm md -i README.md`",
        )];
    }

    match injected_region_matches(readme_path, &render_markdown(workspace, false), &markers) {
        Ok(true) => vec![DoctorCheck::ok(format!(
            "README feature section is up to date at `{}`",
            readme_path.display()
        ))],
        Ok(false) => vec![DoctorCheck::warn(
            format!(
                "README feature section is stale at `{}`",
                readme_path.display()
            ),
            format!(
                "run `cargo fm md -i {}` to refresh generated feature docs",
                readme_path.display()
            ),
        )],
        Err(error) => vec![DoctorCheck::error(
            error.to_string(),
            "inspect the README markers and rerun `cargo fm md -i README.md` once they are valid",
        )],
    }
}

fn ci_check(root_directory: &Path) -> DoctorCheck {
    let workflow_directory = root_directory.join(".github").join("workflows");
    let Ok(entries) = fs::read_dir(&workflow_directory) else {
        return DoctorCheck::warn(
            "no GitHub Actions workflow directory found".to_owned(),
            "run `cargo fm init --ci` or add `cargo fm` to an existing CI workflow",
        );
    };

    for entry in entries.flatten() {
        let path = entry.path();
        let extension = path.extension().and_then(|value| value.to_str());
        if !matches!(extension, Some("yml") | Some("yaml")) {
            continue;
        }

        if fs::read_to_string(&path)
            .map(|contents| contents.contains("cargo fm") || contents.contains("feature-manifest"))
            .unwrap_or(false)
        {
            return DoctorCheck::ok(format!(
                "CI workflow references feature-manifest in `{}`",
                path.display()
            ));
        }
    }

    DoctorCheck::warn(
        "no GitHub Actions workflow references feature-manifest".to_owned(),
        "run `cargo fm init --ci` or add `cargo fm` and `cargo fm md --check -i README.md` to CI",
    )
}
