use std::error::Error;

use hyper_util::rt::TokioIo;
use tipsy::ServerId;
use tonic::codegen::StdError;
pub use tonic::transport::{Channel, Endpoint, Uri};
use tower::service_fn;

pub mod management;
pub mod player;

async fn get_ipc_channel() -> Result<Channel, Box<dyn Error>> {
    let endpoint = tonic::transport::Endpoint::try_from("http://dummy")?;
    let channel = endpoint
        .connect_with_connector(service_fn(|_: Uri| async move {
            Ok::<_, Box<dyn Error + Send + Sync>>(TokioIo::new(
                tipsy::Endpoint::connect(ServerId::new("platune/platuned").parent_folder("/tmp"))
                    .await?,
            ))
        }))
        .await?;

    Ok(channel)
}
