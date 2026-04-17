mod app;
mod cli;
mod collector;
mod correlator;
mod domain;
mod error;
mod report;
mod storage;
mod utils;

use anyhow::Result;
use clap::Parser;
use cli::commands::FlowtraceCli;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = FlowtraceCli::parse();
    cli::run(cli).await
}
