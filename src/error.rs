use thiserror::Error;

#[derive(Debug, Error)]
pub enum FlowtraceError {
    #[error("event parse failure at line {line}: {source}")]
    EventParse {
        line: usize,
        source: serde_json::Error,
    },
}
