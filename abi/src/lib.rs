#[allow(clippy::all, non_camel_case_types)]
mod pb {

    tonic::include_proto!("reservation");
}

use std::time::SystemTime;

use chrono::{DateTime, Utc};
pub use pb::*;

pub fn timestamp_to_datetime(ts: &prost_types::Timestamp) -> DateTime<Utc> {
    DateTime::from(
        SystemTime::UNIX_EPOCH
            + std::time::Duration::from_secs(ts.seconds as u64)
            + std::time::Duration::from_nanos(ts.nanos as u64),
    )
}
