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

impl From<Error> for tonic::Status {
    fn from(e: Error) -> Self {
        match e {
            Error::InvalidUserId => tonic::Status::invalid_argument("Invalid User ID"),
            Error::InvalidTimespan => tonic::Status::invalid_argument("Invalid timespan"),
            Error::ConflictReservation(e) => tonic::Status::already_exists(format!("{:?}", e)),
            Error::Unknown => tonic::Status::unknown("Unknown error"),
            Error::InvalidId => tonic::Status::invalid_argument("Invalid ID"),
            Error::DatabaseError(_) => tonic::Status::internal("Database error"),
            Error::NotFound => tonic::Status::not_found("Row not found"),
            Error::IoError(_) => tonic::Status::internal("IO error"),
            Error::InvalidConfig(_) => tonic::Status::invalid_argument("Invalid config"),
        }
    }
}
