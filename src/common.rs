use std::{
    fs::File,
    io::Read,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};
use tracing::{debug, trace};
use walkdir::WalkDir;

/// Check whether a file is valid sqlite3 or not.
///
/// Check whether first 16 bytes of a file contains "SQLite format 3\000" or not.
///
/// See: https://www.sqlite.org/fileformat.html
pub fn is_sqlite3_file(path: &Path) -> Result<bool> {
    let header_size: usize = 16;
    let mut header: Vec<u8> = Vec::with_capacity(header_size);

    let mut file =
        File::open(path).with_context(|| format!("Could not open `{}`", path.display()))?;

    file.by_ref()
        .take(header_size as u64)
        .read_to_end(&mut header)
        .with_context(|| format!("Could not read header string of `{}`", path.display()))?;

    match std::str::from_utf8(&header).with_context(|| {
        format!(
            "Failed to convert string header `{:?}` of `{}` to UTF8",
            header,
            path.display()
        )
    }) {
        Ok(h) => {
            if h == "SQLite format 3\x00" {
                return Ok(true);
            }
        }
        Err(err) => {
            trace!("{err:#}");
            return Ok(false);
        }
    }

    Ok(false)
}

/// Find all sqlite3 files in `root`
pub fn find_sqlite3_files(root: &Path, max_depth: usize) -> Result<Vec<PathBuf>> {
    let database_files: Vec<PathBuf> = WalkDir::new(root)
        .max_depth(max_depth)
        .into_iter()
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.into_path())
        .filter(|path| path.metadata().is_ok_and(|metadata| metadata.is_file()))
        .filter(|db| match is_sqlite3_file(db) {
            Ok(is_sqlite) => is_sqlite,
            Err(err) => {
                debug!("{err:#}");
                false
            }
        })
        .collect();

    Ok(database_files)
}
