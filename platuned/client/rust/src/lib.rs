use crate::management_client::ManagementClient;
use crate::player_client::PlayerClient;
use rpc::*;
use std::convert::TryInto;
use std::error::Error;
use tonic::{
    codegen::StdError,
    transport::{Channel, Endpoint},
};

pub mod rpc {
    tonic::include_proto!("player_rpc");
    tonic::include_proto!("management_rpc");
}

pub async fn get_player_client<D>(uri: D) -> Result<PlayerClient<Channel>, Box<dyn Error>>
where
    D: TryInto<Endpoint>,
    D::Error: Into<StdError>,
{
    let client = PlayerClient::connect(uri).await?;
    Ok(client)
}

pub async fn get_management_client<D>(uri: D) -> Result<ManagementClient<Channel>, Box<dyn Error>>
where
    D: TryInto<Endpoint>,
    D::Error: Into<StdError>,
{
    let client = ManagementClient::connect(uri).await?;
    Ok(client)
}
