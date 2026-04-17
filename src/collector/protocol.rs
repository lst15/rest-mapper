use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
pub enum CollectorStdoutMessage {
    #[serde(rename = "status")]
    Status {
        message: String,
        #[serde(default)]
        detail: Option<serde_json::Value>,
    },
}

pub fn parse_status_line(line: &str) -> Option<CollectorStdoutMessage> {
    serde_json::from_str::<CollectorStdoutMessage>(line).ok()
}
