use sqlx::postgres::PgDatabaseError;
use thiserror::Error;

use self::conflict::ReservationConflictInfo;

pub mod conflict;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Invalid User ID")]
    InvalidUserId,

    #[error("Invalid timespan")]
    InvalidTimespan,

    #[error("Conflict reservation")]
    ConflictReservation(ReservationConflictInfo),

    #[error("Unknown error")]
    Unknown,

    #[error("Invalid ID")]
    InvalidId,

    #[error("Database error: {0}")]
    DatabaseError(sqlx::Error),

    #[error("Row not found")]
    NotFound,

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Invalid config: {0}")]
    InvalidConfig(String),
}

impl From<sqlx::Error> for Error {
    fn from(e: sqlx::Error) -> Self {
        match e {
            sqlx::Error::Database(e) => {
                let err: &PgDatabaseError = e.downcast_ref();

                match (err.code(), err.schema(), err.table()) {
                    ("23P01", Some("rsvp"), Some("reservations")) => Error::ConflictReservation(
                        err.detail().unwrap_or_default().parse().unwrap(),
                    ),
                    _ => Error::DatabaseError(sqlx::Error::Database(e)),
                }
            }
            sqlx::Error::RowNotFound => Error::NotFound,
            _ => Error::DatabaseError(e),
        }
    }
}
