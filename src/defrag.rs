use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::{bail, Context, Result};
use sysinfo::{ProcessExt, System, SystemExt};
use tempfile::tempdir;
use tracing::error;

#[derive(Debug)]
pub struct Browser {
    pub name: String,
    pub database_lists: Option<Vec<Profile>>,
}

#[derive(Debug, PartialEq)]
pub struct Profile {
    pub name: String,
    pub path: PathBuf,
    pub databases: Option<Vec<Database>>,
}

#[derive(Debug, PartialEq)]
pub struct Database {
    pub path: PathBuf,
    pub size_before: Option<u64>,
    pub size_after: Option<u64>,
    pub defrag: bool,
}

#[derive(Debug)]
pub struct Config {
    pub max_depth: usize,
    pub profile_path: Option<PathBuf>,
}

pub trait Defragment {
    /// Vacuum and reindex databases
    fn defrag(&mut self, dry_run: bool) -> Result<()>;
}

impl Browser {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            database_lists: None,
        }
    }

    /// Update the list of database files
    ///
    /// `func` - Specificed database listing function of a browser
    pub fn list_databases<F>(&mut self, func: F, config: Config) -> Result<()>
    where
        F: FnOnce(Config) -> Result<Vec<Profile>>,
    {
        self.database_lists = Some(func(config)?);
        Ok(())
    }
}

impl Database {
    pub fn new(path: &Path) -> Self {
        Self {
            path: path.to_path_buf(),
            size_before: None,
            size_after: None,
            defrag: false,
        }
    }

    pub fn database_size(&self) -> Result<u64> {
        Ok(self.path.metadata()?.len())
    }
}

impl Defragment for Browser {
    fn defrag(&mut self, dry_run: bool) -> Result<()> {
        if !dry_run {
            // Check if browser is running?
            let mut sys = System::new();
            sys.refresh_processes();
            if sys
                .processes()
                .values()
                .into_iter()
                .any(|p| p.name().to_lowercase().contains(&self.name.to_lowercase()))
            {
                bail!("Cannot defrag. `{}` is running!!!", self.name);
            }
        }

        if self.database_lists.is_none() {
            bail!("Database list is empty");
        }

        for dbs in self.database_lists.as_mut().unwrap().iter_mut() {
            dbs.defrag(dry_run)?;
        }

        Ok(())
    }
}

impl Defragment for Profile {
    fn defrag(&mut self, dry_run: bool) -> Result<()> {
        if self.databases.is_none() {
            return Ok(());
        }

        for db in self.databases.as_mut().unwrap().iter_mut() {
            if let Err(err) = db.defrag(dry_run) {
                error!("{err:#}");
            }
        }

        Ok(())
    }
}

impl Defragment for Database {
    fn defrag(&mut self, dry_run: bool) -> Result<()> {
        if !self.path.exists() {
            bail!("Database file `{}` is not exist", self.path.display());
        }

        // Size of database before defrag
        match self.database_size() {
            Ok(file_size) => self.size_before = Some(file_size),
            Err(err) => bail!("{err:#}"),
        };

        if dry_run {
            self.size_after = self.size_before;
            return Ok(());
        }

        // Copy database file to TMPDIR before defrag
        let tmp_dir = tempdir()?;
        let dp_copy = tmp_dir.path().join(self.path.file_name().unwrap());
        fs::copy(&self.path, &dp_copy)?;

        // Open database file
        let connection = match sqlite::open(&dp_copy)
            .with_context(|| format!("Failed to open database `{}`", self.path.display()))
        {
            Ok(connection) => connection,
            Err(err) => bail!("{err:#}"),
        };

        // VACUUM
        if let Err(err) = connection
            .execute("VACUUM;")
            .with_context(|| format!("Failed to vacuum database `{}`", self.path.display()))
        {
            drop(connection);
            bail!("{err:#}");
        }

        // REINDEX
        if let Err(err) = connection
            .execute("REINDEX;")
            .with_context(|| format!("Failed to reindex database `{}`", self.path.display()))
        {
            drop(connection);
            bail!("{err:#}");
        }

        // Copy database file from TMPDIR to original location if file size smaller than original
        if dp_copy.metadata()?.len() < self.size_before.unwrap() {
            fs::copy(&dp_copy, &self.path)?;
        }

        // Size of database after defrag
        match self.database_size() {
            Ok(size_after) => self.size_after = Some(size_after),
            Err(err) => bail!("{err:#}"),
        };

        self.defrag = true;
        drop(connection);

        Ok(())
    }
}
