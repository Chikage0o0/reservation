use std::time::SystemTime;

use chrono::{DateTime, Utc};

pub fn timestamp_to_datetime(ts: &prost_types::Timestamp) -> DateTime<Utc> {
    DateTime::from(
        SystemTime::UNIX_EPOCH
            + std::time::Duration::from_secs(ts.seconds as u64)
            + std::time::Duration::from_nanos(ts.nanos as u64),
    )
}

pub fn datetime_to_timestamp(dt: DateTime<Utc>) -> prost_types::Timestamp {
    let duration = dt - DateTime::from(SystemTime::UNIX_EPOCH);
    prost_types::Timestamp {
        seconds: duration.num_seconds(),
        nanos: duration.subsec_nanos(),
    }
}
