use std::collections::HashMap;

use crate::domain::{
    correlation::{ActionFlow, RequestCorrelation, RouteFlow},
    event::UserActionEvent,
};

#[derive(Debug, Clone)]
pub struct ActionRecord {
    pub ts_unix_ms: i64,
    pub event: UserActionEvent,
}

pub fn build_route_flows(
    actions: &[ActionRecord],
    correlations: &[RequestCorrelation],
) -> Vec<RouteFlow> {
    let mut by_route: HashMap<String, Vec<ActionFlow>> = HashMap::new();

    for action in actions {
        let route = action
            .event
            .route
            .clone()
            .unwrap_or_else(|| action.event.page_url.clone());

        let request_ids = correlations
            .iter()
            .filter(|corr| corr.action_id.as_deref() == Some(action.event.action_id.as_str()))
            .map(|corr| corr.request_id.clone())
            .collect::<Vec<_>>();

        let label = action
            .event
            .target
            .text
            .clone()
            .filter(|text| !text.trim().is_empty())
            .unwrap_or_else(|| format!("{:?}", action.event.action_type));

        by_route.entry(route).or_default().push(ActionFlow {
            action_id: action.event.action_id.clone(),
            action_type: action.event.action_type.clone(),
            action_label: label,
            request_ids,
        });
    }

    let mut routes = by_route
        .into_iter()
        .map(|(route, mut actions)| {
            actions.sort_by(|a, b| a.action_id.cmp(&b.action_id));
            RouteFlow { route, actions }
        })
        .collect::<Vec<_>>();

    routes.sort_by(|a, b| a.route.cmp(&b.route));
    routes
}
