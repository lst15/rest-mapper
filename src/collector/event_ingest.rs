use anyhow::Result;
use std::path::Path;

use crate::{domain::event::TraceEventEnvelope, storage::raw_trace_store};

pub fn ingest_events(path: &Path) -> Result<Vec<TraceEventEnvelope>> {
    raw_trace_store::load_jsonl(path)
}
