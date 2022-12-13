use std::{env, path::PathBuf};

use anyhow::{anyhow, bail, Result};
use configparser::ini::Ini;
use tracing::debug;

use crate::{
    common::find_sqlite3_files,
    defrag::{Config, Database, DatabaseList},
};

/// Database listing function for Firefox and Firefox Developer Edition
// TODO: Support multiple profiles
pub fn list_db(config: Config) -> Result<Vec<DatabaseList>> {
    let profile_path: PathBuf = {
        // Firefox profile's root at $HOME/.mozilla/firefox
        let firefox_root: PathBuf = PathBuf::from(&env::var("HOME")?)
            .join(".mozilla")
            .join("firefox");

        // $HOME/.mozilla/firefox/profiles.ini
        let profile_ini = firefox_root.join("profiles.ini");
        debug!("Firefox's profile file = `{}`", profile_ini.display());

        if !profile_ini.exists() {
            bail!(
                "Firefox's profile file `{}` is not exist",
                profile_ini.display()
            );
        }

        // Load configurations from `profiles.ini`
        let mut config = Ini::new();
        config.load(profile_ini).map_err(|err| anyhow!("{err}"))?;
        debug!("Firefox's profile = `{:?}`", config.get_map());

        // Get profile0's path
        let profile = config
            .get("profile0", "path")
            .ok_or_else(|| anyhow!("Failed to get Firefox's profile path"))?;
        firefox_root.join(profile)
    };

    // Search all sqlite3 files
    let database_files: Vec<PathBuf> = find_sqlite3_files(&profile_path, config.max_depth)?;

    let database_lists: Vec<DatabaseList> = vec![DatabaseList {
        profile_path,
        databases: database_files
            .into_iter()
            .map(|path| Database::new(&path))
            .collect::<Vec<Database>>(),
    }];

    Ok(database_lists)
}
