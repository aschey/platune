use player_rpc::player_client::PlayerClient;
use std::convert::TryInto;
use std::error::Error;
use tonic::{
    codegen::StdError,
    transport::{Channel, Endpoint},
};

pub mod player_rpc {
    tonic::include_proto!("player_rpc");
}

pub async fn get_client<D>(uri: D) -> Result<PlayerClient<Channel>, Box<dyn Error>>
where
    D: TryInto<Endpoint>,
    D::Error: Into<StdError>,
{
    let client = PlayerClient::connect(uri).await?;
    Ok(client)
}
