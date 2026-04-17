use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceEventEnvelope {
    pub id: String,
    pub ts_unix_ms: i64,
    pub session_id: String,
    pub event: TraceEvent,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum TraceEvent {
    BrowserOpened(BrowserOpenedEvent),
    PageNavigated(PageNavigatedEvent),
    RouteChanged(RouteChangedEvent),
    UserAction(UserActionEvent),
    NetworkRequest(NetworkRequestEvent),
    NetworkResponse(NetworkResponseEvent),
    ConsoleLog(ConsoleLogEvent),
    DomSnapshotMarker(DomSnapshotMarkerEvent),
    BrowserClosed(BrowserClosedEvent),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrowserOpenedEvent {
    pub browser: String,
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageNavigatedEvent {
    pub from_url: Option<String>,
    pub to_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteChangedEvent {
    pub from_url: Option<String>,
    pub to_url: String,
    pub navigation_type: NavigationType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserActionEvent {
    pub action_id: String,
    pub action_type: UserActionType,
    pub page_url: String,
    pub route: Option<String>,
    pub target: ActionTarget,
    pub metadata: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UserActionType {
    Click,
    Submit,
    Input,
    Change,
    KeyPress,
    Navigation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionTarget {
    pub tag_name: Option<String>,
    pub id: Option<String>,
    pub classes: Vec<String>,
    pub text: Option<String>,
    pub test_id: Option<String>,
    pub name: Option<String>,
    pub role: Option<String>,
    pub css_selector: Option<String>,
    pub xpath: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkRequestEvent {
    pub request_id: String,
    pub page_url: String,
    pub route: Option<String>,
    pub method: String,
    pub url: String,
    pub resource_type: Option<String>,
    pub headers: Vec<(String, String)>,
    pub post_data: Option<String>,
    pub initiator_hint: Option<InitiatorHint>,
    #[serde(default)]
    pub classification: RequestClassification,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkResponseEvent {
    pub request_id: String,
    pub status: u16,
    pub url: String,
    pub headers: Vec<(String, String)>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsoleLogEvent {
    pub level: String,
    pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomSnapshotMarkerEvent {
    pub marker: String,
    pub route: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrowserClosedEvent {
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InitiatorHint {
    pub source_type: InitiatorSourceType,
    pub related_action_id: Option<String>,
    pub js_stack: Vec<String>,
    pub trigger_ts_unix_ms: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InitiatorSourceType {
    Fetch,
    Xhr,
    Document,
    Script,
    Router,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NavigationType {
    InitialLoad,
    PushState,
    ReplaceState,
    PopState,
    FullNavigation,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum RequestClassification {
    AppData,
    NavigationDocument,
    StaticAsset,
    Analytics,
    #[default]
    Unknown,
}
