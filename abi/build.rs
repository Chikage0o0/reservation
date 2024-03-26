fn main() {
    println!("cargo:rerun-if-changed=protos/reservation.proto");

    tonic_build::configure()
        .type_attribute(
            "ReservationStatus",
            "#[derive(sqlx::Type)] #[sqlx(type_name = \"reservation_status\")] ",
        )
        .compile(&["protos/reservation.proto"], &["protos"])
        .unwrap();
}
