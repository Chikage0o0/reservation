#[allow(clippy::all, non_camel_case_types)]
mod pb {

    tonic::include_proto!("reservation");
}
pub use pb::*;

pub mod error;
pub use error::Error;

mod types;
mod utils;
