use chrono::{DateTime, Datelike, Duration, TimeZone, Utc};

pub const SESSION_WINDOW_HOURS: i64 = 5;
pub const IDLE_AFTER_SECONDS: i64 = 90;

pub fn session_window() -> Duration {
    Duration::hours(SESSION_WINDOW_HOURS)
}

pub fn monday_start(now: DateTime<Utc>) -> DateTime<Utc> {
    let weekday = now.weekday().num_days_from_monday() as i64;
    let date = now.date_naive() - Duration::days(weekday);
    Utc.from_utc_datetime(&date.and_hms_opt(0, 0, 0).expect("valid midnight"))
}

pub fn weekly_reset_after(now: DateTime<Utc>) -> DateTime<Utc> {
    monday_start(now) + Duration::days(7)
}
