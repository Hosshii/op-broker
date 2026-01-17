pub mod pb {
    tonic::include_proto!("macos_remote");
    pub const FILE_DESCRIPTOR_SET: &[u8] =
        tonic::include_file_descriptor_set!("macos_remote_descriptor");
}

pub use pb::*;
