use std::{
    env,
    path::{Path, PathBuf},
};

use anyhow::{anyhow, bail, Result};
use configparser::ini::Ini;
use tracing::{debug, error};

use crate::{
    common::find_sqlite3_files,
    defrag::{Config, Database, Profile},
};

/// Database listing function for Firefox and Firefox Developer Edition
pub fn list_db(config: Config) -> Result<Vec<Profile>> {
    let mut profiles: Vec<Profile> = {
        // Firefox profile's root at $HOME/.mozilla/firefox
        let firefox_root: PathBuf = PathBuf::from(&env::var("HOME")?)
            .join(".mozilla")
            .join("firefox");

        // $HOME/.mozilla/firefox/profiles.ini
        let profiles_ini = firefox_root.join("profiles.ini");
        debug!("Firefox's profile file = `{}`", profiles_ini.display());

        load_profiles(&profiles_ini)?
    };

    // Search all sqlite3 files for each profile
    for profile in profiles.iter_mut() {
        let database_files: Vec<PathBuf> = find_sqlite3_files(&profile.path, config.max_depth)?;

        profile.databases = Some(
            database_files
                .into_iter()
                .map(|path| Database::new(&path))
                .collect::<Vec<Database>>(),
        );
    }

    Ok(profiles)
}

/// Load Firefox's profile
///
/// See: https://kb.mozillazine.org/Profiles.ini_file
pub fn load_profiles(profiles_ini: &Path) -> Result<Vec<Profile>> {
    if !profiles_ini.exists() {
        bail!(
            "Firefox's profile file `{}` is not exist",
            profiles_ini.display()
        );
    }

    // Load configurations from `profiles.ini`
    let mut config = Ini::new();
    config.load(profiles_ini).map_err(|err| anyhow!("{err}"))?;
    debug!("Firefox's profile = `{:?}`", config.get_map());

    let profile_root = profiles_ini.parent().unwrap();
    let mut profiles: Vec<Profile> = Vec::new();
    let mut profile_index: u32 = 0;
    let mut section = format!("Profile{profile_index}");

    while let Some(name) = config.get(&section, "Name") {
        let is_relative = match config.getboolcoerce(&section, "IsRelative") {
            Ok(value) => match value {
                Some(v) => v,
                None => {
                    error!("Cannot read `IsRelative` of `{section}`");
                    continue;
                }
            },
            Err(err) => {
                error!("Cannot read `IsRelative` of `{section}`: {err:#}");
                continue;
            }
        };

        let Some(profile_path) = config
            .get(&section, "Path") else {
                error!("Cannot read `Path` of `{section}`");
                continue;
            };

        let profile_path: PathBuf = if is_relative {
            profile_root.join(Path::new(&profile_path))
        } else {
            Path::new(&profile_path).to_path_buf()
        };

        profiles.push(Profile {
            name,
            path: profile_path,
            databases: None,
        });

        profile_index += 1;
        section = format!("Profile{profile_index}");
    }

    Ok(profiles)
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_load_single_profile() {
        let profiles = load_profiles(Path::new(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/",
            "tests/",
            "single-profiles.ini"
        )))
        .unwrap();

        let expected_profiles: Vec<Profile> = vec![Profile {
            name: "default".to_string(),
            path: PathBuf::from(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/",
                "tests/",
                "qioxtndq.default",
            )),
            databases: None,
        }];

        assert_eq!(profiles, expected_profiles);
    }

    #[test]
    fn test_load_multiple_profiles() {
        let profiles = load_profiles(Path::new(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/",
            "tests/",
            "multiple-profiles.ini"
        )))
        .unwrap();

        let expected_profiles: Vec<Profile> = vec![
            Profile {
                name: "default".to_string(),
                path: PathBuf::from(concat!(
                    env!("CARGO_MANIFEST_DIR"),
                    "/",
                    "tests/",
                    "qioxtndq.default"
                )),
                databases: None,
            },
            Profile {
                name: "alicew".to_string(),
                path: PathBuf::from("/home/user/.mozilla/firefox/alicew"),
                databases: None,
            },
            Profile {
                name: "sheldon".to_string(),
                path: PathBuf::from("/home/user/.mozilla/firefox/sheldon"),
                databases: None,
            },
        ];

        assert_eq!(profiles, expected_profiles);
    }
}
