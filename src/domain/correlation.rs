use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::domain::{
    event::{NetworkRequestEvent, NetworkResponseEvent, RequestClassification, UserActionType},
    graph::TraceGraph,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorrelatedTrace {
    pub session_id: String,
    pub total_events: usize,
    pub summary: SessionSummary,
    pub correlations: Vec<RequestCorrelation>,
    pub route_flows: Vec<RouteFlow>,
    pub timeline: Vec<TimelineItem>,
    pub graph: TraceGraph,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionSummary {
    pub initial_url: Option<String>,
    pub started_at_unix_ms: Option<i64>,
    pub ended_at_unix_ms: Option<i64>,
    pub duration_ms: i64,
    pub navigation_count: usize,
    pub action_count: usize,
    pub request_count: usize,
    pub endpoint_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestCorrelation {
    pub request_id: String,
    pub method: String,
    pub request_url: String,
    #[serde(default)]
    pub request_event: Option<NetworkRequestEvent>,
    #[serde(default)]
    pub request_payload: Option<String>,
    #[serde(default)]
    pub request_payload_encoding: Option<String>,
    #[serde(default)]
    pub request_payload_size_bytes: Option<usize>,
    #[serde(default)]
    pub request_payload_truncated: bool,
    pub route: Option<String>,
    pub endpoint: String,
    pub request_ts_unix_ms: i64,
    #[serde(default)]
    pub response_event: Option<NetworkResponseEvent>,
    #[serde(default)]
    pub response_status: Option<u16>,
    #[serde(default)]
    pub response_url: Option<String>,
    #[serde(default)]
    pub response_body: Option<String>,
    #[serde(default)]
    pub response_body_encoding: Option<String>,
    #[serde(default)]
    pub response_body_size_bytes: Option<usize>,
    #[serde(default)]
    pub response_body_truncated: bool,
    #[serde(default)]
    pub response_body_capture_error: Option<String>,
    pub classification: RequestClassification,
    pub action_id: Option<String>,
    pub action_type: Option<UserActionType>,
    pub confidence: f32,
    pub evidence: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteFlow {
    pub route: String,
    pub actions: Vec<ActionFlow>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionFlow {
    pub action_id: String,
    pub action_type: UserActionType,
    pub action_label: String,
    pub request_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimelineItem {
    pub ts_unix_ms: i64,
    pub event_type: String,
    pub route: Option<String>,
    pub action_id: Option<String>,
    pub request_id: Option<String>,
    pub description: String,
    #[serde(default)]
    pub raw_event: Option<Value>,
}
