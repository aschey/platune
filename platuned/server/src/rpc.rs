tonic::include_proto!("player_rpc");
tonic::include_proto!("management_rpc");
include!(concat!(env!("OUT_DIR"), "/management_rpc.serde.rs"));
include!(concat!(env!("OUT_DIR"), "/player_rpc.serde.rs"));

pub(crate) const FILE_DESCRIPTOR_SET: &[u8] = tonic::include_file_descriptor_set!("rpc_descriptor");
