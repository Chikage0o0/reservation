trait BuilderExt {
    fn add_builder_for_reservation_query(self) -> Self;
    fn add_builder_for_reservation_filter(self) -> Self;
}

fn main() {
    println!("cargo:rerun-if-changed=protos/reservation.proto");

    tonic_build::configure()
        .add_builder_for_reservation_query()
        .add_builder_for_reservation_filter()
        .compile(&["protos/reservation.proto"], &["protos"])
        .unwrap();
}

impl BuilderExt for tonic_build::Builder {
    fn add_builder_for_reservation_query(self) -> Self {
        self.type_attribute("ReservationQuery", "#[derive(derive_builder::Builder)]")
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
            .field_attribute("ReservationQuery.is_desc", "  #[builder(default)]")
            .field_attribute("ReservationQuery.status", "  #[builder(default)]")
    }

    fn add_builder_for_reservation_filter(self) -> Self {
        self.type_attribute("FilterRequest", "#[derive(derive_builder::Builder)]")
            .field_attribute(
                "FilterRequest.user_id",
                "  #[builder(setter(into, strip_option), default)]",
            )
            .field_attribute(
                "FilterRequest.resource_id",
                "  #[builder(setter(into, strip_option), default)]",
            )
            .field_attribute("FilterRequest.cursor", "  #[builder(default)]")
            .field_attribute("FilterRequest.page_size", "  #[builder(default= \"10\")]")
            .field_attribute("FilterRequest.is_desc", "  #[builder(default)]")
            .field_attribute("FilterRequest.status", "  #[builder(default)]")
    }
}
