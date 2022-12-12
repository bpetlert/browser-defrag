use std::{env, path::PathBuf};

use anyhow::Result;
use tracing::debug;
use walkdir::WalkDir;

use crate::common::is_sqlite_file;

/// Database listing function for Chromium
///
/// Return `(profile-path, list-of-fullpath-of-database-files)`
///
pub fn list_db() -> Result<(PathBuf, Vec<PathBuf>)> {
    let profile_path: PathBuf = {
        let config_root: PathBuf = match env::var("XDG_CONFIG_HOME") {
            Ok(var) => PathBuf::from(var),
            Err(_) => PathBuf::from(env::var("HOME")?).join(".config"),
        };

        config_root.join("chromium")
    };

    // Search all file *.sqlite or *.db and filter out non-sqlite3 files
    let database_files: Vec<PathBuf> = WalkDir::new(&profile_path)
        .max_depth(2)
        .into_iter()
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.into_path())
        .filter(|path| path.metadata().map_or(false, |metadata| metadata.is_file()))
        .filter(|db| match is_sqlite_file(db) {
            Ok(is_sqlite) => is_sqlite,
            Err(err) => {
                debug!("{err:#}");
                false
            }
        })
        .collect();

    Ok((profile_path, database_files))
}
