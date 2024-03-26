use abi::timestamp_to_datetime;
use chrono::DateTime;
use chrono::Utc;
use sqlx::postgres::types::PgRange;
use sqlx::Row;

use crate::ReservationManager;
use crate::Rsvp;

impl Rsvp for ReservationManager {
    async fn reserve(
        &self,
        rsvp: abi::Reservation,
    ) -> Result<abi::Reservation, crate::errors::ReservationError> {
        if rsvp.start.is_none() || rsvp.end.is_none() {
            return Err(crate::errors::ReservationError::InvalidTimespan);
        }
        let start = timestamp_to_datetime(rsvp.start.as_ref().unwrap());
        let end = timestamp_to_datetime(rsvp.end.as_ref().unwrap());

        if start >= end {
            return Err(crate::errors::ReservationError::InvalidTimespan);
        }

        let timespan: PgRange<DateTime<Utc>> = (start..end).into();

        let id = sqlx::query(
            r#"
            INSERT INTO rsvp.reservations (user_id, resource_id, status, timespan, note) VALUES ($1, $2, $3, $4, $5)
            RETURNING id
        "#)
        .bind(&rsvp.user_id)
        .bind(&rsvp.resource_id)
        .bind(rsvp.status)
        .bind(timespan)
        .bind(&rsvp.note)
        .fetch_one(&self.pool)
        .await?.get(0);
        let mut rsvp = rsvp;
        rsvp.id = id;

        Ok(rsvp)
    }

    async fn delete(
        &self,
        _rsvp: crate::ReservationId,
    ) -> Result<(), crate::errors::ReservationError> {
        todo!()
    }

    async fn change_status(
        &self,
        _rsvp: crate::ReservationId,
    ) -> Result<abi::Reservation, crate::errors::ReservationError> {
        todo!()
    }

    async fn update_notes(
        &self,
        _rsvp: crate::ReservationId,
        _note: String,
    ) -> Result<abi::Reservation, crate::errors::ReservationError> {
        todo!()
    }

    async fn get(
        &self,
        _rsvp: crate::ReservationId,
    ) -> Result<abi::Reservation, crate::errors::ReservationError> {
        todo!()
    }

    async fn query(
        &self,
        _query: abi::ReservationQuery,
    ) -> Result<Vec<abi::Reservation>, crate::errors::ReservationError> {
        todo!()
    }
}
