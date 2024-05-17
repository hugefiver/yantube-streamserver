pub mod errors;
pub mod wish_entrypoint;
pub mod session;

pub use self::session::WebRTCServerSession;
pub use self::wish_entrypoint::WishEntrypointServer;

use streamhub::{
    define::{
        DataSender, InformationSender, 
         SubscribeType, TStreamHandler,
    },
    errors::StreamHubError,
    statistics::StatisticsStream,
};
use tokio::sync::Mutex;
use async_trait::async_trait;

#[derive(Default)]
pub struct WebRTCStreamHandler {
    sps: Mutex<Vec<u8>>,
    pps: Mutex<Vec<u8>>,
}

impl WebRTCStreamHandler {
    pub async fn set_sps(&self, sps: Vec<u8>) {
        *self.sps.lock().await = sps;
    }
    pub async fn set_pps(&self, pps: Vec<u8>) {
        *self.pps.lock().await = pps;
    }
}

#[async_trait]
impl TStreamHandler for WebRTCStreamHandler {
    async fn send_prior_data(
        &self, _data_sender: DataSender, _sub_type: SubscribeType,
    ) -> Result<(), StreamHubError> {
        Ok(())
    }
    async fn get_statistic_data(&self) -> Option<StatisticsStream> {
        None
    }

    async fn send_information(&self, _sender: InformationSender) {}
}
