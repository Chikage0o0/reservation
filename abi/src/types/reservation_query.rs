use chrono::{DateTime, Utc};
use sqlx::postgres::types::PgRange;

use crate::{utils::timestamp_to_datetime, Error, ReservationQuery};

impl ReservationQuery {
    pub fn timespan(&self) -> Result<PgRange<DateTime<Utc>>, Error> {
        let range = match (&self.start, &self.end) {
            (Some(start), Some(end)) => {
                if start.seconds >= end.seconds && start.nanos >= end.nanos {
                    return Err(Error::InvalidTimespan);
                }
                (timestamp_to_datetime(start)..timestamp_to_datetime(end)).into()
            }
            (None, None) => return Err(Error::InvalidTimespan),
            (None, Some(end)) => (..timestamp_to_datetime(end)).into(),
            (Some(start), None) => (timestamp_to_datetime(start)..).into(),
        };
        Ok(range)
    }
}
