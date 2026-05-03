use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use cargo_metadata::{Metadata, MetadataCommand, Package, PackageId};

use crate::model::{DependencyInfo, WorkspaceManifest};
use crate::parse::load_manifest;

/// Selects which package set to load from a Cargo workspace.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PackageSelection {
    Default,
    Workspace,
    Package(String),
}

/// Resolves a manifest path from either a `Cargo.toml` file or a crate
/// directory. When omitted, the current directory is used.
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

/// Loads a selected package set from Cargo metadata and parses each manifest.
pub fn load_workspace(
    manifest_path: impl AsRef<Path>,
    selection: PackageSelection,
) -> Result<WorkspaceManifest> {
    let manifest_path = manifest_path.as_ref();
    let metadata = MetadataCommand::new()
        .manifest_path(manifest_path)
        .no_deps()
        .exec()
        .with_context(|| {
            format!(
                "failed to run `cargo metadata` for `{}`",
                manifest_path.display()
            )
        })?;

    let selected_ids = select_package_ids(&metadata, &selection)?;
    let mut packages = Vec::new();

    for package_id in selected_ids {
        let package = metadata
            .packages
            .iter()
            .find(|candidate| candidate.id == package_id)
            .with_context(|| format!("missing workspace package metadata for `{package_id}`"))?;
        let mut manifest = load_manifest(package.manifest_path.as_std_path())?;
        manifest.dependencies = collect_dependency_info(package);
        packages.push(manifest);
    }

    packages.sort_by(|left, right| {
        left.package_name
            .as_deref()
            .cmp(&right.package_name.as_deref())
            .then_with(|| left.manifest_path.cmp(&right.manifest_path))
    });

    Ok(WorkspaceManifest {
        root_manifest_path: metadata.workspace_root.as_std_path().join("Cargo.toml"),
        packages,
    })
}

fn select_package_ids(metadata: &Metadata, selection: &PackageSelection) -> Result<Vec<PackageId>> {
    let workspace_packages = metadata
        .workspace_members
        .iter()
        .filter_map(|member_id| {
            metadata
                .packages
                .iter()
                .find(|package| package.id == *member_id)
                .map(|package| (member_id.clone(), package))
        })
        .collect::<Vec<_>>();

    match selection {
        PackageSelection::Workspace => Ok(workspace_packages
            .iter()
            .map(|(member_id, _)| member_id.clone())
            .collect()),
        PackageSelection::Package(package_name) => {
            let matches = workspace_packages
                .iter()
                .filter(|(_, package)| package.name.as_str() == package_name)
                .map(|(member_id, _)| member_id.clone())
                .collect::<Vec<_>>();

            if matches.is_empty() {
                bail!(
                    "package `{package_name}` was not found in the workspace; available packages: {}",
                    available_package_names(&workspace_packages)
                );
            }

            Ok(matches)
        }
        PackageSelection::Default => select_default_package(metadata, &workspace_packages),
    }
}

fn select_default_package(
    metadata: &Metadata,
    workspace_packages: &[(PackageId, &Package)],
) -> Result<Vec<PackageId>> {
    if let Some(root_package) = metadata.root_package() {
        if workspace_packages
            .iter()
            .any(|(member_id, _)| *member_id == root_package.id)
        {
            return Ok(vec![root_package.id.clone()]);
        }
    }

    if workspace_packages.len() == 1 {
        return Ok(vec![workspace_packages[0].0.clone()]);
    }

    bail!(
        "the selected manifest resolves to a workspace with multiple packages; use `--workspace` or `--package <name>`. Available packages: {}",
        available_package_names(workspace_packages)
    );
}

fn available_package_names(workspace_packages: &[(PackageId, &Package)]) -> String {
    workspace_packages
        .iter()
        .map(|(_, package)| package.name.as_str())
        .collect::<Vec<_>>()
        .join(", ")
}

fn collect_dependency_info(package: &Package) -> BTreeMap<String, DependencyInfo> {
    let mut dependencies = BTreeMap::new();

    for dependency in &package.dependencies {
        let key = dependency
            .rename
            .clone()
            .unwrap_or_else(|| dependency.name.to_string());
        dependencies.insert(
            key.clone(),
            DependencyInfo {
                key,
                package: dependency.name.to_string(),
                optional: dependency.optional,
            },
        );
    }

    dependencies
}
