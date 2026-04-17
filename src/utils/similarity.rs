use std::collections::HashSet;

use url::Url;

const STOPWORDS: &[&str] = &["api", "v1", "v2", "internal", "public", "rest", "graphql"];

pub fn similarity_between_route_and_endpoint(route: &str, endpoint: &str) -> f32 {
    let route_tokens = tokenize_path(route);
    let endpoint_tokens = tokenize_path(endpoint);

    if route_tokens.is_empty() || endpoint_tokens.is_empty() {
        return 0.0;
    }

    let route_set: HashSet<_> = route_tokens.into_iter().collect();
    let endpoint_set: HashSet<_> = endpoint_tokens.into_iter().collect();

    let intersection = route_set.intersection(&endpoint_set).count() as f32;
    let union = route_set.union(&endpoint_set).count() as f32;

    if union == 0.0 {
        0.0
    } else {
        (intersection / union).clamp(0.0, 1.0)
    }
}

pub fn tokenize_path(input: &str) -> Vec<String> {
    let path = normalize_path(input);

    path.split(|c: char| c == '/' || c == '-' || c == '_' || c == '.' || c == '?' || c == '&')
        .map(str::trim)
        .filter(|segment| !segment.is_empty())
        .map(|segment| segment.to_ascii_lowercase())
        .filter(|segment| !segment.chars().all(|c| c.is_ascii_digit()))
        .map(singularize_simple)
        .filter(|segment| !STOPWORDS.contains(&segment.as_str()))
        .collect()
}

fn normalize_path(input: &str) -> String {
    if let Ok(url) = Url::parse(input) {
        let mut full = url.path().to_string();
        if let Some(q) = url.query() {
            full.push('?');
            full.push_str(q);
        }
        return full;
    }

    input.to_string()
}

fn singularize_simple(token: String) -> String {
    if token.ends_with("ies") && token.len() > 3 {
        format!("{}y", &token[..token.len() - 3])
    } else if token.ends_with('s') && token.len() > 3 {
        token[..token.len() - 1].to_string()
    } else {
        token
    }
}
