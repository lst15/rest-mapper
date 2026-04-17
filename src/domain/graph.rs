use serde::{Deserialize, Serialize};

use crate::domain::event::{RequestClassification, UserActionType};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TraceGraph {
    pub nodes: Vec<TraceNode>,
    pub edges: Vec<TraceEdge>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", content = "data")]
pub enum TraceNode {
    Route(RouteNode),
    Action(ActionNode),
    Request(RequestNode),
    BackendEndpoint(BackendEndpointNode),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteNode {
    pub id: String,
    pub route: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionNode {
    pub id: String,
    pub action_id: String,
    pub action_type: UserActionType,
    pub label: String,
    pub route: Option<String>,
    pub ts_unix_ms: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestNode {
    pub id: String,
    pub request_id: String,
    pub method: String,
    pub url: String,
    pub route: Option<String>,
    pub classification: RequestClassification,
    pub ts_unix_ms: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackendEndpointNode {
    pub id: String,
    pub endpoint: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceEdge {
    pub from: String,
    pub to: String,
    pub relation: EdgeRelation,
    pub confidence: f32,
    pub evidence: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EdgeRelation {
    OccurredOnRoute,
    Triggered,
    CorrelatesTo,
    ConsumesEndpoint,
    FollowedBy,
}
