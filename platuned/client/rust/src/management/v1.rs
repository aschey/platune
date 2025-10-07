use std::error::Error;

use management_client::ManagementClient;
use tonic::transport::{Channel, Endpoint};

use crate::{StdError, get_ipc_channel};

tonic::include_proto!("platune.management.v1");

impl ManagementClient<Channel> {
    pub async fn connect_http<D>(uri: D) -> Result<ManagementClient<Channel>, Box<dyn Error>>
    where
        D: TryInto<Endpoint>,
        D::Error: Into<StdError>,
    {
        let client = ManagementClient::connect(uri).await?;
        Ok(client)
    }

    pub async fn connect_ipc(name: &str) -> Result<ManagementClient<Channel>, Box<dyn Error>> {
        let channel = get_ipc_channel(name).await?;
        let client = ManagementClient::new(channel);
        Ok(client)
    }
}
