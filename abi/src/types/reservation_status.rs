// use crate::{PgReservationStatus, ReservationStatus};

// impl From<PgReservationStatus> for ReservationStatus {
//     fn from(status: PgReservationStatus) -> Self {
//         match status {
//             PgReservationStatus::Pending => ReservationStatus::Pending,
//             PgReservationStatus::Confirmed => ReservationStatus::Confirmed,
//             PgReservationStatus::Blocked => ReservationStatus::Blocked,
//             PgReservationStatus::Unknown => ReservationStatus::Unknown,
//         }
//     }
// }

// impl From<ReservationStatus> for PgReservationStatus {
//     fn from(status: ReservationStatus) -> Self {
//         match status {
//             ReservationStatus::Pending => PgReservationStatus::Pending,
//             ReservationStatus::Confirmed => PgReservationStatus::Confirmed,
//             ReservationStatus::Blocked => PgReservationStatus::Blocked,
//             ReservationStatus::Unknown => PgReservationStatus::Unknown,
//         }
//     }
// }

use std::fmt::Display;

use crate::ReservationStatus;

impl sqlx::Type<sqlx::Postgres> for ReservationStatus {
    fn type_info() -> <sqlx::Postgres as sqlx::Database>::TypeInfo {
        sqlx::postgres::PgTypeInfo::with_name("reservation_status")
    }
}

impl sqlx::Encode<'_, sqlx::Postgres> for ReservationStatus {
    fn encode_by_ref(
        &self,
        buf: &mut <sqlx::Postgres as sqlx::database::HasArguments<'_>>::ArgumentBuffer,
    ) -> sqlx::encode::IsNull {
        let status = match self {
            ReservationStatus::Pending => "pending",
            ReservationStatus::Confirmed => "confirmed",
            ReservationStatus::Blocked => "blocked",
            ReservationStatus::Unknown => "unknown",
        };

        buf.extend(status.as_bytes());
        sqlx::encode::IsNull::No
    }
}

impl sqlx::Decode<'_, sqlx::Postgres> for ReservationStatus {
    fn decode(
        value: <sqlx::Postgres as sqlx::database::HasValueRef<'_>>::ValueRef,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let status = value.as_str()?;

        Ok(match status {
            "pending" => ReservationStatus::Pending,
            "confirmed" => ReservationStatus::Confirmed,
            "blocked" => ReservationStatus::Blocked,
            "unknown" => ReservationStatus::Unknown,
            _ => return Err("Invalid status".into()),
        })
    }
}

impl Display for ReservationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let status = match self {
            ReservationStatus::Pending => "pending",
            ReservationStatus::Confirmed => "confirmed",
            ReservationStatus::Blocked => "blocked",
            ReservationStatus::Unknown => "unknown",
        };

        write!(f, "{}", status)
    }
}
