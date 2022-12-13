use std::path::PathBuf;

use anyhow::Result;

use crate::{
    common::find_sqlite3_files,
    defrag::{Config, Database, DatabaseList},
};

/// Database listing function for Unknown browser
pub fn list_db(config: Config) -> Result<Vec<DatabaseList>> {
    // Search all sqlite3 files
    let database_files: Vec<PathBuf> =
        find_sqlite3_files(config.profile_path.as_ref().unwrap(), config.max_depth)?;

    let database_lists: Vec<DatabaseList> = vec![DatabaseList {
        profile_path: config.profile_path.unwrap(),
        databases: database_files
            .into_iter()
            .map(|path| Database::new(&path))
            .collect::<Vec<Database>>(),
    }];

    Ok(database_lists)
}
