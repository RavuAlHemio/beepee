use chrono::{DateTime, Timelike, TimeZone};

pub fn milliseconds_since_midnight<T: Timelike>(t: &T) -> u32 {
    t.hour() * 60 * 60 * 1000
        + t.minute() * 60 * 1000
        + t.second() * 1000
}

pub fn milliseconds_since_epoch<Z: TimeZone>(t: &DateTime<Z>) -> i64 {
    t.timestamp() * 1000
}
