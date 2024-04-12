use std::error::Error;

use parity_tokio_ipc::{IpcEndpoint, ServerId};
use rpc::*;
use tonic::codegen::StdError;
use tonic::transport::{Channel, Endpoint, Uri};
use tower::service_fn;

pub use crate::management_client::*;
pub use crate::player_client::*;

pub mod rpc {
    tonic::include_proto!("player_rpc");
    tonic::include_proto!("management_rpc");
}

pub async fn connect_player_http<D>(uri: D) -> Result<PlayerClient<Channel>, Box<dyn Error>>
where
    D: TryInto<Endpoint>,
    D::Error: Into<StdError>,
{
    let client = PlayerClient::connect(uri).await?;
    Ok(client)
}

pub async fn connect_player_ipc() -> Result<PlayerClient<Channel>, Box<dyn Error>> {
    let channel = get_ipc_channel().await?;
    let client = PlayerClient::new(channel);
    Ok(client)
}

pub async fn connect_management_http<D>(uri: D) -> Result<ManagementClient<Channel>, Box<dyn Error>>
where
    D: TryInto<Endpoint>,
    D::Error: Into<StdError>,
{
    let client = ManagementClient::connect(uri).await?;
    Ok(client)
}

pub async fn connect_management_ipc() -> Result<ManagementClient<Channel>, Box<dyn Error>> {
    let channel = get_ipc_channel().await?;
    let client = ManagementClient::new(channel);
    Ok(client)
}

async fn get_ipc_channel() -> Result<Channel, Box<dyn Error>> {
    let channel = tonic::transport::Endpoint::try_from("http://dummy")?
        .connect_with_connector(service_fn(|_: Uri| {
            parity_tokio_ipc::Endpoint::connect(ServerId("platune/platuned"))
        }))
        .await?;
    Ok(channel)
}
