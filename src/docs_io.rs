use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};

/// Marker pair used for injecting generated content into an existing file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InjectionMarkers {
    pub start: String,
    pub end: String,
}

impl Default for InjectionMarkers {
    fn default() -> Self {
        Self {
            start: "<!-- feature-manifest:start -->".to_owned(),
            end: "<!-- feature-manifest:end -->".to_owned(),
        }
    }
}

/// Result of a marker-based document injection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InjectionReport {
    pub path: PathBuf,
}

/// Writes generated content to a file, ensuring a trailing newline.
pub fn write_output(path: impl AsRef<Path>, contents: &str) -> Result<()> {
    let path = path.as_ref();
    let normalized = ensure_trailing_newline(contents);
    fs::write(path, normalized)
        .with_context(|| format!("failed to write generated output to `{}`", path.display()))
}

/// Replaces the region between two markers, preserving the markers themselves.
pub fn inject_between_markers(
    path: impl AsRef<Path>,
    contents: &str,
    markers: &InjectionMarkers,
) -> Result<InjectionReport> {
    let path = path.as_ref();
    let existing = fs::read_to_string(path)
        .with_context(|| format!("failed to read document `{}` for injection", path.display()))?;

    let start_index = existing.find(&markers.start).ok_or_else(|| {
        anyhow::anyhow!(
            "start marker `{}` was not found in `{}`",
            markers.start,
            path.display()
        )
    })?;
    let end_index = existing.find(&markers.end).ok_or_else(|| {
        anyhow::anyhow!(
            "end marker `{}` was not found in `{}`",
            markers.end,
            path.display()
        )
    })?;

    if end_index <= start_index {
        bail!(
            "marker order is invalid in `{}`; the end marker appears before the start marker",
            path.display()
        );
    }

    let before = &existing[..start_index + markers.start.len()];
    let after = &existing[end_index..];
    let injected = format!(
        "{before}\n\n{}\n{after}",
        ensure_trailing_newline(contents).trim_end()
    );

    fs::write(path, injected)
        .with_context(|| format!("failed to write injected document `{}`", path.display()))?;

    Ok(InjectionReport {
        path: path.to_path_buf(),
    })
}

fn ensure_trailing_newline(contents: &str) -> String {
    if contents.ends_with('\n') {
        contents.to_owned()
    } else {
        format!("{contents}\n")
    }
}
