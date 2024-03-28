fn main() {
    println!("cargo:rerun-if-changed=protos/reservation.proto");

    tonic_build::configure()
        .type_attribute("ReservationQuery", "#[derive(derive_builder::Builder)]")
        .field_attribute(
            "ReservationQuery.user_id",
            "  #[builder(setter(into, strip_option), default)]",
        )
        .field_attribute(
            "ReservationQuery.resource_id",
            "  #[builder(setter(into, strip_option), default)]",
        )
        .field_attribute(
            "ReservationQuery.start",
            "  #[builder(setter(into, strip_option), default)]",
        )
        .field_attribute(
            "ReservationQuery.end",
            "  #[builder(setter(into, strip_option), default)]",
        )
        .field_attribute("ReservationQuery.page", "  #[builder(default= \"1\")]")
        .field_attribute(
            "ReservationQuery.page_size",
            "  #[builder(default= \"10\")]",
        )
        .field_attribute("ReservationQuery.sort_desc", "  #[builder(default)]")
        .field_attribute("ReservationQuery.status", "  #[builder(default)]")
        .compile(&["protos/reservation.proto"], &["protos"])
        .unwrap();
}
