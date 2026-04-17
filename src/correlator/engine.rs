use std::collections::{HashMap, HashSet};

use crate::{
    correlator::{
        grouping::{self, ActionRecord},
        heuristics,
        scoring::{self, ScoreWeights},
    },
    domain::{
        correlation::{CorrelatedTrace, RequestCorrelation, SessionSummary, TimelineItem},
        event::{RequestClassification, TraceEvent, TraceEventEnvelope},
        graph::{
            ActionNode, BackendEndpointNode, EdgeRelation, RequestNode, RouteNode, TraceEdge,
            TraceGraph, TraceNode,
        },
        trace,
    },
};

#[derive(Debug, Clone)]
pub struct CorrelatorConfig {
    pub temporal_window_ms: i64,
    pub min_score: f32,
    pub score_weights: ScoreWeights,
}

impl Default for CorrelatorConfig {
    fn default() -> Self {
        Self {
            temporal_window_ms: 2_000,
            min_score: 0.35,
            score_weights: ScoreWeights::default(),
        }
    }
}

#[derive(Debug, Default)]
pub struct CorrelatorEngine {
    pub config: CorrelatorConfig,
}

impl CorrelatorEngine {
    pub fn correlate(&self, events: &[TraceEventEnvelope]) -> CorrelatedTrace {
        if events.is_empty() {
            return CorrelatedTrace {
                session_id: "unknown".to_string(),
                total_events: 0,
                summary: SessionSummary {
                    initial_url: None,
                    started_at_unix_ms: None,
                    ended_at_unix_ms: None,
                    duration_ms: 0,
                    navigation_count: 0,
                    action_count: 0,
                    request_count: 0,
                    endpoint_count: 0,
                },
                correlations: Vec::new(),
                route_flows: Vec::new(),
                timeline: Vec::new(),
                graph: TraceGraph::default(),
            };
        }

        let ordered = trace::sort_events(events.to_vec());
        let session_id = ordered
            .first()
            .map(|evt| evt.session_id.clone())
            .unwrap_or_else(|| "unknown".to_string());

        let mut initial_url = None;
        let started_at_unix_ms = ordered.first().map(|evt| evt.ts_unix_ms);
        let ended_at_unix_ms = ordered.last().map(|evt| evt.ts_unix_ms);

        let mut navigation_count = 0usize;
        let mut timeline = Vec::new();
        let mut actions: Vec<ActionRecord> = Vec::new();

        #[derive(Debug, Clone)]
        struct RequestRecord {
            ts_unix_ms: i64,
            event: crate::domain::event::NetworkRequestEvent,
        }

        let mut requests: Vec<RequestRecord> = Vec::new();

        for envelope in &ordered {
            match &envelope.event {
                TraceEvent::BrowserOpened(evt) => {
                    initial_url = Some(evt.url.clone());
                    timeline.push(TimelineItem {
                        ts_unix_ms: envelope.ts_unix_ms,
                        event_type: "BrowserOpened".to_string(),
                        route: Some(evt.url.clone()),
                        action_id: None,
                        request_id: None,
                        description: format!("Browser {} aberto", evt.browser),
                    });
                }
                TraceEvent::PageNavigated(evt) => {
                    navigation_count += 1;
                    timeline.push(TimelineItem {
                        ts_unix_ms: envelope.ts_unix_ms,
                        event_type: "PageNavigated".to_string(),
                        route: Some(evt.to_url.clone()),
                        action_id: None,
                        request_id: None,
                        description: format!("Navegação para {}", evt.to_url),
                    });
                }
                TraceEvent::RouteChanged(evt) => {
                    navigation_count += 1;
                    timeline.push(TimelineItem {
                        ts_unix_ms: envelope.ts_unix_ms,
                        event_type: "RouteChanged".to_string(),
                        route: Some(evt.to_url.clone()),
                        action_id: None,
                        request_id: None,
                        description: format!("Rota alterada para {}", evt.to_url),
                    });
                }
                TraceEvent::UserAction(evt) => {
                    actions.push(ActionRecord {
                        ts_unix_ms: envelope.ts_unix_ms,
                        event: evt.clone(),
                    });
                    timeline.push(TimelineItem {
                        ts_unix_ms: envelope.ts_unix_ms,
                        event_type: "UserAction".to_string(),
                        route: evt.route.clone().or_else(|| Some(evt.page_url.clone())),
                        action_id: Some(evt.action_id.clone()),
                        request_id: None,
                        description: format!("Ação {:?}", evt.action_type),
                    });
                }
                TraceEvent::NetworkRequest(evt) => {
                    let mut event = evt.clone();
                    if event.classification == RequestClassification::Unknown {
                        event.classification = heuristics::classify_request(&event);
                    }

                    requests.push(RequestRecord {
                        ts_unix_ms: envelope.ts_unix_ms,
                        event: event.clone(),
                    });
                    timeline.push(TimelineItem {
                        ts_unix_ms: envelope.ts_unix_ms,
                        event_type: "NetworkRequest".to_string(),
                        route: event.route.clone().or_else(|| Some(event.page_url.clone())),
                        action_id: None,
                        request_id: Some(event.request_id.clone()),
                        description: format!("{} {}", event.method, event.url),
                    });
                }
                _ => {
                    timeline.push(TimelineItem {
                        ts_unix_ms: envelope.ts_unix_ms,
                        event_type: format_event_type(&envelope.event),
                        route: trace::route_for_event(&envelope.event),
                        action_id: None,
                        request_id: None,
                        description: format_event_type(&envelope.event),
                    });
                }
            }
        }

        let mut correlations = Vec::<RequestCorrelation>::new();

        for request in &requests {
            let endpoint = heuristics::canonical_endpoint(&request.event.url);
            let mut best: Option<(usize, f32, Vec<String>)> = None;

            for (action_idx, action) in actions.iter().enumerate() {
                if action.ts_unix_ms > request.ts_unix_ms {
                    continue;
                }

                let delta = request.ts_unix_ms - action.ts_unix_ms;
                if delta > self.config.temporal_window_ms {
                    continue;
                }

                let score_details = scoring::score_action_request(
                    &action.event,
                    action.ts_unix_ms,
                    &request.event,
                    request.ts_unix_ms,
                    self.config.temporal_window_ms,
                    &self.config.score_weights,
                );

                match &best {
                    Some((_, best_score, _)) if *best_score >= score_details.score => {}
                    _ => {
                        best = Some((action_idx, score_details.score, score_details.evidence));
                    }
                }
            }

            let mut correlation = RequestCorrelation {
                request_id: request.event.request_id.clone(),
                method: request.event.method.clone(),
                request_url: request.event.url.clone(),
                route: request
                    .event
                    .route
                    .clone()
                    .or_else(|| Some(request.event.page_url.clone())),
                endpoint,
                request_ts_unix_ms: request.ts_unix_ms,
                classification: request.event.classification,
                action_id: None,
                action_type: None,
                confidence: 0.0,
                evidence: vec!["sem associação forte com ação explícita".to_string()],
            };

            if let Some((action_idx, base_score, mut evidence)) = best
                && base_score >= self.config.min_score
            {
                let action = &actions[action_idx];
                correlation.action_id = Some(action.event.action_id.clone());
                correlation.action_type = Some(action.event.action_type.clone());
                correlation.confidence = base_score;
                evidence.push(format!("score base {:.2}", base_score));
                correlation.evidence = evidence;
            }

            correlations.push(correlation);
        }

        let mut burst_count_by_action: HashMap<String, usize> = HashMap::new();
        for correlation in &correlations {
            if let Some(action_id) = &correlation.action_id {
                *burst_count_by_action.entry(action_id.clone()).or_insert(0) += 1;
            }
        }

        for correlation in &mut correlations {
            if let Some(action_id) = &correlation.action_id {
                let burst_size = *burst_count_by_action.get(action_id).unwrap_or(&1);
                let boosted = scoring::apply_burst_boost(
                    correlation.confidence,
                    burst_size,
                    &self.config.score_weights,
                );
                if boosted > correlation.confidence {
                    correlation
                        .evidence
                        .push(format!("burst de {} requests para mesma ação", burst_size));
                }
                correlation.confidence = boosted;
            }
        }

        let route_flows = grouping::build_route_flows(&actions, &correlations);
        let graph = build_graph(&actions, &correlations);

        let endpoint_count = correlations
            .iter()
            .map(|corr| corr.endpoint.clone())
            .collect::<HashSet<_>>()
            .len();

        let duration_ms = match (started_at_unix_ms, ended_at_unix_ms) {
            (Some(start), Some(end)) => end.saturating_sub(start),
            _ => 0,
        };

        CorrelatedTrace {
            session_id,
            total_events: ordered.len(),
            summary: SessionSummary {
                initial_url,
                started_at_unix_ms,
                ended_at_unix_ms,
                duration_ms,
                navigation_count,
                action_count: actions.len(),
                request_count: requests.len(),
                endpoint_count,
            },
            correlations,
            route_flows,
            timeline,
            graph,
        }
    }
}

