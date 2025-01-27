use crate::api::clock;
use crate::sys;

use alloc::{string::{String, ToString}, vec::Vec};
use time::{Duration, OffsetDateTime, UtcOffset};

pub fn now() -> OffsetDateTime {
    now_utc().to_offset(offset())
}

pub fn now_utc() -> OffsetDateTime {
    let s = clock::epoch_time(); // Since Unix Epoch
    let ns = Duration::nanoseconds(libm::floor(1e9 * (s - libm::floor(s))) as i64);
    OffsetDateTime::from_unix_timestamp(s as i64).expect("Invalid timestamp") + ns
}

pub fn from_timestamp(ts: i64) -> OffsetDateTime {
    from_timestamp_utc(ts).to_offset(offset())
}

pub fn from_timestamp_utc(ts: i64) -> OffsetDateTime {
    OffsetDateTime::from_unix_timestamp(ts).expect("Invalid timestamp")
}

fn offset() -> UtcOffset {
    if let Some(tz) = sys::process::env("TZ") {
        if let Ok(offset) = tz.parse::<i32>() {
            return match UtcOffset::from_whole_seconds(offset) {
                Ok(offset) => offset,
                Err(_) => UtcOffset::UTC,
            };
        }
    }
    UtcOffset::UTC
}

pub fn format_offset_time(time: OffsetDateTime) -> String {
    // time.format(...) is not available without std.
    // Manually format the time.
    let mut s = String::new();
    s.push_str(&time.year().to_string());
    s.push('-');
    s.push_str(&(time.month() as u8).to_string());
    s.push('-');
    s.push_str(&time.day().to_string());
    s.push(' ');
    s.push_str(&time.hour().to_string());
    s.push(':');
    s.push_str(&time.minute().to_string());
    s.push(':');
    s.push_str(&time.second().to_string());
    s
}

pub fn format_primitive_time(time: time::PrimitiveDateTime) -> String {
    // time.format(...) is not available without std.
    // Manually format the time.
    let mut s = String::new();
    s.push_str(&time.year().to_string());
    s.push('-');
    s.push_str(&(time.month() as u8).to_string());
    s.push('-');
    s.push_str(&time.day().to_string());
    s.push(' ');
    s.push_str(&time.hour().to_string());
    s.push(':');
    s.push_str(&time.minute().to_string());
    s.push(':');
    s.push_str(&time.second().to_string());
    s
}

pub fn parse_primitive_date_time(s: &str) -> Vec<u8> {
    let mut v = Vec::new();
    for part in s.split(|c| c == '-' || c == ' ' || c == ':') {
        if let Ok(n) = part.parse::<u8>() {
            v.push(n);
        }
    }
    v
}
