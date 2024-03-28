use abi::Reservation;
use abi::ReservationStatus;
use chrono::DateTime;
use chrono::Utc;
use sqlx::postgres::types::PgRange;
use sqlx::FromRow;

use crate::ReservationManager;
use crate::Rsvp;

impl Rsvp for ReservationManager {
    async fn reserve(&self, rsvp: abi::Reservation) -> Result<abi::Reservation, abi::Error> {
        rsvp.validate()?;

        let timespan: PgRange<DateTime<Utc>> = rsvp.timespan()?;
        let status = ReservationStatus::try_from(rsvp.status)
            .unwrap_or(ReservationStatus::Pending)
            .to_string();

        let id :i64= sqlx::query!(
            r#"
            INSERT INTO rsvp.reservations (user_id, resource_id, status, timespan, note) VALUES ($1, $2, $3::rsvp.reservation_status, $4, $5)
            RETURNING id"#,&rsvp.user_id,&rsvp.resource_id,status as _,timespan,&rsvp.note)
        .fetch_one(&self.pool)
        .await?.id;
        let mut rsvp = rsvp;

        rsvp.id = id;

        Ok(rsvp)
    }

    async fn delete(&self, rsvp: crate::ReservationId) -> Result<(), abi::Error> {
        let _ = sqlx::query(
            r#"
            DELETE FROM rsvp.reservations WHERE id=$1
            RETURNING *
            "#,
        )
        .bind(rsvp)
        .fetch_one(&self.pool)
        .await?;

        Ok(())
    }

    async fn change_status(
        &self,
        rsvp: crate::ReservationId,
    ) -> Result<abi::Reservation, abi::Error> {
        // if a reservation is pending, it will be confirmed
        let reservation: Reservation = sqlx::query_as(
            r#"
            UPDATE rsvp.reservations SET status='confirmed' WHERE id=$1 AND status='pending'
            RETURNING *
            "#,
        )
        .bind(rsvp)
        .fetch_one(&self.pool)
        .await?;

        Ok(reservation)
    }

    async fn update_notes(
        &self,
        rsvp: crate::ReservationId,
        note: String,
    ) -> Result<abi::Reservation, abi::Error> {
        let reservation: Reservation = sqlx::query_as(
            r#"
            UPDATE rsvp.reservations SET note=$1 WHERE id=$2
            RETURNING *
            "#,
        )
        .bind(note)
        .bind(rsvp)
        .fetch_one(&self.pool)
        .await?;

        Ok(reservation)
    }

    async fn get(&self, rsvp: crate::ReservationId) -> Result<abi::Reservation, abi::Error> {
        let reservation: Reservation = sqlx::query_as(
            r#"
            SELECT * FROM rsvp.reservations WHERE id=$1
            "#,
        )
        .bind(rsvp)
        .fetch_one(&self.pool)
        .await?;

        Ok(reservation)
    }

    async fn query(
        &self,
        para: abi::ReservationQuery,
    ) -> Result<Vec<abi::Reservation>, abi::Error> {
        let timespan: PgRange<DateTime<Utc>> = para.timespan()?;
        let status = ReservationStatus::try_from(para.status)
            .unwrap_or(ReservationStatus::Unknown)
            .to_string();
        // if user_id is null, find all reservations within during for the resource
        // if resource_id is null, find all reservations within during for the user
        // if both are null, find all reservations within during
        // if both set, find all reservations within during for the resource and user
        // if status == unknown, find all reservations within during
        let query = sqlx::query(
            r#"
            SELECT * FROM rsvp.query($1, $2, $3, $4::rsvp.reservation_status, $5, $6, $7)
            "#,
        )
        .bind(para.user_id)
        .bind(para.resource_id)
        .bind(timespan)
        .bind(status)
        .bind(para.page)
        .bind(para.sort_desc)
        .bind(para.page_size)
        .fetch_all(&self.pool)
        .await?;

        let query = query
            .into_iter()
            .map(|row| Reservation::from_row(&row))
            .collect::<Result<Vec<Reservation>, _>>()?;

        Ok(query)
    }
}

#[cfg(test)]
mod test {
    use abi::error::conflict::{ReservationConflict, ReservationConflictInfo};
    use chrono::Duration;
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

        assert!(rsvp.id != 0);
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
        assert!(result.id != 0);

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

