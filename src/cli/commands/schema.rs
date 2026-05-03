use std::path::PathBuf;

use anyhow::Result;

use crate::write_output;

#[derive(Debug, Clone, Copy)]
pub enum SchemaKind {
    Metadata,
    CheckReport,
}

impl SchemaKind {
    fn contents(self) -> &'static str {
        match self {
            Self::Metadata => include_str!("../../../schemas/feature-manifest.v1.schema.json"),
            Self::CheckReport => include_str!("../../../schemas/check-report.v1.schema.json"),
        }
    }
}

pub fn run(kind: SchemaKind, write: Option<PathBuf>) -> Result<()> {
    let contents = kind.contents();

    if let Some(path) = write {
        write_output(&path, contents)?;
        println!("wrote JSON Schema to `{}`", path.display());
    } else {
        print!("{contents}");
    }

    Ok(())
}
