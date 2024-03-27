#[allow(clippy::all, non_camel_case_types)]
mod pb {

    tonic::include_proto!("reservation");
}
pub use pb::*;

mod errors;
pub use errors::Error;

mod types;
mod utils;
