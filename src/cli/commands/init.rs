use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

use crate::{
    InjectionMarkers, MetadataLayout, PackageSelection, SyncOptions, WorkspaceManifest,
    ensure_injection_markers, inject_between_markers, load_workspace, render_markdown,
    sync_manifest,
};

#[derive(Debug, Clone)]
pub struct InitOptions {
    pub manifest_path: PathBuf,
    pub selection: PackageSelection,
    pub readme: Option<PathBuf>,
    pub no_readme: bool,
    pub ci: bool,
    pub style: Option<MetadataLayout>,
}

pub fn run(workspace: &WorkspaceManifest, options: InitOptions) -> Result<()> {
    let root_directory = workspace
        .root_manifest_path
        .parent()
        .unwrap_or_else(|| Path::new("."));

    for package in &workspace.packages {
        let report = sync_manifest(
            &package.manifest_path,
            &SyncOptions {
                check_only: false,
                remove_stale: false,
                style: options.style,
            },
        )?;
        let package_name = report.package_name.as_deref().unwrap_or("unknown-package");
        if report.changed() {
            println!("initialized metadata for `{package_name}`");
        } else {
            println!("metadata for `{package_name}` is already initialized");
        }
    }

    let refreshed_workspace = load_workspace(&options.manifest_path, options.selection)?;

    if !options.no_readme {
        let readme_path = options
            .readme
            .unwrap_or_else(|| root_directory.join("README.md"));
        let markers = InjectionMarkers::default();
        ensure_injection_markers(&readme_path, &markers, "## Features")?;
        inject_between_markers(
            &readme_path,
            &render_markdown(&refreshed_workspace, false),
            &markers,
        )?;
        println!(
            "updated README feature section at `{}`",
            readme_path.display()
        );
    }

    if options.ci {
        let workflow_path = root_directory
            .join(".github")
            .join("workflows")
            .join("feature-manifest.yml");
        write_ci_workflow(&workflow_path)?;
        println!("added CI workflow at `{}`", workflow_path.display());
    }

    println!("next: cargo fm");
    Ok(())
}

fn write_ci_workflow(path: &Path) -> Result<()> {
    if path.exists() {
        return Ok(());
    }

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).with_context(|| {
            format!("failed to create workflow directory `{}`", parent.display())
        })?;
    }

    fs::write(path, CI_WORKFLOW)
        .with_context(|| format!("failed to write CI workflow `{}`", path.display()))
}

const CI_WORKFLOW: &str = r#"name: Feature Manifest

on:
  push:
    branches:
      - main
  pull_request:

jobs:
  feature-manifest:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - name: Install feature-manifest
        run: cargo install feature-manifest --locked
      - name: Check feature metadata
        run: cargo fm
      - name: Check generated README section
        run: cargo fm md --check -i README.md
"#;
