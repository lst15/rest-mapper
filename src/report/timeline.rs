use crate::domain::correlation::TimelineItem;
use crate::utils::time;

pub fn render_timeline_rows(items: &[TimelineItem]) -> String {
    let mut rows = String::new();

    for item in items {
        rows.push_str("<tr>");
        rows.push_str(&format!(
            "<td>{}</td>",
            time::format_unix_ms(item.ts_unix_ms)
        ));
        rows.push_str(&format!("<td>{}</td>", escape_html(&item.event_type)));
        rows.push_str(&format!(
            "<td>{}</td>",
            escape_html(item.route.as_deref().unwrap_or("-"))
        ));
        rows.push_str(&format!(
            "<td>{}</td>",
            escape_html(item.action_id.as_deref().unwrap_or("-"))
        ));
        rows.push_str(&format!(
            "<td>{}</td>",
            escape_html(item.request_id.as_deref().unwrap_or("-"))
        ));
        rows.push_str(&format!("<td>{}</td>", escape_html(&item.description)));
        rows.push_str("</tr>");
    }

    rows
}

fn escape_html(input: &str) -> String {
    input
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}
