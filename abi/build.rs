fn main() {
    println!("cargo:rerun-if-changed=protos/reservation.proto");

    tonic_build::configure()
        .type_attribute("ReservationQuery", "#[derive(derive_builder::Builder)]")
        .compile(&["protos/reservation.proto"], &["protos"])
        .unwrap();
}
