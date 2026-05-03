use crate::model::{FeatureMetadata, FeatureRef};

pub(super) fn reference_summary(references: &[FeatureRef]) -> String {
    if references.is_empty() {
        return "—".to_owned();
    }

    references
        .iter()
        .map(|reference| format!("`{}`", escape_markdown_inline(&reference.to_string())))
        .collect::<Vec<_>>()
        .join(", ")
}

pub(super) fn sanitize_id(raw: &str) -> String {
    raw.chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() {
                character
            } else {
                '_'
            }
        })
        .collect()
}

pub(super) fn status_summary(metadata: &FeatureMetadata) -> String {
    metadata.status_labels().join(", ")
}

pub(super) fn visibility_label(metadata: &FeatureMetadata) -> &'static str {
    if metadata.public { "public" } else { "private" }
}

pub(super) fn yes_no(value: bool) -> &'static str {
    if value { "yes" } else { "no" }
}

pub(super) fn escape_markdown_inline(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('|', "\\|")
        .replace('\r', "")
        .replace('\n', "<br>")
}

pub(super) fn escape_markdown_cell(value: &str) -> String {
    escape_markdown_inline(value)
}

pub(super) fn escape_mermaid_label(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\r', "")
        .replace('\n', "\\n")
}
