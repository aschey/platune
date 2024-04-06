use std::env;
use std::error::Error;
use std::path::{Path, PathBuf};

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
            let socket_path = if cfg!(unix) {
                let socket_base = match env::var("XDG_RUNTIME_DIR") {
                    Ok(socket_base) => socket_base,
                    Err(_) => "/tmp".to_owned(),
                };
                Path::new(&socket_base).join("platuned/platuned.sock")
            } else {
                PathBuf::from(r"\\.\pipe\platuned")
            };
            parity_tokio_ipc::Endpoint::connect(socket_path)
        }))
        .await?;
    Ok(channel)
}
