use abi::{Reservation, ReservationQuery};

mod errors;

use errors::ReservationError;
mod manager;
pub type ReservationId = String;

pub trait Rsvp {
    // Reserve a Reservation
    fn reserve(
        &self,
        rsvp: abi::Reservation,
    ) -> impl std::future::Future<Output = Result<Reservation, ReservationError>> + Send;

    // delete a Reservation
    fn delete(
        &self,
        rsvp: ReservationId,
    ) -> impl std::future::Future<Output = Result<(), ReservationError>> + Send;

    // Change a Reservation Status
    // If the reservation is pending, it will be confirmed.
    fn change_status(
        &self,
        rsvp: ReservationId,
    ) -> impl std::future::Future<Output = Result<Reservation, ReservationError>> + Send;

    fn update_notes(
        &self,
        rsvp: ReservationId,
        note: String,
    ) -> impl std::future::Future<Output = Result<Reservation, ReservationError>> + Send;

    fn get(
        &self,
        rsvp: ReservationId,
    ) -> impl std::future::Future<Output = Result<Reservation, ReservationError>> + Send;

    fn query(
        &self,
        query: ReservationQuery,
    ) -> impl std::future::Future<Output = Result<Vec<Reservation>, ReservationError>> + Send;
}

#[derive(Debug)]
pub struct ReservationManager {
    pool: sqlx::PgPool,
}
