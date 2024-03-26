use abi::timestamp_to_datetime;
use chrono::DateTime;
use chrono::Utc;
use sqlx::postgres::types::PgRange;
use sqlx::types::Uuid;
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

        let status = abi::ReservationStatus::try_from(rsvp.status)
            .unwrap_or(abi::ReservationStatus::Pending)
            .to_string();

        if start >= end {
            return Err(crate::errors::ReservationError::InvalidTimespan);
        }

        let timespan: PgRange<DateTime<Utc>> = (start..end).into();

        let id :Uuid= sqlx::query(
            r#"
            INSERT INTO rsvp.reservations (user_id, resource_id, status, timespan, note) VALUES ($1, $2, $3::rsvp.reservation_status, $4, $5)
            RETURNING id
        "#)
        .bind(&rsvp.user_id)
        .bind(&rsvp.resource_id)
        .bind(status)
        .bind(timespan)
        .bind(&rsvp.note)
        .fetch_one(&self.pool)
        .await?.get(0);
        let mut rsvp = rsvp;
        rsvp.id = id.to_string();

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

#[cfg(test)]
mod test {
    use sqlx::PgPool;

    use super::*;

    #[sqlx::test(migrations = "../migrations")]
    async fn reserve_should_work_with_valid_timespan(pool: PgPool) {
        let manager = ReservationManager { pool: pool.clone() };

        let rsvp = abi::Reservation::new_pendding(
            "user",
            "resource",
            DateTime::parse_from_rfc3339("2021-01-01T00:00:00Z").unwrap(),
            DateTime::parse_from_rfc3339("2021-01-02T00:00:00Z").unwrap(),
            "note",
        );

        let rsvp = manager.reserve(rsvp).await.unwrap();

        assert!(!rsvp.id.is_empty());
    }

    #[sqlx::test(migrations = "../migrations")]
    async fn reserve_should_fail_with_invalid_timespan(pool: PgPool) {
        let manager = ReservationManager { pool: pool.clone() };

        let rsvp = abi::Reservation::new_pendding(
            "user",
            "resource",
            DateTime::parse_from_rfc3339("2021-01-02T00:00:00Z").unwrap(),
            DateTime::parse_from_rfc3339("2021-01-01T00:00:00Z").unwrap(),
            "note",
        );

        let result = manager.reserve(rsvp).await;

        assert!(result.is_err());
    }

    #[sqlx::test(migrations = "../migrations")]
    async fn reserve_should_fail_with_missing_timespan(pool: PgPool) {
        let manager = ReservationManager { pool: pool.clone() };

        let rsvp = abi::Reservation {
            id: "".to_string(),
            user_id: "user".to_string(),
            resource_id: "resource".to_string(),
            status: 0,
            start: None,
            end: None,
            note: "note".to_string(),
        };

        let result = manager.reserve(rsvp).await;

        assert!(result.is_err());
    }

    #[sqlx::test(migrations = "../migrations")]
    async fn reserve_should_fail_with_conflicting_timespan(pool: PgPool) {
        let manager = ReservationManager { pool: pool.clone() };

        let rsvp = abi::Reservation::new_pendding(
            "user",
            "resource",
            DateTime::parse_from_rfc3339("2021-01-01T00:00:00Z").unwrap(),
            DateTime::parse_from_rfc3339("2021-01-02T00:00:00Z").unwrap(),
            "note",
        );

        let result = manager.reserve(rsvp).await.unwrap();
        assert!(!result.id.is_empty());

        let rsvp = abi::Reservation::new_pendding(
            "user",
            "resource",
            DateTime::parse_from_rfc3339("2021-01-01T12:00:00Z").unwrap(),
            DateTime::parse_from_rfc3339("2021-01-02T12:00:00Z").unwrap(),
            "note",
        );

        let result = manager.reserve(rsvp).await;

        assert!(result.is_err());
    }
}
