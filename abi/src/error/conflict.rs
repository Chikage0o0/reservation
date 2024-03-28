use std::{convert::Infallible, str::FromStr, sync::OnceLock};

use chrono::{DateTime, Utc};
use regex::Regex;

static RESERVATION_CONFLICT_REGEX: OnceLock<Regex> = OnceLock::new();

#[derive(Debug)]
pub enum ReservationConflictInfo {
    Parsed(ReservationConflict),
    Raw(String),
}

#[derive(Debug)]
pub struct ReservationConflict {
    pub new: ReservationWindow,
    pub old: ReservationWindow,
}
#[derive(Debug)]
pub struct ReservationWindow {
    pub resource_id: String,
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
}

impl FromStr for ReservationConflictInfo {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Ok(parsed) = s.parse() {
            return Ok(ReservationConflictInfo::Parsed(parsed));
        }
        Ok(ReservationConflictInfo::Raw(s.to_string()))
    }
}

impl FromStr for ReservationConflict {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let re=RESERVATION_CONFLICT_REGEX.get_or_init(|| Regex::new(r#"\(resource_id, timespan\)=\((\w+), \[\"([\d-]+\s[\d:+]+)\",\"([\d-]+\s[\d:+]+)\"\)\)"#).unwrap());

        const TIME_FORMAT: &str = "%Y-%m-%d %H:%M:%S%#z";
        let mut cap = re.captures_iter(s);
        let cap1 = cap.next().ok_or(())?;
        let new = ReservationWindow {
            resource_id: cap1[1].to_string(),
            start: DateTime::parse_from_str(&cap1[2], TIME_FORMAT)
                .map_err(|_| ())?
                .with_timezone(&Utc),
            end: DateTime::parse_from_str(&cap1[3], TIME_FORMAT)
                .map_err(|_| ())?
                .with_timezone(&Utc),
        };

        let cap2 = cap.next().ok_or(())?;
        let old = ReservationWindow {
            resource_id: cap2[1].to_string(),
            start: DateTime::parse_from_str(&cap2[2], TIME_FORMAT)
                .map_err(|_| ())?
                .with_timezone(&Utc),
            end: DateTime::parse_from_str(&cap2[3], TIME_FORMAT)
                .map_err(|_| ())?
                .with_timezone(&Utc),
        };

        Ok(ReservationConflict { new, old })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_conflict() {
        let conflict = "Key (resource_id, timespan)=(resource, [\"2021-01-01 12:00:00+00\",\"2021-01-02 12:00:00+00\")) conflicts with existing key (resource_id, timespan)=(resource, [\"2021-01-01 00:00:00+00\",\"2021-01-02 00:00:00+00\")).";
        let conflict: ReservationConflictInfo = conflict.parse().unwrap();
        dbg!(&conflict);
        assert!(matches!(conflict, ReservationConflictInfo::Parsed(_)));
    }
}
