use chrono::{DateTime, Utc};

pub fn now_unix_ms() -> i64 {
    Utc::now().timestamp_millis()
}

pub fn format_unix_ms(ts_unix_ms: i64) -> String {
    DateTime::from_timestamp_millis(ts_unix_ms)
        .map(|dt| dt.format("%Y-%m-%d %H:%M:%S%.3f UTC").to_string())
        .unwrap_or_else(|| ts_unix_ms.to_string())
}
