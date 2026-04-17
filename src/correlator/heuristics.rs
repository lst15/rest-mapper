use crate::domain::event::{NetworkRequestEvent, RequestClassification};
use url::Url;

pub fn classify_request(request: &NetworkRequestEvent) -> RequestClassification {
    let resource = request
        .resource_type
        .as_ref()
        .map(|v| v.to_ascii_lowercase())
        .unwrap_or_default();
    let url = request.url.to_ascii_lowercase();

    if is_analytics(&url) || resource == "beacon" {
        return RequestClassification::Analytics;
    }

    if ["image", "font", "stylesheet", "media", "other"].contains(&resource.as_str())
        || url.contains("favicon")
    {
        return RequestClassification::StaticAsset;
    }

    if resource == "document" {
        return RequestClassification::NavigationDocument;
    }

    if resource == "fetch" || resource == "xhr" || url.contains("/api/") || url.contains("graphql")
    {
        return RequestClassification::AppData;
    }

    RequestClassification::Unknown
}

pub fn canonical_endpoint(url: &str) -> String {
    if let Ok(parsed) = Url::parse(url) {
        let mut endpoint = parsed.path().to_string();
        if endpoint.is_empty() {
            endpoint = "/".to_string();
        }
        return endpoint;
    }

    url.to_string()
}

fn is_analytics(url: &str) -> bool {
    [
        "google-analytics",
        "googletagmanager",
        "mixpanel",
        "segment",
        "analytics",
        "amplitude",
        "sentry",
        "datadog",
    ]
    .iter()
    .any(|needle| url.contains(needle))
}
