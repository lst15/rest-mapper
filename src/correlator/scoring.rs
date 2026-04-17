use crate::{
    domain::event::{NetworkRequestEvent, UserActionEvent},
    utils::similarity,
};

#[derive(Debug, Clone)]
pub struct ScoreWeights {
    pub explicit_context: f32,
    pub temporal_proximity: f32,
    pub same_route: f32,
    pub semantic_similarity: f32,
    pub same_burst: f32,
}

impl Default for ScoreWeights {
    fn default() -> Self {
        Self {
            explicit_context: 0.45,
            temporal_proximity: 0.20,
            same_route: 0.15,
            semantic_similarity: 0.15,
            same_burst: 0.05,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ScoreDetails {
    pub score: f32,
    pub evidence: Vec<String>,
}

pub fn score_action_request(
    action: &UserActionEvent,
    action_ts_unix_ms: i64,
    request: &NetworkRequestEvent,
    request_ts_unix_ms: i64,
    temporal_window_ms: i64,
    weights: &ScoreWeights,
) -> ScoreDetails {
    let delta_ms = (request_ts_unix_ms - action_ts_unix_ms).max(0);
    let temporal_score = if delta_ms > temporal_window_ms {
        0.0
    } else {
        1.0 - (delta_ms as f32 / temporal_window_ms as f32)
    };

    let action_route = action.route.as_deref().or(Some(action.page_url.as_str()));
    let request_route = request.route.as_deref().or(Some(request.page_url.as_str()));

    let same_route_score = if action_route.is_some() && action_route == request_route {
        1.0
    } else {
        0.0
    };

    let explicit_score = if request
        .initiator_hint
        .as_ref()
        .and_then(|hint| hint.related_action_id.as_deref())
        == Some(action.action_id.as_str())
    {
        1.0
    } else {
        0.0
    };

    let semantic_score = action
        .route
        .as_deref()
        .map(|route| similarity::similarity_between_route_and_endpoint(route, &request.url))
        .unwrap_or(0.0);

    let mut evidence = Vec::new();

    if explicit_score > 0.0 {
        evidence.push("related_action_id explícito no initiator_hint".to_string());
    }

    evidence.push(format!("proximidade temporal de {}ms", delta_ms));

    if same_route_score > 0.0 {
        evidence.push("ação e request na mesma rota".to_string());
    }

    if semantic_score > 0.0 {
        evidence.push(format!("similaridade semântica {:.2}", semantic_score));
    }

    let score = (weights.explicit_context * explicit_score
        + weights.temporal_proximity * temporal_score
        + weights.same_route * same_route_score
        + weights.semantic_similarity * semantic_score)
        .clamp(0.0, 1.0);

    ScoreDetails { score, evidence }
}

pub fn apply_burst_boost(score: f32, burst_size: usize, weights: &ScoreWeights) -> f32 {
    let burst_factor = (burst_size as f32 / 4.0).clamp(0.0, 1.0);
    (score + weights.same_burst * burst_factor).clamp(0.0, 1.0)
}
