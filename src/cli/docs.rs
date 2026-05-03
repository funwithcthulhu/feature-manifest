use anyhow::Context as _;
use clap::Command;

use crate::lint_docs;

use super::command_definition;

pub fn render_cli_markdown() -> String {
    let root = command_definition();
    let mut output = String::new();
    output.push_str("# CLI Reference\n\n");
    output.push_str("Generated from the Clap command definitions. Update this file with `cargo fm help-markdown > docs/cli.md`.\n\n");

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
    output.push_str("Generated from the feature-manifest lint registry. Update this file with `cargo fm lints --markdown > docs/lints.md`.\n\n");
    output.push_str("| Code | Default | Meaning | Fix |\n");
    output.push_str("| --- | --- | --- | --- |\n");

    for lint in lint_docs() {
        output.push_str(&format!(
            "| `{}` | `{}` | {} | {} |\n",
            lint.code,
            lint.default_severity,
            escape_markdown_table_cell(lint.summary),
            escape_markdown_table_cell(lint.guidance)
        ));
    }

    output
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

fn escape_markdown_table_cell(value: &str) -> String {
    value.replace('|', "\\|").replace('\n', "<br>")
}
