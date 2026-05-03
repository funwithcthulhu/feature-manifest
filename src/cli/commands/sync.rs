use anyhow::{Result, bail};

use crate::cli::util::pluralized;
use crate::{SyncOptions, SyncReport, WorkspaceManifest, sync_manifest};

pub fn run(workspace: &WorkspaceManifest, options: SyncOptions) -> Result<()> {
    let mut changed_packages = 0usize;
    let mut check_failed = false;

    for package in &workspace.packages {
        let report = sync_manifest(&package.manifest_path, &options)?;
        let package_name = report.package_name.as_deref().unwrap_or("unknown-package");

        if report.changed() {
            changed_packages += 1;
            if options.check_only {
                check_failed = true;
                println!(
                    "sync drift in `{package_name}`: {}",
                    change_summary(&report)
                );
            } else {
                println!("synced `{package_name}`: {}", change_summary(&report));
            }

            for feature in &report.added_features {
                println!("  + {feature}");
            }
            for feature in &report.removed_features {
                println!("  - {feature}");
            }
        } else {
            println!("`{package_name}` is already in sync");
        }
    }

    if changed_packages > 0 && !options.check_only {
        println!("updated {changed_packages} package(s)");
    }

    if check_failed {
        bail!("sync drift detected");
    }

    Ok(())
}

pub fn change_summary(report: &SyncReport) -> String {
    let mut parts = Vec::new();
    if !report.added_features.is_empty() {
        parts.push(format!(
            "added {}",
            pluralized(report.added_features.len(), "feature")
        ));
    }
    if !report.removed_features.is_empty() {
        parts.push(format!(
            "removed {} stale metadata entr{}",
            report.removed_features.len(),
            if report.removed_features.len() == 1 {
                "y"
            } else {
                "ies"
            }
        ));
    }
    if parts.is_empty() {
        parts.push("layout updated".to_owned());
    }
    parts.push(format!("using `{}` layout", report.style));
    parts.join(", ")
}
