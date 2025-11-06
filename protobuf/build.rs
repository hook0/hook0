use prost_wkt_build::{FileDescriptorSet, Message};
use std::env;
use std::path::PathBuf;

fn main() {
    // Build protobuf messages
    println!("cargo:rerun-if-changed=./proto");
    let out = PathBuf::from(env::var("OUT_DIR").unwrap());
    let descriptor_file = out.join("descriptors.bin");
    let mut prost_build = prost_build::Config::new();
    prost_build
        .type_attribute(".", "#[derive(serde::Serialize,serde::Deserialize)]")
        .extern_path(".google.protobuf.Any", "::prost_wkt_types::Any")
        .extern_path(".google.protobuf.Timestamp", "::prost_wkt_types::Timestamp")
        .extern_path(".google.protobuf.Value", "::prost_wkt_types::Value")
        .enum_attribute(".", "#[allow(clippy::enum_variant_names)]")
        .file_descriptor_set_path(&descriptor_file)
        .compile_protos(
            &[
                "./proto/object_storage_response.proto",
                "./proto/request_attempt.proto",
            ],
            &["./proto/"],
        )
        .unwrap();
    let descriptor_bytes = std::fs::read(descriptor_file).unwrap();
    let descriptor = FileDescriptorSet::decode(&descriptor_bytes[..]).unwrap();
    prost_wkt_build::add_serde(out, descriptor);
}
