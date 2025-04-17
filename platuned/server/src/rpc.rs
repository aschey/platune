pub(crate) mod v1 {
    tonic::include_proto!("platune.player.v1");
    tonic::include_proto!("platune.management.v1");
}

pub(crate) const FILE_DESCRIPTOR_SET: &[u8] = tonic::include_file_descriptor_set!("rpc_descriptor");
