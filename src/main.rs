mod args;
mod chromium;
mod common;
mod defrag;
mod firefox;
mod report;

use std::{
    io::{self, Write},
    process::ExitCode,
};

use anyhow::{anyhow, Context, Result};
use clap::Parser;
use tracing::{debug, error};
use tracing_subscriber::EnvFilter;

use crate::{
    args::Arguments,
    defrag::{Browser, Config, Defragment},
};

fn run() -> Result<()> {
    let filter =
        EnvFilter::try_from_default_env().unwrap_or(EnvFilter::try_new("browser_defrag=warn")?);
    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .without_time()
        .with_writer(io::stderr)
        .try_init()
        .map_err(|err| anyhow!("{err:#}"))
        .context("Failed to initialize tracing subscriber")?;

    let arguments = Arguments::parse();
    debug!("Run with {:?}", arguments);

    match arguments.browser {
        args::BrowserType::Firefox => {
            let config = Config {
                max_depth: arguments.max_depth,
                profile_path: None,
            };

            let mut browser = Browser::new("Firefox");
            browser.list_databases(firefox::list_db, config)?;
            browser.defrag(arguments.dry_run)?;
            let mut stdout = io::BufWriter::new(io::stdout().lock());
            writeln!(stdout, "{browser}")?;
        }
        args::BrowserType::Chromium => {
            let config = Config {
                max_depth: arguments.max_depth,
                profile_path: None,
            };

            let mut browser = Browser::new("Chromium");
            browser.list_databases(chromium::list_db, config)?;
            browser.defrag(arguments.dry_run)?;
            let mut stdout = io::BufWriter::new(io::stdout().lock());
            writeln!(stdout, "{browser}")?;
        }
        args::BrowserType::Unknown { profile_path: _ } => todo!(),
    }

    Ok(())
}

fn main() -> ExitCode {
    if let Err(err) = run() {
        error!("{err:#}");
        return ExitCode::FAILURE;
    }
    ExitCode::SUCCESS
}
