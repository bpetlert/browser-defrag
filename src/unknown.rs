use std::path::PathBuf;

use anyhow::Result;

use crate::{
    common::find_sqlite3_files,
    defrag::{Config, Database, Profile},
};

/// Database listing function for Unknown browser
pub fn list_db(config: Config) -> Result<Vec<Profile>> {
    // Search all sqlite3 files
    let database_files: Vec<PathBuf> =
        find_sqlite3_files(config.profile_path.as_ref().unwrap(), config.max_depth)?;

    let database_lists: Vec<Profile> = vec![Profile {
        name: "".to_string(),
        path: config.profile_path.unwrap(),
        databases: Some(
            database_files
                .into_iter()
                .map(|path| Database::new(&path))
                .collect::<Vec<Database>>(),
        ),
    }];

    Ok(database_lists)
}
