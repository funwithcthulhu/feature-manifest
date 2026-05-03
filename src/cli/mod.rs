mod commands;
mod output;
mod util;

use std::ffi::OsString;
use std::path::PathBuf;
use std::process;

use anyhow::{Result, bail};
use clap::{ArgAction, Parser, Subcommand, ValueEnum};

use crate::{
    InjectionMarkers, LintPreset, MetadataLayout, PackageSelection, SyncOptions, known_lint_codes,
    load_workspace, render_explain, render_json, render_mermaid, resolve_manifest_path,
};

#[derive(Debug, Parser)]
#[command(
    version,
    about = "Document, validate, and render Cargo feature metadata.",
    after_help = "Examples:\n  cargo fm\n  cargo fm init --ci\n  cargo fm doctor\n  cargo fm c -f sarif\n  cargo fm -w c -l missing-description=warn\n  cargo fm md -o FEATURES.md\n  cargo fm md --check -i README.md\n  cargo fm s -c -r -s structured\n  cargo fm -p cli show serde\n\nThe original `cargo feature-manifest ...` command and long subcommand names remain supported."
)]
struct Cli {
    #[arg(
        short = 'm',
        long,
        global = true,
        value_name = "PATH",
        help = "Path to Cargo.toml or a crate directory."
    )]
    manifest_path: Option<PathBuf>,

    #[arg(
        short = 'w',
        long,
        global = true,
        action = ArgAction::SetTrue,
        help = "Operate on every workspace member."
    )]
    workspace: bool,

    #[arg(
        short = 'p',
        long,
        global = true,
        value_name = "NAME",
        help = "Select a specific package within a workspace."
    )]
    package: Option<String>,

    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Set up feature-manifest metadata, README markers, and optional CI.
    Init {
        #[arg(long, value_name = "PATH", help = "README path to create or update.")]
        readme: Option<PathBuf>,
        #[arg(long, action = ArgAction::SetTrue, help = "Skip README marker setup.")]
        no_readme: bool,
        #[arg(long, action = ArgAction::SetTrue, help = "Add a GitHub Actions workflow.")]
        ci: bool,
        #[arg(
            short = 's',
            long,
            value_enum,
            help = "Choose the metadata layout to write."
        )]
        style: Option<SyncStyle>,
    },
    /// Check project wiring, generated docs, CI config, and metadata health.
    Doctor {
        #[arg(long, value_name = "PATH", help = "README path to inspect.")]
        readme: Option<PathBuf>,
    },
    /// Validate feature metadata and CI-oriented rules.
    #[command(visible_aliases = ["c", "chk"])]
    Check {
        #[arg(short = 'f', long, value_enum, default_value_t = CheckFormat::Text)]
        format: CheckFormat,
        #[arg(
            short = 'l',
            long = "lint",
            value_name = "CODE=LEVEL",
            action = ArgAction::Append,
            help = "Override one lint level for this run."
        )]
        lint_overrides: Vec<String>,
        #[arg(long, value_enum, help = "Apply a lint preset for this run.")]
        preset: Option<LintPresetArg>,
    },
    /// Render a Markdown feature table.
    #[command(visible_aliases = ["md", "m"])]
    Markdown {
        #[arg(
            short = 'a',
            long,
            action = ArgAction::SetTrue,
            help = "Include private/internal features in the output."
        )]
        include_private: bool,
        #[arg(
            short = 'o',
            long,
            value_name = "PATH",
            help = "Write Markdown to a file."
        )]
        write: Option<PathBuf>,
        #[arg(
            short = 'i',
            long,
            value_name = "PATH",
            help = "Inject Markdown between markers in an existing file."
        )]
        insert_into: Option<PathBuf>,
        #[arg(
            short = 'c',
            long,
            action = ArgAction::SetTrue,
            help = "Exit non-zero when generated Markdown is stale."
        )]
        check: bool,
        #[arg(
            long,
            value_name = "TEXT",
            default_value = "<!-- feature-manifest:start -->",
            help = "Start marker used by `--insert-into`."
        )]
        start_marker: String,
        #[arg(
            long,
            value_name = "TEXT",
            default_value = "<!-- feature-manifest:end -->",
            help = "End marker used by `--insert-into`."
        )]
        end_marker: String,
    },
    /// Emit normalized machine-readable feature metadata as JSON.
    #[command(visible_aliases = ["j"])]
    Json,
    /// Render a Mermaid graph of feature relationships.
    #[command(visible_aliases = ["g", "viz"])]
    Graph {
        #[arg(
            short = 'a',
            long,
            action = ArgAction::SetTrue,
            help = "Include private/internal features in the output."
        )]
        include_private: bool,
    },
    /// Scaffold missing metadata entries directly into Cargo.toml.
    #[command(visible_aliases = ["s"])]
    Sync {
        #[arg(
            short = 'c',
            long,
            action = ArgAction::SetTrue,
            help = "Exit non-zero if changes would be needed, without rewriting files."
        )]
        check: bool,
        #[arg(
            short = 'r',
            long,
            action = ArgAction::SetTrue,
            help = "Remove stale metadata entries for missing features."
        )]
        remove_stale: bool,
        #[arg(
            short = 's',
            long,
            value_enum,
            help = "Choose the metadata layout to write back."
        )]
        style: Option<SyncStyle>,
    },
    /// Explain one feature in human-readable form.
    #[command(visible_aliases = ["show", "x"])]
    Explain {
        feature: String,
        #[arg(
            short = 'a',
            long,
            action = ArgAction::SetTrue,
            help = "Include private/internal features when searching for matches."
        )]
        include_private: bool,
    },
    /// List the lint codes supported by `check`.
    #[command(visible_aliases = ["lints"])]
    ListLints,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum CheckFormat {
    Text,
    Json,
    Github,
    Sarif,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum SyncStyle {
    Flat,
    Structured,
}

impl From<SyncStyle> for MetadataLayout {
    fn from(value: SyncStyle) -> Self {
        match value {
            SyncStyle::Flat => MetadataLayout::Flat,
            SyncStyle::Structured => MetadataLayout::Structured,
        }
    }
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum LintPresetArg {
    Adopt,
    Strict,
}

impl From<LintPresetArg> for LintPreset {
    fn from(value: LintPresetArg) -> Self {
        match value {
            LintPresetArg::Adopt => LintPreset::Adopt,
            LintPresetArg::Strict => LintPreset::Strict,
        }
    }
}

pub fn cli_main() {
    if let Err(error) = run() {
        eprintln!("error: {error:#}");
        process::exit(1);
    }
}

fn run() -> Result<()> {
    let cli = Cli::parse_from(normalize_args(std::env::args_os()));
    let selection = selection_from_cli(&cli)?;
    let manifest_path = resolve_manifest_path(cli.manifest_path.as_deref())?;
    let workspace = load_workspace(&manifest_path, selection.clone())?;

    match cli.command.unwrap_or(Command::Check {
        format: CheckFormat::Text,
        lint_overrides: Vec::new(),
        preset: None,
    }) {
        Command::Init {
            readme,
            no_readme,
            ci,
            style,
        } => commands::init::run(
            &workspace,
            commands::init::InitOptions {
                manifest_path,
                selection,
                readme,
                no_readme,
                ci,
                style: style.map(Into::into),
            },
        ),
        Command::Doctor { readme } => {
            commands::doctor::run(&workspace, commands::doctor::DoctorOptions { readme })
        }
        Command::Check {
            format,
            lint_overrides,
            preset,
        } => commands::check::run(&workspace, format, &lint_overrides, preset.map(Into::into)),
        Command::Markdown {
            include_private,
            write,
            insert_into,
            check,
            start_marker,
            end_marker,
        } => commands::markdown::run(
            &workspace,
            include_private,
            write,
            insert_into,
            check,
            InjectionMarkers {
                start: start_marker,
                end: end_marker,
            },
        ),
        Command::Json => {
            println!("{}", render_json(&workspace)?);
            Ok(())
        }
        Command::Graph { include_private } => {
            println!("{}", render_mermaid(&workspace, include_private));
            Ok(())
        }
        Command::Sync {
            check,
            remove_stale,
            style,
        } => commands::sync::run(
            &workspace,
            SyncOptions {
                check_only: check,
                remove_stale,
                style: style.map(Into::into),
            },
        ),
        Command::Explain {
            feature,
            include_private,
        } => {
            println!("{}", render_explain(&workspace, &feature, include_private)?);
            Ok(())
        }
        Command::ListLints => {
            for code in known_lint_codes() {
                println!("{code}");
            }
            Ok(())
        }
    }
}

fn selection_from_cli(cli: &Cli) -> Result<PackageSelection> {
    if cli.workspace && cli.package.is_some() {
        bail!("`--workspace` and `--package` cannot be used together");
    }

    if cli.workspace {
        return Ok(PackageSelection::Workspace);
    }

    if let Some(package_name) = &cli.package {
        return Ok(PackageSelection::Package(package_name.clone()));
    }

    Ok(PackageSelection::Default)
}

fn normalize_args(args: impl IntoIterator<Item = OsString>) -> Vec<OsString> {
    let mut args = args.into_iter().collect::<Vec<_>>();

    if args
        .get(1)
        .and_then(|argument| argument.to_str())
        .is_some_and(|argument| {
            argument == "feature-manifest" || argument == "feature_manifest" || argument == "fm"
        })
    {
        args.remove(1);
    }

    args
}
