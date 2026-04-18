use anyhow::Result;
use std::time::Duration;
use uuid::Uuid;

use crate::{
    app::shutdown,
    cli::commands::RecordArgs,
    collector::playwright_process::{CollectorConfig, CollectorProcess},
    correlator::engine::CorrelatorEngine,
    report::{html, json},
    storage::{raw_trace_store, session_store},
};

pub async fn run_record_session(args: RecordArgs) -> Result<()> {
    let session_id = Uuid::new_v4().to_string();
    let session_paths = session_store::create_session_layout(&args.reports_dir, &session_id)?;

    println!("[flowtrace] Sessão iniciada: {}", session_paths.session_id);
    println!("[flowtrace] Abrindo navegador em {}", args.url);

    let config = CollectorConfig {
        session_id: session_paths.session_id.clone(),
        url: args.url.clone(),
        output_path: session_paths.raw_trace_path.clone(),
        script_path: args.collector_script.clone(),
        node_bin: args.node_bin.clone(),
        browser: args.browser.clone(),
        headless: args.headless,
        event_types: args
            .event_types
            .iter()
            .map(|event_type| event_type.as_event_name().to_string())
            .collect(),
    };

    let mut collector = CollectorProcess::spawn(config).await?;

    println!("[flowtrace] Navegue livremente. Pressione Enter aqui para encerrar.");

    let enter_task = tokio::spawn(shutdown::wait_for_enter());
    let exit_status = loop {
        if enter_task.is_finished() {
            let _ = enter_task.await?;
            println!("[flowtrace] Encerrando captura...");
            collector.request_shutdown().await?;
            break collector.wait().await?;
        }

        if let Some(status) = collector.try_wait()? {
            println!(
                "[flowtrace] Collector finalizado (browser possivelmente fechado manualmente)."
            );
            enter_task.abort();
            break status;
        }

        tokio::time::sleep(Duration::from_millis(250)).await;
    };

    collector
        .finalize(exit_status, Duration::from_secs(args.shutdown_timeout_secs))
        .await?;

    println!("[flowtrace] Correlacionando eventos...");
    let events = raw_trace_store::load_jsonl(&session_paths.raw_trace_path)?;
    let correlated = CorrelatorEngine::default().correlate(&events);

    json::write_correlated_trace(&session_paths.correlated_trace_path, &correlated)?;
    html::write_html_report(&session_paths.index_html_path, &correlated)?;

    println!(
        "[flowtrace] Relatório gerado em {} (sessão em {})",
        session_paths.index_html_path.display(),
        session_paths.session_dir.display()
    );

    Ok(())
}
