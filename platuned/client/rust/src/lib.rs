use std::io;

use hyper_util::rt::TokioIo;
use tipsy::ServerId;
use tonic::codegen::StdError;
pub use tonic::transport::{Channel, Endpoint, Uri};
use tower::service_fn;

pub mod management;
pub mod player;

async fn get_ipc_channel(name: &str) -> Result<Channel, tonic::transport::Error> {
    let endpoint = tonic::transport::Endpoint::try_from("http://dummy")?;
    let name = name.to_string();
    let channel = endpoint
        .connect_with_connector(service_fn(move |_: Uri| {
            let name = name.clone();
            async move {
                Ok::<_, io::Error>(TokioIo::new(
                    tipsy::Endpoint::connect(
                        ServerId::new(format!("platune/{name}")).parent_folder("/tmp"),
                    )
                    .await?,
                ))
            }
        }))
        .await?;

    Ok(channel)
}
