fn main() {
    println!("cargo:rerun-if-changed=protos/reservation.proto");

    tonic_build::configure()
        .compile(&["protos/reservation.proto"], &["protos"])
        .unwrap();
}
