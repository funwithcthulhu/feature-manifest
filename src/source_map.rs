use crate::Issue;

/// Best-effort source location inside a manifest file.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SourceSpan {
    pub line: usize,
    pub column: usize,
}

/// Lightweight source map for locating feature-manifest diagnostics in TOML.
#[derive(Debug, Clone, Copy)]
pub struct ManifestSourceMap<'a> {
    source: &'a str,
}

impl<'a> ManifestSourceMap<'a> {
    pub fn new(source: &'a str) -> Self {
        Self { source }
    }

    pub fn span_for_issue(&self, issue: &Issue) -> Option<SourceSpan> {
        let name = issue.feature.as_deref()?;

        match issue.code {
            "small-group"
            | "duplicate-group-member"
            | "unknown-group-member"
            | "mutually-exclusive-default" => self.group_name_span(name),
            "unknown-metadata" => self.metadata_key_span(name),
            _ => self
                .feature_key_span(name)
                .or_else(|| self.metadata_key_span(name)),
        }
    }

    pub fn feature_key_span(&self, name: &str) -> Option<SourceSpan> {
        self.key_span_in_sections(&["[features]"], name)
    }

    pub fn metadata_key_span(&self, name: &str) -> Option<SourceSpan> {
        self.key_span_in_sections(
            &[
                "[package.metadata.feature-manifest.features]",
                "[package.metadata.feature-manifest]",
                "[package.metadata.feature-docs.features]",
                "[package.metadata.feature-docs]",
            ],
            name,
        )
    }

    pub fn group_name_span(&self, group_name: &str) -> Option<SourceSpan> {
        let mut in_group = false;

        for (index, line) in self.source.lines().enumerate() {
            let trimmed = line.trim();
            if trimmed.starts_with('[') {
                in_group = matches!(
                    trimmed,
                    "[[package.metadata.feature-manifest.groups]]"
                        | "[[package.metadata.feature-docs.groups]]"
                );
                continue;
            }

            if in_group {
                let Some((key, column)) = toml_key_span(line) else {
                    continue;
                };
                if key == "name" && line_has_string_value(line, group_name) {
                    return Some(SourceSpan {
                        line: index + 1,
                        column,
                    });
                }
            }
        }

        None
    }

    fn key_span_in_sections(&self, sections: &[&str], name: &str) -> Option<SourceSpan> {
        let mut in_section = false;

        for (index, line) in self.source.lines().enumerate() {
            let trimmed = line.trim();
            if trimmed.starts_with('[') {
                in_section = sections.contains(&trimmed);
                continue;
            }

            if in_section {
                let Some((key, column)) = toml_key_span(line) else {
                    continue;
                };
                if key == name {
                    return Some(SourceSpan {
                        line: index + 1,
                        column,
                    });
                }
            }
        }

        None
    }
}

fn toml_key_span(line: &str) -> Option<(String, usize)> {
    let (raw_key, _) = line.split_once('=')?;
    let trimmed_start = raw_key.find(|character: char| !character.is_whitespace())?;
    let key = raw_key[trimmed_start..].trim();
    if key.starts_with('#') || key.is_empty() {
        return None;
    }

    Some((
        key.trim_matches('"').trim_matches('\'').trim().to_owned(),
        trimmed_start + 1,
    ))
}

fn line_has_string_value(line: &str, value: &str) -> bool {
    line.contains(&format!("\"{value}\"")) || line.contains(&format!("'{value}'"))
}
