#[allow(clippy::all, non_camel_case_types)]
mod pb {

    tonic::include_proto!("reservation");
}

pub use pb::*;
