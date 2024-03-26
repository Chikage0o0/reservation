#[allow(clippy::all, non_camel_case_types)]
mod pb {

    tonic::include_proto!("reservation");
}

use std::{
    fmt::{Display, Formatter},
    time::SystemTime,
};

use chrono::{DateTime, FixedOffset, Utc};
pub use pb::*;

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

impl Display for ReservationStatus {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match &self {
            ReservationStatus::Pending => write!(f, "pending"),
            ReservationStatus::Confirmed => write!(f, "confirmed"),
            ReservationStatus::Blocked => write!(f, "blocked"),
            ReservationStatus::Unknown => write!(f, "unknown"),
        }
    }
}

impl Reservation {
    pub fn new_pendding(
        user_id: impl Into<String>,
        resource_id: impl Into<String>,
        start: DateTime<FixedOffset>,
        end: DateTime<FixedOffset>,
        note: impl Into<String>,
    ) -> Self {
        Self {
            id: "".to_string(),
            user_id: user_id.into(),
            resource_id: resource_id.into(),
            status: ReservationStatus::Pending as i32,
            start: Some(datetime_to_timestamp(start.with_timezone(&Utc))),
            end: Some(datetime_to_timestamp(end.with_timezone(&Utc))),
            note: note.into(),
        }
    }
}
