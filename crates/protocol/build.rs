use std::{env, path::PathBuf};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let descriptor_path = PathBuf::from(env::var("OUT_DIR")?)
        .join("opbroker_descriptor.bin");

    tonic_prost_build::configure()
        .file_descriptor_set_path(&descriptor_path)
        .compile_protos(&["proto/opbroker.proto"], &["proto"])?;
    println!("cargo:rerun-if-changed=proto/opbroker.proto");
    Ok(())
}
