use anyhow::{Context, Result};
use std::{
    fs::File,
    io::{BufRead, BufReader, Write},
    path::Path,
};

use crate::domain::event::TraceEventEnvelope;
use crate::error::FlowtraceError;

pub fn load_jsonl(path: &Path) -> Result<Vec<TraceEventEnvelope>> {
    let file = File::open(path).with_context(|| format!("falha ao abrir {}", path.display()))?;
    let reader = BufReader::new(file);

    let mut events = Vec::new();

    for (idx, line_result) in reader.lines().enumerate() {
        let line = line_result?;
        let trimmed = line.trim();

        if trimmed.is_empty() {
            continue;
        }

        let parsed = serde_json::from_str::<TraceEventEnvelope>(trimmed).map_err(|source| {
            FlowtraceError::EventParse {
                line: idx + 1,
                source,
            }
        })?;

        events.push(parsed);
    }

    Ok(events)
}

#[allow(dead_code)]
pub fn write_jsonl(path: &Path, events: &[TraceEventEnvelope]) -> Result<()> {
    let mut file = File::create(path)?;
    for event in events {
        let line = serde_json::to_string(event)?;
        writeln!(file, "{line}")?;
    }
    Ok(())
}
