use std::ops::{Bound, Range};

use chrono::{DateTime, FixedOffset, Utc};
use sqlx::{
    postgres::{types::PgRange, PgRow},
    types::Uuid,
    FromRow, Row,
};

use crate::{
    utils::{datetime_to_timestamp, timestamp_to_datetime},
    Error, Reservation, ReservationStatus,
};

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

    pub fn validate(&self) -> Result<(), Error> {
        if self.user_id.is_empty() {
            return Err(Error::InvalidUserId);
        }

        Ok(())
    }

    pub fn timespan(&self) -> Result<Range<DateTime<Utc>>, Error> {
        if self.start.is_none() || self.end.is_none() {
            return Err(Error::InvalidTimespan);
        }
        if self.start.as_ref().unwrap().seconds >= self.end.as_ref().unwrap().seconds
            && self.start.as_ref().unwrap().nanos >= self.end.as_ref().unwrap().nanos
        {
            return Err(Error::InvalidTimespan);
        }

        let start = timestamp_to_datetime(self.start.as_ref().unwrap());
        let end = timestamp_to_datetime(self.end.as_ref().unwrap());
        Ok(start..end)
    }
}

impl FromRow<'_, PgRow> for Reservation {
    fn from_row(row: &sqlx::postgres::PgRow) -> std::result::Result<Self, sqlx::Error> {
        let id = row.get::<Uuid, _>("id").to_string();

        let range = row.get::<PgRange<DateTime<Utc>>, _>("timespan");
        let start = match range.start {
            Bound::Included(start) | Bound::Excluded(start) => Some(start),
            Bound::Unbounded => None,
        }
        .map(datetime_to_timestamp);
        let end = match range.end {
            Bound::Included(end) | Bound::Excluded(end) => Some(end),
            Bound::Unbounded => None,
        }
        .map(datetime_to_timestamp);

        let status: ReservationStatus = row.get("status");

        Ok(Self {
            id,
            user_id: row.get("user_id"),
            resource_id: row.get("resource_id"),
            status: status as i32,
            start,
            end,
            note: row.get("note"),
        })
    }
}
