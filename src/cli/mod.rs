pub mod commands;

use anyhow::{Context, Result};
use std::{
    path::{Path, PathBuf},
    process::Command,
};

use crate::{
    app::run_session,
    cli::commands::{AnalyzeArgs, Commands, FlowtraceCli, OpenArgs, RecordArgs, ReportArgs},
    collector::event_ingest,
    correlator::engine::CorrelatorEngine,
    report::{html, json},
};

pub async fn run(cli: FlowtraceCli) -> Result<()> {
    match cli.command {
        Commands::Record(args) => run_record_command(args).await,
        Commands::Analyze(args) => run_analyze_command(args),
        Commands::Report(args) => run_report_command(args),
        Commands::Open(args) => run_open_command(args),
    }
}

async fn run_record_command(args: RecordArgs) -> Result<()> {
    run_session::run_record_session(args).await
}

fn run_analyze_command(args: AnalyzeArgs) -> Result<()> {
    let events = event_ingest::ingest_events(&args.input)
        .with_context(|| format!("falha ao ler {}", args.input.display()))?;
    let correlated = CorrelatorEngine::default().correlate(&events);

    let output_path = args.output.unwrap_or_else(|| {
        args.input
            .parent()
            .unwrap_or(Path::new("."))
            .join("correlated_trace.json")
    });

    json::write_correlated_trace(&output_path, &correlated)?;
    println!(
        "[flowtrace] Correlacionado salvo em {}",
        output_path.display()
    );
    Ok(())
}

fn run_report_command(args: ReportArgs) -> Result<()> {
    let correlated = json::read_correlated_trace(&args.input)?;
    let output_path = args.output.unwrap_or_else(|| {
        args.input
            .parent()
            .unwrap_or(Path::new("."))
            .join("index.html")
    });

    html::write_html_report(&output_path, &correlated)?;
    println!("[flowtrace] HTML salvo em {}", output_path.display());
    Ok(())
}

fn run_open_command(args: OpenArgs) -> Result<()> {
    let path = args.path.unwrap_or_else(|| {
        PathBuf::from("reports")
            .join(args.session)
            .join("index.html")
    });

    if !path.exists() {
        anyhow::bail!("arquivo não encontrado: {}", path.display());
    }

    let (cmd, cmd_args): (&str, Vec<String>) = if cfg!(target_os = "macos") {
        ("open", vec![path.to_string_lossy().to_string()])
    } else if cfg!(target_os = "windows") {
        (
            "cmd",
            vec![
                "/C".to_string(),
                "start".to_string(),
                path.to_string_lossy().to_string(),
            ],
        )
    } else {
        ("xdg-open", vec![path.to_string_lossy().to_string()])
    };

    match Command::new(cmd).args(cmd_args).status() {
        Ok(status) if status.success() => {
            println!("[flowtrace] Abrindo {}", path.display());
        }
        _ => {
            println!(
                "[flowtrace] Não foi possível abrir automaticamente. Caminho: {}",
                path.display()
            );
        }
    }

    Ok(())
}
