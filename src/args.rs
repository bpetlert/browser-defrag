use clap::Parser;

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
pub struct Arguments {
    #[arg(name = "BROWSER", value_enum, ignore_case = true, required = true)]
    pub browser: BrowserName,

    #[arg(long)]
    pub dry_run: bool,
}

#[derive(Debug, Clone, clap::ValueEnum)]
pub enum BrowserName {
    Firefox,
}
