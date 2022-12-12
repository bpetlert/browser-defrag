use std::{env, path::PathBuf};

use anyhow::{anyhow, bail, Result};
use configparser::ini::Ini;
use tracing::debug;
use walkdir::WalkDir;

use crate::common::is_sqlite_file;

/// Database listing function for Firefox and Firefox Developer Edition
///
/// Return `(profile-path, list-of-database-files)`
///
// TODO: Support multiple profiles
pub fn list_db() -> Result<(PathBuf, Vec<PathBuf>)> {
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

    // Search all file *.sqlite or *.db and filter out non-sqlite3 files
    let database_files: Vec<PathBuf> = WalkDir::new(&profile_path)
        .min_depth(1)
        .max_depth(2)
        .sort_by(|a, b| a.file_name().cmp(b.file_name()))
        .into_iter()
        .filter_entry(|entry| {
            entry
                .file_name()
                .to_str()
                .map(|s| s.ends_with(".sqlite") || s.ends_with(".db"))
                .unwrap_or(false)
        })
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.into_path())
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
