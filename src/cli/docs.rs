use anyhow::Context as _;
use clap::Command;

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
