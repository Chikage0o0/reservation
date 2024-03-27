use abi::Reservation;
use abi::ReservationStatus;
use chrono::DateTime;
use chrono::Utc;
use sqlx::postgres::types::PgRange;
use sqlx::types::Uuid;

use crate::ReservationManager;
use crate::Rsvp;

impl Rsvp for ReservationManager {
    async fn reserve(&self, rsvp: abi::Reservation) -> Result<abi::Reservation, abi::Error> {
        rsvp.validate()?;

        let timespan: PgRange<DateTime<Utc>> = rsvp.timespan()?.into();
        let status = ReservationStatus::try_from(rsvp.status)
            .unwrap_or(ReservationStatus::Unknown)
            .to_string();

        let id :Uuid= sqlx::query!(
            r#"
            INSERT INTO rsvp.reservations (user_id, resource_id, status, timespan, note) VALUES ($1, $2, $3::rsvp.reservation_status, $4, $5)
            RETURNING id"#,&rsvp.user_id,&rsvp.resource_id,status as _,timespan,&rsvp.note)
        .fetch_one(&self.pool)
        .await?.id;
        let mut rsvp = rsvp;

        rsvp.id = id.to_string();

        Ok(rsvp)
    }

    async fn delete(&self, _rsvp: crate::ReservationId) -> Result<(), abi::Error> {
        todo!()
    }

    async fn change_status(
        &self,
        rsvp: crate::ReservationId,
    ) -> Result<abi::Reservation, abi::Error> {
        let uuid = Uuid::parse_str(&rsvp).map_err(|_| abi::Error::InvalidId)?;
        // if a reservation is pending, it will be confirmed
        let reservation: Reservation = sqlx::query_as(
            r#"
            UPDATE rsvp.reservations SET status='confirmed' WHERE id=$1 AND status='pending'
            RETURNING *
            "#,
        )
        .bind(uuid)
        .fetch_one(&self.pool)
        .await?;

        Ok(reservation)
    }

    async fn update_notes(
        &self,
        rsvp: crate::ReservationId,
        note: String,
    ) -> Result<abi::Reservation, abi::Error> {
        let uuid = Uuid::parse_str(&rsvp).map_err(|_| abi::Error::InvalidId)?;
        let reservation: Reservation = sqlx::query_as(
            r#"
            UPDATE rsvp.reservations SET note=$1 WHERE id=$2
            RETURNING *
            "#,
        )
        .bind(note)
        .bind(uuid)
        .fetch_one(&self.pool)
        .await?;

        Ok(reservation)
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
    use abi::error::conflict::{ReservationConflict, ReservationConflictInfo};
    use sqlx::PgPool;

    use super::*;

    fn default_rsvp() -> abi::Reservation {
        abi::Reservation::new_pendding(
            "user",
            "resource",
            DateTime::parse_from_rfc3339("2021-01-01T00:00:00Z").unwrap(),
            DateTime::parse_from_rfc3339("2021-01-02T00:00:00Z").unwrap(),
            "note",
        )
    }

    #[sqlx::test(migrations = "../migrations")]
    async fn reserve_should_work_with_valid_timespan(pool: PgPool) {
        let manager = ReservationManager { pool: pool.clone() };

        let rsvp = default_rsvp();

        let rsvp = manager.reserve(rsvp).await.unwrap();

        assert!(!rsvp.id.is_empty());
    }

    #[sqlx::test(migrations = "../migrations")]
    async fn reserve_should_fail_with_invalid_timespan(pool: PgPool) {
        let manager = ReservationManager { pool: pool.clone() };

        let rsvp = abi::Reservation::new_pendding(
            "user",
            "resource",
            DateTime::parse_from_rfc3339("2021-01-01T00:00:00Z").unwrap(),
            DateTime::parse_from_rfc3339("2021-01-01T00:00:00Z").unwrap(),
            "note",
        );

        let result = manager.reserve(rsvp).await;

        assert!(matches!(result, Err(abi::Error::InvalidTimespan)));
    }

    #[sqlx::test(migrations = "../migrations")]
    async fn reserve_should_fail_with_conflicting_timespan(pool: PgPool) {
        let manager = ReservationManager { pool: pool.clone() };

        let conflict_start = DateTime::parse_from_rfc3339("2021-01-01T12:00:00Z").unwrap();
        let conflict_end = DateTime::parse_from_rfc3339("2021-01-02T12:00:00Z").unwrap();

        let rsvp = default_rsvp();

        let result = manager.reserve(rsvp).await.unwrap();
        assert!(!result.id.is_empty());

        let rsvp = abi::Reservation::new_pendding(
            "user",
            "resource",
            conflict_start,
            conflict_end,
            "note",
        );

        let result = manager.reserve(rsvp).await;

        assert!(matches!(result, Err(abi::Error::ConflictReservation(_))));
        match result.unwrap_err() {
            abi::Error::ConflictReservation(ReservationConflictInfo::Parsed(
                ReservationConflict { new, old },
            )) => {
                assert_eq!(new.resource_id, "resource");
                assert_eq!(old.resource_id, "resource");
                assert_eq!(new.start, conflict_start);
                assert_eq!(new.end, conflict_end);
            }

            e => {
                eprintln!("{:?}", e);
                panic!("Unexpected error");
            }
        };
    }

    #[sqlx::test(migrations = "../migrations")]
    async fn reservation_can_be_confirmed(pool: PgPool) {
        let manager = ReservationManager { pool: pool.clone() };

        let rsvp = default_rsvp();

        let rsvp = manager.reserve(rsvp).await.unwrap();

        let rsvp = manager.change_status(rsvp.id).await.unwrap();
        assert_eq!(rsvp.status, abi::ReservationStatus::Confirmed as i32);
    }

    #[sqlx::test(migrations = "../migrations")]
    async fn change_reservation_again_should_do_nothing(pool: PgPool) {
        let manager = ReservationManager { pool: pool.clone() };

        let rsvp = default_rsvp();

        let rsvp = manager.reserve(rsvp).await.unwrap();

        let rsvp = manager.change_status(rsvp.id).await.unwrap();
        assert_eq!(rsvp.status, abi::ReservationStatus::Confirmed as i32);

        let ret = manager.change_status(rsvp.id).await;
        assert!(matches!(ret, Err(abi::Error::NotFound)));
    }

    #[sqlx::test(migrations = "../migrations")]
    async fn update_notes_should_work(pool: PgPool) {
        let manager = ReservationManager { pool: pool.clone() };

        let rsvp = default_rsvp();

        let rsvp = manager.reserve(rsvp).await.unwrap();

        let rsvp = manager
            .update_notes(rsvp.id, "new note".to_string())
            .await
            .unwrap();
        assert_eq!(rsvp.note, "new note");
    }
}
