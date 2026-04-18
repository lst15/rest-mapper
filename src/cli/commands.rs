use clap::{Parser, Subcommand, ValueEnum};
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
    #[arg(
        long = "event-type",
        value_name = "EVENT_TYPE",
        value_enum,
        value_delimiter = ',',
        num_args = 1..,
        help = "Tipos de evento para coletar no raw_trace (repita a flag ou use CSV). Se omitido, coleta todos."
    )]
    pub event_types: Vec<TraceEventType>,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum TraceEventType {
    #[value(
        name = "BrowserOpened",
        alias = "browser-opened",
        alias = "browseropened"
    )]
    BrowserOpened,
    #[value(
        name = "PageNavigated",
        alias = "page-navigated",
        alias = "pagenavigated"
    )]
    PageNavigated,
    #[value(name = "RouteChanged", alias = "route-changed", alias = "routechanged")]
    RouteChanged,
    #[value(name = "UserAction", alias = "user-action", alias = "useraction")]
    UserAction,
    #[value(
        name = "NetworkRequest",
        alias = "network-request",
        alias = "networkrequest"
    )]
    NetworkRequest,
    #[value(
        name = "NetworkResponse",
        alias = "network-response",
        alias = "networkresponse"
    )]
    NetworkResponse,
    #[value(name = "ConsoleLog", alias = "console-log", alias = "consolelog")]
    ConsoleLog,
    #[value(
        name = "DomSnapshotMarker",
        alias = "dom-snapshot-marker",
        alias = "domsnapshotmarker"
    )]
    DomSnapshotMarker,
    #[value(
        name = "BrowserClosed",
        alias = "browser-closed",
        alias = "browserclosed"
    )]
    BrowserClosed,
}

impl TraceEventType {
    pub fn as_event_name(self) -> &'static str {
        match self {
            TraceEventType::BrowserOpened => "BrowserOpened",
            TraceEventType::PageNavigated => "PageNavigated",
            TraceEventType::RouteChanged => "RouteChanged",
            TraceEventType::UserAction => "UserAction",
            TraceEventType::NetworkRequest => "NetworkRequest",
            TraceEventType::NetworkResponse => "NetworkResponse",
            TraceEventType::ConsoleLog => "ConsoleLog",
            TraceEventType::DomSnapshotMarker => "DomSnapshotMarker",
            TraceEventType::BrowserClosed => "BrowserClosed",
        }
    }
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