    #[sqlx::test(migrations = "../migrations")]
    async fn get_should_work(pool: PgPool) {
        let manager = ReservationManager { pool: pool.clone() };

        let rsvp = default_rsvp();

        let rsvp1 = manager.reserve(rsvp).await.unwrap();

        let rsvp = manager.get(rsvp1.id).await.unwrap();
        assert_eq!(rsvp.id, rsvp.id);
    }

    #[sqlx::test(migrations = "../migrations")]
    async fn delete_should_work(pool: PgPool) {
        let manager = ReservationManager { pool: pool.clone() };

        let rsvp = default_rsvp();

        let rsvp = manager.reserve(rsvp).await.unwrap();

        manager.delete(rsvp.id).await.unwrap();

        let rsvp = manager.get(rsvp.id).await;
        assert!(matches!(rsvp, Err(abi::Error::NotFound)));
    }

    #[sqlx::test(migrations = "../migrations")]
    async fn delete_null_should_fail(pool: PgPool) {
        let manager = ReservationManager { pool: pool.clone() };

        let result = manager.delete(0).await;
        assert!(matches!(result, Err(abi::Error::NotFound)));
    }

    #[sqlx::test(migrations = "../migrations")]
    async fn query_should_work(pool: PgPool) {
        let manager = ReservationManager { pool: pool.clone() };

        let rsvp = default_rsvp();

        let rsvp = manager.reserve(rsvp).await.unwrap();

        let query = abi::ReservationQueryBuilder::default()
            .end(abi::utils::datetime_to_timestamp(Utc::now()))
            .build()
            .unwrap();

        let query = manager.query(query).await.unwrap();

        assert_eq!(query.len(), 1);
        assert_eq!(query[0].id, rsvp.id);

        let query = abi::ReservationQueryBuilder::default()
            .user_id("user")
            .resource_id("resource")
            .end(abi::utils::datetime_to_timestamp(Utc::now()))
            .build()
            .unwrap();

        let query = manager.query(query).await.unwrap();
        assert_eq!(query.len(), 1);
        assert_eq!(query[0].id, rsvp.id);
    }

    #[sqlx::test(migrations = "../migrations")]
    async fn query_should_work_with_user_id(pool: PgPool) {
        let manager = ReservationManager { pool: pool.clone() };

        let rsvp = default_rsvp();

        let _rsvp = manager.reserve(rsvp).await.unwrap();

        let query = abi::ReservationQueryBuilder::default()
            .user_id("user1")
            .end(abi::utils::datetime_to_timestamp(Utc::now()))
            .build()
            .unwrap();

        let query = manager.query(query).await.unwrap();

        assert_eq!(query.len(), 0);
    }

    #[sqlx::test(migrations = "../migrations")]
    async fn query_should_work_with_resource_id(pool: PgPool) {
        let manager = ReservationManager { pool: pool.clone() };

        let rsvp = default_rsvp();

        let _rsvp = manager.reserve(rsvp).await.unwrap();

        let query = abi::ReservationQueryBuilder::default()
            .resource_id("resource1")
            .end(abi::utils::datetime_to_timestamp(Utc::now()))
            .build()
            .unwrap();

        let query = manager.query(query).await.unwrap();

        assert_eq!(query.len(), 0);
    }

    #[sqlx::test(migrations = "../migrations")]
    async fn query_should_work_with_status(pool: PgPool) {
        let manager = ReservationManager { pool: pool.clone() };

        let rsvp = default_rsvp();

        let _rsvp = manager.reserve(rsvp).await.unwrap();

        let query = abi::ReservationQueryBuilder::default()
            .status(2)
            .end(abi::utils::datetime_to_timestamp(Utc::now()))
            .build()
            .unwrap();

        let query = manager.query(query).await.unwrap();

        assert_eq!(query.len(), 0);
    }

    #[sqlx::test(migrations = "../migrations")]
    async fn query_should_work_with_timespan(pool: PgPool) {
        let manager = ReservationManager { pool: pool.clone() };

        let rsvp = default_rsvp();

        let _rsvp = manager.reserve(rsvp).await.unwrap();

        let query = abi::ReservationQueryBuilder::default()
            .start(abi::utils::datetime_to_timestamp(Utc::now()))
            .end(abi::utils::datetime_to_timestamp(
                Utc::now() + Duration::try_days(1).unwrap(),
            ))
            .build()
            .unwrap();

        let query = manager.query(query).await.unwrap();

        assert_eq!(query.len(), 0);
    }
}
