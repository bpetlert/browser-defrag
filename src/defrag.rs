use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::{bail, Context, Result};
use humansize::{format_size_i, DECIMAL};
use sysinfo::{ProcessExt, System, SystemExt};
use tempfile::tempdir;
use tracing::error;

#[derive(Debug)]
pub struct Browser {
    name: String,
    profile_path: Option<PathBuf>,
    databases: Option<Vec<Database>>,
}

#[derive(Debug)]
pub struct Database {
    path: PathBuf,
    profile_path: PathBuf,
    size_before: Option<u64>,
    size_after: Option<u64>,
    defrag: bool,
}

pub trait Defragment {
    /// Vacuum and reindex databases
    fn defrag(&mut self, dry_run: bool) -> Result<()>;
}

impl Browser {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            profile_path: None,
            databases: None,
        }
    }

    /// Update the list of database files
    ///
    /// `func` - Specificed database listing function of a browser,
    /// which returns `(profile-path, list-of-fullpath-of-database-files)`
    pub fn list_databases<F>(&mut self, func: F) -> Result<()>
    where
        F: FnOnce() -> Result<(PathBuf, Vec<PathBuf>)>,
    {
        self.databases = Some(Vec::new());

        let (profile_path, db_paths) = func()?;
        self.profile_path = Some(profile_path.clone());

        for path in db_paths {
            self.databases
                .as_mut()
                .unwrap()
                .push(Database::new(&profile_path, &path));
        }

        Ok(())
    }
}

impl Database {
    pub fn new(profile_fullpath: &Path, db_fullpath: &Path) -> Self {
        Self {
            path: db_fullpath.to_path_buf(),
            profile_path: profile_fullpath.to_path_buf(),
            size_before: None,
            size_after: None,
            defrag: false,
        }
    }

    pub fn relative_path(&self) -> Result<PathBuf> {
        Ok(self.path.strip_prefix(&self.profile_path)?.to_path_buf())
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

        if self.databases.is_none() {
            bail!("Database list is empty");
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

impl std::fmt::Display for Browser {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        if self.profile_path.is_none() {
            return write!(f, "{}: NO PROFILE FOUND", self.name);
        }

        if self.databases.is_none() || self.databases.as_ref().unwrap().is_empty() {
            return write!(
                f,
                "{browser_name}: {profile_path}/\nNO DATABASE FOUND",
                browser_name = self.name,
                profile_path = self.profile_path.as_ref().unwrap().display()
            );
        }

        let databases: String = self
            .databases
            .as_ref()
            .unwrap()
            .iter()
            .map(|db| db.to_string())
            .collect::<Vec<String>>()
            .join("\n");

        let total_before: f64 = self
            .databases
            .as_ref()
            .unwrap()
            .iter()
            .map(|db| db.size_before.unwrap_or(0))
            .sum::<u64>() as f64;

        let total_after: f64 = self
            .databases
            .as_ref()
            .unwrap()
            .iter()
            .map(|db| db.size_after.unwrap_or(0))
            .sum::<u64>() as f64;

        let size_diff: f64 = total_after - total_before;
        let percent: f64 = size_diff * 100.0_f64 / (total_before);

        let total_before = format_size_i(total_before, DECIMAL);

        let total_diff_after = format!(
            "( {diff} ) {after}",
            diff = format_size_i(size_diff, DECIMAL),
            after = format_size_i(total_after, DECIMAL),
        );

        let percent = format!("{:.2} %", percent);

        write!(
            f,
            "{browser_name}: {profile_path}/\n{databases}\n{total_before:>total_col2$} => {total_diff_after:<total_col3$} {percent:>total_col4$}",
            browser_name = self.name,
            profile_path = self.profile_path.as_ref().unwrap().display(),
            total_col2 = 71,
            total_col3 = 40,
            total_col4 = 20,
        )
    }
}

impl std::fmt::Display for Database {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let size_before: f64 = self.size_before.unwrap() as f64;
        let size_after: f64 = self.size_after.unwrap() as f64;
        let size_diff: f64 = size_after - size_before;
        let percent: f64 = size_diff * 100.0_f64 / size_before;

        let size_before = format_size_i(size_before, DECIMAL);

        let size_diff_after = format!(
            "( {diff} ) {after}",
            diff = format_size_i(size_diff, DECIMAL),
            after = format_size_i(size_after, DECIMAL)
        );

        let percent = format!("{:.2} %", percent);

        write!(
            f,
            "{db_file:<col1_width$} {size_before:>col2_width$} => {size_diff_after:<col3_width$} {percent:>col4_width$}",
            db_file = self.relative_path().unwrap().display(),
            col1_width = 50,
            col2_width = 20,
            col3_width = 40,
            col4_width = 20,
        )
    }
}
