use std::fmt::{Display, Formatter};

use crate::ReservationStatus;

impl Display for ReservationStatus {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match &self {
            ReservationStatus::Pending => write!(f, "pending"),
            ReservationStatus::Confirmed => write!(f, "confirmed"),
            ReservationStatus::Blocked => write!(f, "blocked"),
            ReservationStatus::Unknown => write!(f, "unknown"),
        }
    }
}
