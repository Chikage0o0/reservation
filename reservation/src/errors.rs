use thiserror::Error;

#[derive(Error, Debug)]
pub enum ReservationError {
    #[error("Invalid timespan")]
    InvalidTimespan,

    #[error("Database error: {0}")]
    DatabaseError(#[from] sqlx::Error),
}
