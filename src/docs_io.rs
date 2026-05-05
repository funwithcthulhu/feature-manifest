use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};

/// Marker pair used for injecting generated content into an existing file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InjectionMarkers {
    /// Marker that starts the generated region.
    pub start: String,
    /// Marker that ends the generated region.
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
    /// Path that was updated.
    pub path: PathBuf,
}

/// Marker discovery details for a generated documentation region.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MarkerReport {
    /// Path that was inspected.
    pub path: PathBuf,
    /// Number of start markers found.
    pub start_count: usize,
    /// Number of end markers found.
    pub end_count: usize,
    /// Whether the first start marker appears before the first end marker.
    pub ordered: bool,
}

impl MarkerReport {
    /// Returns `true` when exactly one ordered marker pair exists.
    pub fn ready(&self) -> bool {
        self.start_count == 1 && self.end_count == 1 && self.ordered
    }
}

/// Writes generated content to a file, ensuring a trailing newline.
pub fn write_output(path: impl AsRef<Path>, contents: &str) -> Result<()> {
    let path = path.as_ref();
    let normalized = ensure_trailing_newline(contents);
    fs::write(path, normalized)
        .with_context(|| format!("failed to write generated output to `{}`", path.display()))
}

/// Returns `true` when a generated file already matches the expected contents.
pub fn output_matches(path: impl AsRef<Path>, contents: &str) -> Result<bool> {
    let path = path.as_ref();
    let existing = fs::read_to_string(path)
        .with_context(|| format!("failed to read generated output `{}`", path.display()))?;
    Ok(normalize_line_endings(&existing)
        == normalize_line_endings(&ensure_trailing_newline(contents)))
}

/// Returns marker status for a document without changing it.
pub fn inspect_markers(path: impl AsRef<Path>, markers: &InjectionMarkers) -> Result<MarkerReport> {
    let path = path.as_ref();
    let existing = fs::read_to_string(path)
        .with_context(|| format!("failed to read document `{}`", path.display()))?;
    Ok(inspect_marker_source(path, &existing, markers))
}

/// Ensures a document contains a marker pair, appending it when absent.
pub fn ensure_injection_markers(
    path: impl AsRef<Path>,
    markers: &InjectionMarkers,
    heading: &str,
) -> Result<MarkerReport> {
    let path = path.as_ref();
    let existing = match fs::read_to_string(path) {
        Ok(existing) => existing,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => String::new(),
        Err(error) => {
            return Err(error)
                .with_context(|| format!("failed to read document `{}`", path.display()));
        }
    };

    let report = inspect_marker_source(path, &existing, markers);
    if report.ready() {
        return Ok(report);
    }

    if report.start_count != 0 || report.end_count != 0 {
        bail!(
            "document `{}` has partial or duplicated feature-manifest markers",
            path.display()
        );
    }

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).with_context(|| {
            format!(
                "failed to create parent directory for document `{}`",
                path.display()
            )
        })?;
    }

    let mut updated = existing.trim_end().to_owned();
    if !updated.is_empty() {
        updated.push_str("\n\n");
    }
    updated.push_str(&format!(
        "{heading}\n\n{}\n{}\n",
        markers.start, markers.end
    ));

    fs::write(path, updated)
        .with_context(|| format!("failed to write document `{}`", path.display()))?;

    inspect_markers(path, markers)
}

/// Returns `true` when the region between markers already matches the expected contents.
pub fn injected_region_matches(
    path: impl AsRef<Path>,
    contents: &str,
    markers: &InjectionMarkers,
) -> Result<bool> {
    let path = path.as_ref();
    let existing = fs::read_to_string(path).with_context(|| {
        format!(
            "failed to read document `{}` for injection check",
            path.display()
        )
    })?;
    let region = marker_region(path, &existing, markers)?;
    Ok(normalize_line_endings(region.trim()) == normalize_line_endings(contents.trim()))
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

    let (start_index, end_index) = marker_bounds(path, &existing, markers)?;

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

fn inspect_marker_source(path: &Path, source: &str, markers: &InjectionMarkers) -> MarkerReport {
    let start_positions = marker_positions(source, &markers.start);
    let end_positions = marker_positions(source, &markers.end);
    let ordered = match (start_positions.first(), end_positions.first()) {
        (Some(start), Some(end)) => start < end,
        _ => false,
    };

    MarkerReport {
        path: path.to_path_buf(),
        start_count: start_positions.len(),
        end_count: end_positions.len(),
        ordered,
    }
}

fn marker_region<'a>(path: &Path, source: &'a str, markers: &InjectionMarkers) -> Result<&'a str> {
    let (start_index, end_index) = marker_bounds(path, source, markers)?;
    Ok(&source[start_index + markers.start.len()..end_index])
}

fn marker_bounds(path: &Path, source: &str, markers: &InjectionMarkers) -> Result<(usize, usize)> {
    let report = inspect_marker_source(path, source, markers);
    if report.start_count == 0 {
        bail!(
            "start marker `{}` was not found in `{}`",
            markers.start,
            path.display()
        );
    }
    if report.end_count == 0 {
        bail!(
            "end marker `{}` was not found in `{}`",
            markers.end,
            path.display()
        );
    }
    if report.start_count > 1 || report.end_count > 1 {
        bail!(
            "document `{}` has duplicate feature-manifest markers",
            path.display()
        );
    }
    if !report.ordered {
        bail!(
            "marker order is invalid in `{}`; the end marker appears before the start marker",
            path.display()
        );
    }

    Ok((
        source
            .find(&markers.start)
            .expect("start marker was counted"),
        source.find(&markers.end).expect("end marker was counted"),
    ))
}

fn marker_positions(source: &str, marker: &str) -> Vec<usize> {
    source
        .match_indices(marker)
        .map(|(index, _)| index)
        .collect()
}

fn ensure_trailing_newline(contents: &str) -> String {
    if contents.ends_with('\n') {
        contents.to_owned()
    } else {
        format!("{contents}\n")
    }
}

fn normalize_line_endings(contents: &str) -> String {
    contents.replace("\r\n", "\n").replace('\r', "\n")
}
