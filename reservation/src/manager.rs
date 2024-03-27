use abi::ReservationStatus;
use chrono::DateTime;
use chrono::Utc;
use sqlx::postgres::types::PgRange;
use sqlx::types::Uuid;
use sqlx::Row;

use crate::ReservationManager;
use crate::Rsvp;

impl Rsvp for ReservationManager {
    async fn reserve(&self, rsvp: abi::Reservation) -> Result<abi::Reservation, abi::Error> {
        rsvp.validate()?;

        let timespan: PgRange<DateTime<Utc>> = rsvp.timespan()?.into();
        let status = ReservationStatus::try_from(rsvp.status)
            .unwrap_or(ReservationStatus::Unknown)
            .to_string();

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

    async fn delete(&self, _rsvp: crate::ReservationId) -> Result<(), abi::Error> {
        todo!()
    }

    async fn change_status(
        &self,
        _rsvp: crate::ReservationId,
    ) -> Result<abi::Reservation, abi::Error> {
        todo!()
    }

    async fn update_notes(
        &self,
        _rsvp: crate::ReservationId,
        _note: String,
    ) -> Result<abi::Reservation, abi::Error> {
        todo!()
    }

    async fn get(&self, _rsvp: crate::ReservationId) -> Result<abi::Reservation, abi::Error> {
        todo!()
    }

    async fn query(
        &self,
        _query: abi::ReservationQuery,
    ) -> Result<Vec<abi::Reservation>, abi::Error> {
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

        assert!(matches!(result, Err(abi::Error::InvalidTimespan)));
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

        assert!(matches!(result, Err(abi::Error::ConflictReservation(_))));
    }
}
