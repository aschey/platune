use std::error::Error;

use player_client::PlayerClient;
use tonic::transport::{Channel, Endpoint};

use crate::{StdError, get_ipc_channel};

tonic::include_proto!("platune.player.v1");

impl PlayerClient<Channel> {
    pub async fn connect_http<D>(uri: D) -> Result<PlayerClient<Channel>, Box<dyn Error>>
    where
        D: TryInto<Endpoint>,
        D::Error: Into<StdError>,
    {
        let client = PlayerClient::connect(uri).await?;
        Ok(client)
    }

    pub async fn connect_ipc() -> Result<PlayerClient<Channel>, Box<dyn Error>> {
        let channel = get_ipc_channel().await?;
        let client = PlayerClient::new(channel);
        Ok(client)
    }
}
