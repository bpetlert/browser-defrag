use std::{env, path::PathBuf};

use anyhow::{anyhow, bail, Result};
use configparser::ini::Ini;
use tracing::debug;
use walkdir::WalkDir;

pub fn list_db() -> Result<(PathBuf, Vec<PathBuf>)> {
    let profile_path: PathBuf = {
        let firefox_root: PathBuf = PathBuf::from(&env::var("HOME")?)
            .join(".mozilla")
            .join("firefox");

        let profile_ini = firefox_root.join("profiles.ini");
        debug!("Firefox's profile file = `{}`", profile_ini.display());

        if !profile_ini.exists() {
            bail!(
                "Firefox's profile file `{}` is not exist",
                profile_ini.display()
            );
        }

        let mut config = Ini::new();
        config.load(profile_ini).map_err(|err| anyhow!("{err}"))?;
        debug!("Firefox's profile = `{:?}`", config.get_map());

        let profile = config
            .get("profile0", "path")
            .ok_or_else(|| anyhow!("Failed to get Firefox's profile path"))?;
        firefox_root.join(profile)
    };

    let database_files: Vec<PathBuf> = WalkDir::new(&profile_path)
        .into_iter()
        .filter_entry(|entry| {
            entry
                .file_name()
                .to_str()
                .map(|s| s.ends_with(".sqlite"))
                .unwrap_or(false)
        })
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.into_path())
        .collect();

    Ok((profile_path, database_files))
}
