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
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DoctorLevel {
    Ok,
    Warn,
    Error,
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

    for (level, message) in &checks {
        let label = match level {
            DoctorLevel::Ok => "ok",
            DoctorLevel::Warn => "warn",
            DoctorLevel::Error => "error",
        };
        println!("{label}: {message}");
    }

    if checks.iter().any(|(level, _)| *level == DoctorLevel::Error) {
        bail!("doctor found errors");
    }

    Ok(())
}

fn install_shape_check() -> (DoctorLevel, String) {
    let Ok(current_exe) = std::env::current_exe() else {
        return (
            DoctorLevel::Warn,
            "could not inspect current executable".to_owned(),
        );
    };
    let Some(directory) = current_exe.parent() else {
        return (
            DoctorLevel::Warn,
            "could not inspect executable directory".to_owned(),
        );
    };

    let suffix = std::env::consts::EXE_SUFFIX;
    let short = directory.join(format!("cargo-fm{suffix}"));
    let long = directory.join(format!("cargo-feature-manifest{suffix}"));

    if short.exists() && long.exists() {
        (
            DoctorLevel::Ok,
            "both cargo-fm and cargo-feature-manifest entrypoints are present".to_owned(),
        )
    } else {
        (
            DoctorLevel::Warn,
            "could not find both cargo-fm and cargo-feature-manifest beside the current executable"
                .to_owned(),
        )
    }
}

fn validation_checks(workspace: &WorkspaceManifest) -> Vec<(DoctorLevel, String)> {
    let reports = check::collect_reports(workspace, &[], None).unwrap_or_default();
    reports
        .into_iter()
        .map(|(package, report)| {
            let package_name = package.package_name.as_deref().unwrap_or("unknown-package");
            if report.has_errors() {
                (
                    DoctorLevel::Error,
                    format!(
                        "`{package_name}` has {} validation error(s)",
                        report.error_count()
                    ),
                )
            } else if report.warning_count() > 0 {
                (
                    DoctorLevel::Warn,
                    format!(
                        "`{package_name}` has {} validation warning(s)",
                        report.warning_count()
                    ),
                )
            } else {
                (
                    DoctorLevel::Ok,
                    format!("`{package_name}` feature metadata validates cleanly"),
                )
            }
        })
        .collect()
}

fn lint_compatibility_checks(workspace: &WorkspaceManifest) -> Vec<(DoctorLevel, String)> {
    let known = KNOWN_LINT_CODES
        .iter()
        .copied()
        .collect::<std::collections::BTreeSet<_>>();
    let mut checks = Vec::new();

    for package in &workspace.packages {
        let package_name = package.package_name.as_deref().unwrap_or("unknown-package");
        for code in package.lint_overrides.keys() {
            if !known.contains(code.as_str()) {
                checks.push((
                    DoctorLevel::Error,
                    format!("`{package_name}` configures unknown lint `{code}`"),
                ));
            }
        }
    }

    if checks.is_empty() {
        checks.push((
            DoctorLevel::Ok,
            "lint configuration uses known codes".to_owned(),
        ));
    }

    checks
}

fn readme_checks(workspace: &WorkspaceManifest, readme_path: &Path) -> Vec<(DoctorLevel, String)> {
    let markers = InjectionMarkers::default();
    let Ok(report) = inspect_markers(readme_path, &markers) else {
        return vec![(
            DoctorLevel::Warn,
            format!(
                "README markers were not found at `{}`",
                readme_path.display()
            ),
        )];
    };

    if !report.ready() {
        return vec![(
            DoctorLevel::Error,
            format!(
                "README markers in `{}` are partial, duplicated, or out of order",
                readme_path.display()
            ),
        )];
    }

    match injected_region_matches(readme_path, &render_markdown(workspace, false), &markers) {
        Ok(true) => vec![(
            DoctorLevel::Ok,
            format!(
                "README feature section is up to date at `{}`",
                readme_path.display()
            ),
        )],
        Ok(false) => vec![(
            DoctorLevel::Warn,
            format!(
                "README feature section is stale at `{}`",
                readme_path.display()
            ),
        )],
        Err(error) => vec![(DoctorLevel::Error, error.to_string())],
    }
}

fn ci_check(root_directory: &Path) -> (DoctorLevel, String) {
    let workflow_directory = root_directory.join(".github").join("workflows");
    let Ok(entries) = fs::read_dir(&workflow_directory) else {
        return (
            DoctorLevel::Warn,
            "no GitHub Actions workflow directory found".to_owned(),
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
            return (
                DoctorLevel::Ok,
                format!(
                    "CI workflow references feature-manifest in `{}`",
                    path.display()
                ),
            );
        }
    }

    (
        DoctorLevel::Warn,
        "no GitHub Actions workflow references feature-manifest".to_owned(),
    )
}
