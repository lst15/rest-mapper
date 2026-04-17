use anyhow::Result;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct SessionPaths {
    pub session_id: String,
    pub session_dir: PathBuf,
    pub raw_trace_path: PathBuf,
    pub correlated_trace_path: PathBuf,
    pub index_html_path: PathBuf,
}

pub fn create_session_layout(reports_dir: &Path, session_id: &str) -> Result<SessionPaths> {
    let session_dir = reports_dir.join(session_id);
    std::fs::create_dir_all(&session_dir)?;

    Ok(SessionPaths {
        session_id: session_id.to_string(),
        raw_trace_path: session_dir.join("raw_trace.jsonl"),
        correlated_trace_path: session_dir.join("correlated_trace.json"),
        index_html_path: session_dir.join("index.html"),
        session_dir,
    })
}
