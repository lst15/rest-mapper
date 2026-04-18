use anyhow::Result;
use std::{fs, path::Path};

use crate::{
    domain::correlation::CorrelatedTrace,
    report::{graph_view, timeline},
    utils::time,
};

pub fn write_html_report(path: &Path, correlated: &CorrelatedTrace) -> Result<()> {
    let timeline_rows = timeline::render_timeline_rows(&correlated.timeline);
    let graph_nodes = graph_view::render_nodes(&correlated.graph.nodes);
    let graph_edges = graph_view::render_edges(&correlated.graph.edges);

    let mut flows_html = String::new();
    for route_flow in &correlated.route_flows {
        flows_html.push_str(&format!("<h4>{}</h4>", escape_html(&route_flow.route)));
        flows_html.push_str("<ul>");
        for action in &route_flow.actions {
            flows_html.push_str(&format!(
                "<li><strong>{:?}</strong> {} -> {}</li>",
                action.action_type,
                escape_html(&action.action_label),
                escape_html(&action.request_ids.join(", "))
            ));
        }
        flows_html.push_str("</ul>");
    }

    let html = format!(
        r#"<!doctype html>
<html lang="pt-BR">
<head>
  <meta charset="utf-8" />
  <meta name="viewport" content="width=device-width, initial-scale=1" />
  <title>Flowtrace Report {session}</title>
  <style>
    body {{ font-family: 'Segoe UI', Tahoma, sans-serif; margin: 24px; background: #f6f8fb; color: #1f2937; }}
    .grid {{ display: grid; grid-template-columns: repeat(auto-fit, minmax(220px, 1fr)); gap: 12px; }}
    .card {{ background: #fff; border-radius: 10px; padding: 12px 14px; box-shadow: 0 2px 8px rgba(0,0,0,0.06); }}
    table {{ width: 100%; border-collapse: collapse; background: #fff; border-radius: 10px; overflow: hidden; }}
    th, td {{ border-bottom: 1px solid #e5e7eb; text-align: left; padding: 8px; font-size: 13px; }}
    th {{ background: #eef2ff; }}
    td code {{ display: inline-block; max-width: 520px; white-space: pre-wrap; word-break: break-word; }}
    ul {{ padding-left: 20px; }}
    .section {{ margin-top: 24px; }}
    .muted {{ color: #6b7280; }}
    code {{ background: #eef2ff; padding: 2px 4px; border-radius: 4px; }}
  </style>
</head>
<body>
  <h1>Flowtrace Session <code>{session}</code></h1>
  <p class="muted">Gerado em {generated_at}</p>

  <div class="section">
    <h2>Resumo</h2>
    <div class="grid">
      <div class="card"><strong>URL inicial</strong><br />{initial_url}</div>
      <div class="card"><strong>Duração</strong><br />{duration_ms} ms</div>
      <div class="card"><strong>Navegações</strong><br />{navigation_count}</div>
      <div class="card"><strong>Ações</strong><br />{action_count}</div>
      <div class="card"><strong>Requests</strong><br />{request_count}</div>
      <div class="card"><strong>Endpoints</strong><br />{endpoint_count}</div>
    </div>
  </div>

  <div class="section">
    <h2>Timeline</h2>
    <table>
      <thead>
        <tr>
          <th>Timestamp</th><th>Evento</th><th>Rota</th><th>Ação</th><th>Request</th><th>Descrição</th><th>Modelo raw_trace</th>
        </tr>
      </thead>
      <tbody>{timeline_rows}</tbody>
    </table>
  </div>

  <div class="section">
    <h2>Flows por Rota</h2>
    {flows_html}
  </div>

  <div class="section">
    <h2>Grafo (Lista)</h2>
    <h3>Nós</h3>
    <ul>{graph_nodes}</ul>
    <h3>Arestas</h3>
    <ul>{graph_edges}</ul>
  </div>
</body>
</html>
"#,
        session = correlated.session_id,
        generated_at = time::format_unix_ms(time::now_unix_ms()),
        initial_url = escape_html(
            correlated
                .summary
                .initial_url
                .as_deref()
                .unwrap_or("desconhecida")
        ),
        duration_ms = correlated.summary.duration_ms,
        navigation_count = correlated.summary.navigation_count,
        action_count = correlated.summary.action_count,
        request_count = correlated.summary.request_count,
        endpoint_count = correlated.summary.endpoint_count,
        timeline_rows = timeline_rows,
        flows_html = flows_html,
        graph_nodes = graph_nodes,
        graph_edges = graph_edges,
    );

    fs::write(path, html)?;
    Ok(())
}

fn escape_html(input: &str) -> String {
    input
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}
