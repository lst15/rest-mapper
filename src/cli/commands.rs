use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[command(
    name = "flowtrace",
    version,
    about = "Mapeamento causal de fluxos frontend/backend"
)]
pub struct FlowtraceCli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    Record(RecordArgs),
    Analyze(AnalyzeArgs),
    Report(ReportArgs),
    Open(OpenArgs),
}

#[derive(Debug, Clone, Parser)]
pub struct RecordArgs {
    #[arg(long)]
    pub url: String,
    #[arg(long, default_value = "reports")]
    pub reports_dir: PathBuf,
    #[arg(long, default_value = "collector/src/index.js")]
    pub collector_script: PathBuf,
    #[arg(long, default_value = "node")]
    pub node_bin: String,
    #[arg(long, default_value = "chromium")]
    pub browser: String,
    #[arg(long)]
    pub headless: bool,
    #[arg(long, default_value_t = 30)]
    pub shutdown_timeout_secs: u64,
}

#[derive(Debug, Clone, Parser)]
pub struct AnalyzeArgs {
    #[arg(long)]
    pub input: PathBuf,
    #[arg(long)]
    pub output: Option<PathBuf>,
}

#[derive(Debug, Clone, Parser)]
pub struct ReportArgs {
    #[arg(long)]
    pub input: PathBuf,
    #[arg(long)]
    pub output: Option<PathBuf>,
}

#[derive(Debug, Clone, Parser)]
pub struct OpenArgs {
    #[arg(long)]
    pub session: String,
    #[arg(long)]
    pub path: Option<PathBuf>,
}
