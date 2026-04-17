use anyhow::Result;
use std::{fs, path::Path};

use crate::domain::correlation::CorrelatedTrace;

pub fn write_correlated_trace(path: &Path, correlated: &CorrelatedTrace) -> Result<()> {
    let payload = serde_json::to_string_pretty(correlated)?;
    fs::write(path, payload)?;
    Ok(())
}

pub fn read_correlated_trace(path: &Path) -> Result<CorrelatedTrace> {
    let payload = fs::read_to_string(path)?;
    let parsed = serde_json::from_str(&payload)?;
    Ok(parsed)
}
