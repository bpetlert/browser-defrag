use std::{env, path::PathBuf};

use anyhow::Result;

use crate::{
    common::find_sqlite3_files,
    defrag::{Config, Database, Profile},
};

/// Database listing function for Chromium
pub fn list_db(config: Config) -> Result<Vec<Profile>> {
    let profile_path: PathBuf = {
        let config_root: PathBuf = match env::var("XDG_CONFIG_HOME") {
            Ok(var) => PathBuf::from(var),
            Err(_) => PathBuf::from(env::var("HOME")?).join(".config"),
        };

        config_root.join("chromium")
    };

    // Search all sqlite3 files
    let database_files: Vec<PathBuf> = find_sqlite3_files(&profile_path, config.max_depth)?;

    let database_lists: Vec<Profile> = vec![Profile {
        name: "".to_string(),
        path: profile_path,
        databases: Some(
            database_files
                .into_iter()
                .map(|path| Database::new(&path))
                .collect::<Vec<Database>>(),
        ),
    }];

    Ok(database_lists)
}
