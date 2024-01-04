use chrono::{DateTime, Timelike, Utc};

pub mod blockslib;
pub mod currency;
pub mod kraken;

fn unix_to_str(unix_time: i32) -> String {
    chrono::prelude::NaiveDateTime::from_timestamp_opt(unix_time as i64, 0)
        .unwrap()
        .format("%Y-%m-%d %H:%M")
        .to_string()
}

pub fn get_midnight(dt: DateTime<Utc>) -> DateTime<Utc> {
    dt.with_hour(0)
        .unwrap()
        .with_minute(0)
        .unwrap()
        .with_second(0)
        .unwrap()
}
