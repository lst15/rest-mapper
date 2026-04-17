use crate::domain::event::{TraceEvent, TraceEventEnvelope};

pub fn route_for_event(event: &TraceEvent) -> Option<String> {
    match event {
        TraceEvent::RouteChanged(evt) => Some(evt.to_url.clone()),
        TraceEvent::PageNavigated(evt) => Some(evt.to_url.clone()),
        TraceEvent::UserAction(evt) => evt.route.clone().or_else(|| Some(evt.page_url.clone())),
        TraceEvent::NetworkRequest(evt) => evt.route.clone().or_else(|| Some(evt.page_url.clone())),
        TraceEvent::DomSnapshotMarker(evt) => evt.route.clone(),
        _ => None,
    }
}

pub fn sort_events(mut events: Vec<TraceEventEnvelope>) -> Vec<TraceEventEnvelope> {
    events.sort_by_key(|evt| evt.ts_unix_ms);
    events
}
