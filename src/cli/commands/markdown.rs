use std::path::PathBuf;

use anyhow::{Result, bail};

use crate::{
    InjectionMarkers, WorkspaceManifest, inject_between_markers, injected_region_matches,
    output_matches, render_markdown, write_output,
};

pub fn run(
    workspace: &WorkspaceManifest,
    include_private: bool,
    write: Option<PathBuf>,
    insert_into: Option<PathBuf>,
    check: bool,
    markers: InjectionMarkers,
) -> Result<()> {
    let markdown = render_markdown(workspace, include_private);

    if check {
        return check_outputs(&markdown, write, insert_into, &markers);
    }

    let mut wrote_somewhere = false;

    if let Some(path) = write {
        write_output(&path, &markdown)?;
        println!("wrote Markdown to `{}`", path.display());
        wrote_somewhere = true;
    }

    if let Some(path) = insert_into {
        inject_between_markers(&path, &markdown, &markers)?;
        println!("injected Markdown into `{}`", path.display());
        wrote_somewhere = true;
    }

    if !wrote_somewhere {
        println!("{markdown}");
    }

    Ok(())
}

fn check_outputs(
    markdown: &str,
    write: Option<PathBuf>,
    insert_into: Option<PathBuf>,
    markers: &InjectionMarkers,
) -> Result<()> {
    if write.is_none() && insert_into.is_none() {
        bail!("`md --check` requires `--write <PATH>` or `--insert-into <PATH>`");
    }

    let mut stale = Vec::new();
    let mut next_actions = Vec::new();

    if let Some(path) = write {
        if output_matches(&path, markdown)? {
            println!("`{}` is up to date", path.display());
        } else {
            println!("`{}` is stale", path.display());
            next_actions.push(format!("`cargo fm markdown --write {}`", path.display()));
            stale.push(path);
        }
    }

    if let Some(path) = insert_into {
        if injected_region_matches(&path, markdown, markers)? {
            println!("`{}` injected region is up to date", path.display());
        } else {
            println!("`{}` injected region is stale", path.display());
            next_actions.push(format!(
                "`cargo fm markdown --insert-into {}`",
                path.display()
            ));
            stale.push(path);
        }
    }

    if !stale.is_empty() {
        if next_actions.len() == 1 {
            bail!(
                "generated Markdown is stale; run {} to update it",
                next_actions[0]
            );
        }
        bail!(
            "generated Markdown is stale; rerun the markdown command without `--check` to update it"
        );
    }

    Ok(())
}
