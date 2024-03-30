use tokio::sync::mpsc::Receiver;

use abi::{config::DbConfig, Reservation, ReservationFilter, ReservationQuery};
use sqlx::Error;

mod manager;
pub type ReservationId = i64;

pub trait Rsvp {
    // Reserve a Reservation
    fn reserve(
        &self,
        rsvp: abi::Reservation,
    ) -> impl std::future::Future<Output = Result<Reservation, abi::Error>> + Send;

    /// delete a Reservation
    fn delete(
        &self,
        rsvp: ReservationId,
    ) -> impl std::future::Future<Output = Result<(), abi::Error>> + Send;

    // Change a Reservation Status
    // If the reservation is pending, it will be confirmed.
    fn change_status(
        &self,
        rsvp: ReservationId,
    ) -> impl std::future::Future<Output = Result<Reservation, abi::Error>> + Send;

    fn update_notes(
        &self,
        rsvp: ReservationId,
        note: String,
    ) -> impl std::future::Future<Output = Result<Reservation, abi::Error>> + Send;

    fn get(
        &self,
        rsvp: ReservationId,
    ) -> impl std::future::Future<Output = Result<Reservation, abi::Error>> + Send;

    fn query(
        &self,
        query: ReservationQuery,
    ) -> impl std::future::Future<
        Output = Result<Receiver<Result<abi::Reservation, abi::Error>>, abi::Error>,
    > + Send;

    fn filter(
        &self,
        filter: ReservationFilter,
    ) -> impl std::future::Future<Output = Result<(abi::FilterPager, Vec<Reservation>), abi::Error>> + Send;
}

#[derive(Debug)]
pub struct ReservationManager {
    pool: sqlx::PgPool,
}

impl ReservationManager {
    pub async fn from_config(confg: &DbConfig) -> Result<Self, Error> {
        let db_url = confg.db_url();
        let pool = sqlx::PgPool::connect(&db_url).await?;
        Ok(Self { pool })
    }

    pub fn new(pool: sqlx::PgPool) -> Self {
        Self { pool }
    }
}
