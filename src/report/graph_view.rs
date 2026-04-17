use crate::domain::graph::{TraceEdge, TraceNode};

pub fn render_nodes(nodes: &[TraceNode]) -> String {
    let mut html = String::new();
    for node in nodes {
        let label = match node {
            TraceNode::Route(route) => format!("Route: {}", route.route),
            TraceNode::Action(action) => {
                format!("Action: {:?} ({})", action.action_type, action.action_id)
            }
            TraceNode::Request(request) => {
                format!("Request: {} {}", request.method, request.url)
            }
            TraceNode::BackendEndpoint(endpoint) => format!("Endpoint: {}", endpoint.endpoint),
        };
        html.push_str(&format!("<li>{}</li>", escape_html(&label)));
    }
    html
}

pub fn render_edges(edges: &[TraceEdge]) -> String {
    let mut html = String::new();
    for edge in edges {
        let line = format!(
            "{} -> {} ({:?}, conf {:.2})",
            edge.from, edge.to, edge.relation, edge.confidence
        );
        html.push_str(&format!("<li>{}</li>", escape_html(&line)));
    }
    html
}

fn escape_html(input: &str) -> String {
    input
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}
