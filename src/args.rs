use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
pub struct Arguments {
    #[command(subcommand)]
    pub browser: BrowserType,

    /// Set the maximum searching of descendent directory
    #[arg(long, value_name = "DEPTH", global = true, default_value_t = 2)]
    pub max_depth: usize,

    /// Show list of database files without defragging
    #[arg(long, global = true)]
    pub dry_run: bool,
}

#[derive(Debug, Subcommand)]
pub enum BrowserType {
    #[command(about = "Firefox or Firefox Developer Edition")]
    Firefox,

    #[command(about = "Chromium")]
    Chromium,

    #[command(about = "Unknown browser")]
    Unknown {
        /// Profile's path of unknown browser
        #[arg(long, value_name = "PATH", required = true)]
        profile_path: PathBuf,
    },
}
