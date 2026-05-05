use anyhow::Context as _;
use clap::Command;

use crate::lint_docs;

use super::command_definition;

pub fn render_cli_markdown() -> String {
    let root = command_definition();
    let mut output = String::new();
    output.push_str("# CLI Reference\n\n");
    output.push_str("Generated from the Clap command definitions.\n\n");
    output.push_str("Update this file with:\n\n");
    output.push_str("```text\ncargo fm help-markdown > docs/cli.md\n```\n\n");

    push_command_section(
        &mut output,
        "cargo fm",
        root.clone().bin_name("cargo fm"),
        2,
    );

    for subcommand in root
        .get_subcommands()
        .filter(|command| !command.is_hide_set())
    {
        let mut command = subcommand.clone();
        let name = command.get_name().to_owned();
        command = command.bin_name(format!("cargo fm {name}"));
        push_command_section(&mut output, &format!("cargo fm {name}"), command, 2);
    }

    output
}

pub fn render_lint_markdown() -> String {
    let mut output = String::new();
    output.push_str("# Lint Reference\n\n");
    output.push_str("Generated from the feature-manifest lint registry.\n\n");
    output.push_str("Update this file with:\n\n");
    output.push_str("```text\ncargo fm lints --markdown > docs/lints.md\n```\n\n");

    for lint in lint_docs() {
        output.push_str(&format!("## `{}`\n\n", lint.code));
        output.push_str(&format!("Default: `{}`\n\n", lint.default_severity));
        output.push_str(&wrap_markdown_text(lint.summary, 88));
        output.push_str("\n\n");
        output.push_str(&wrap_markdown_text(&format!("Fix: {}", lint.guidance), 88));
        output.push_str("\n\n");
    }

    output.truncate(output.trim_end().len());
    output.push('\n');
    output
}

fn wrap_markdown_text(value: &str, max_width: usize) -> String {
    let mut lines = Vec::new();
    let mut current = String::new();

    for word in markdown_words(value) {
        let next_len = if current.is_empty() {
            word.len()
        } else {
            current.len() + 1 + word.len()
        };

        if next_len > max_width && !current.is_empty() {
            lines.push(current);
            current = word;
        } else {
            if !current.is_empty() {
                current.push(' ');
            }
            current.push_str(&word);
        }
    }

    if !current.is_empty() {
        lines.push(current);
    }

    lines.join("\n")
}

fn markdown_words(value: &str) -> Vec<String> {
    let mut words = Vec::new();
    let mut current = String::new();
    let mut in_code = false;

    for character in value.chars() {
        if character == '`' {
            current.push(character);
            in_code = !in_code;
            continue;
        }

        if character.is_whitespace() && !in_code {
            if !current.is_empty() {
                words.push(std::mem::take(&mut current));
            }
        } else {
            current.push(character);
        }
    }

    if !current.is_empty() {
        words.push(current);
    }

    words
}

fn push_command_section(output: &mut String, title: &str, mut command: Command, level: usize) {
    output.push_str(&format!("{} `{title}`\n\n", "#".repeat(level)));
    output.push_str("```text\n");

    let mut help = Vec::new();
    command
        .write_long_help(&mut help)
        .context("failed to render CLI help")
        .expect("writing CLI help to a buffer should not fail");
    let help = String::from_utf8(help).expect("Clap help should be valid UTF-8");
    output.push_str(help.trim_end());

    output.push_str("\n```\n\n");
}