fn build_graph(actions: &[ActionRecord], correlations: &[RequestCorrelation]) -> TraceGraph {
    let mut graph = TraceGraph::default();
    let mut seen_nodes = HashSet::new();

    let mut action_by_id: HashMap<String, &ActionRecord> = HashMap::new();
    for action in actions {
        action_by_id.insert(action.event.action_id.clone(), action);
    }

    for action in actions {
        let action_node_id = format!("action:{}", action.event.action_id);
        add_node(
            &mut graph,
            &mut seen_nodes,
            action_node_id.clone(),
            TraceNode::Action(ActionNode {
                id: action_node_id.clone(),
                action_id: action.event.action_id.clone(),
                action_type: action.event.action_type.clone(),
                label: action
                    .event
                    .target
                    .text
                    .clone()
                    .filter(|text| !text.trim().is_empty())
                    .unwrap_or_else(|| format!("{:?}", action.event.action_type)),
                route: action.event.route.clone(),
                ts_unix_ms: action.ts_unix_ms,
            }),
        );

        if let Some(route) = action
            .event
            .route
            .clone()
            .or_else(|| Some(action.event.page_url.clone()))
        {
            let route_node_id = format!("route:{route}");
            add_node(
                &mut graph,
                &mut seen_nodes,
                route_node_id.clone(),
                TraceNode::Route(RouteNode {
                    id: route_node_id.clone(),
                    route,
                }),
            );

            graph.edges.push(TraceEdge {
                from: route_node_id,
                to: action_node_id,
                relation: EdgeRelation::OccurredOnRoute,
                confidence: 1.0,
                evidence: vec!["ação observada nesta rota".to_string()],
            });
        }
    }

    for correlation in correlations {
        let request_node_id = format!("request:{}", correlation.request_id);
        add_node(
            &mut graph,
            &mut seen_nodes,
            request_node_id.clone(),
            TraceNode::Request(RequestNode {
                id: request_node_id.clone(),
                request_id: correlation.request_id.clone(),
                method: correlation.method.clone(),
                url: correlation.request_url.clone(),
                route: correlation.route.clone(),
                classification: correlation.classification,
                ts_unix_ms: correlation.request_ts_unix_ms,
            }),
        );

        let endpoint_node_id = format!("endpoint:{}", correlation.endpoint);
        add_node(
            &mut graph,
            &mut seen_nodes,
            endpoint_node_id.clone(),
            TraceNode::BackendEndpoint(BackendEndpointNode {
                id: endpoint_node_id.clone(),
                endpoint: correlation.endpoint.clone(),
            }),
        );

        graph.edges.push(TraceEdge {
            from: request_node_id.clone(),
            to: endpoint_node_id,
            relation: EdgeRelation::ConsumesEndpoint,
            confidence: 1.0,
            evidence: vec!["request consumiu endpoint backend".to_string()],
        });

        if let Some(route) = &correlation.route {
            let route_node_id = format!("route:{route}");
            add_node(
                &mut graph,
                &mut seen_nodes,
                route_node_id.clone(),
                TraceNode::Route(RouteNode {
                    id: route_node_id.clone(),
                    route: route.clone(),
                }),
            );

            graph.edges.push(TraceEdge {
                from: route_node_id,
                to: request_node_id.clone(),
                relation: EdgeRelation::CorrelatesTo,
                confidence: 0.6,
                evidence: vec!["request observada nesta rota".to_string()],
            });
        }

        if let Some(action_id) = &correlation.action_id {
            let action_node_id = format!("action:{action_id}");
            graph.edges.push(TraceEdge {
                from: action_node_id,
                to: request_node_id,
                relation: EdgeRelation::Triggered,
                confidence: correlation.confidence,
                evidence: correlation.evidence.clone(),
            });
        }
    }

    let mut ordered_actions = actions.iter().collect::<Vec<_>>();
    ordered_actions.sort_by_key(|action| action.ts_unix_ms);

    for pair in ordered_actions.windows(2) {
        if let [from, to] = pair {
            graph.edges.push(TraceEdge {
                from: format!("action:{}", from.event.action_id),
                to: format!("action:{}", to.event.action_id),
                relation: EdgeRelation::FollowedBy,
                confidence: 0.8,
                evidence: vec!["sequência temporal de ações".to_string()],
            });
        }
    }

    graph
}

fn add_node(graph: &mut TraceGraph, seen: &mut HashSet<String>, id: String, node: TraceNode) {
    if seen.insert(id) {
        graph.nodes.push(node);
    }
}

fn format_event_type(event: &TraceEvent) -> String {
    match event {
        TraceEvent::BrowserOpened(_) => "BrowserOpened",
        TraceEvent::PageNavigated(_) => "PageNavigated",
        TraceEvent::RouteChanged(_) => "RouteChanged",
        TraceEvent::UserAction(_) => "UserAction",
        TraceEvent::NetworkRequest(_) => "NetworkRequest",
        TraceEvent::NetworkResponse(_) => "NetworkResponse",
        TraceEvent::ConsoleLog(_) => "ConsoleLog",
        TraceEvent::DomSnapshotMarker(_) => "DomSnapshotMarker",
        TraceEvent::BrowserClosed(_) => "BrowserClosed",
    }
    .to_string()
}
