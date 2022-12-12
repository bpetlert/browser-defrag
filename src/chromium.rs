use std::{env, path::PathBuf};

use anyhow::Result;

use crate::common::find_sqlite3_files;

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

    // Search all sqlite3 files
    let database_files: Vec<PathBuf> = find_sqlite3_files(&profile_path, 2)?;

    Ok((profile_path, database_files))
}
