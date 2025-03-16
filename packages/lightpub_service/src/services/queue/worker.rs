use activitypub_federation::config::Data;
use tokio_util::sync::CancellationToken;

use crate::{MyFederationData, ServiceResult};

use super::QConn;

pub struct ApubWorker {
    qconn: QConn,
}

impl ApubWorker {
    pub fn new(qconn: QConn) -> Self {
        Self { qconn }
    }

    pub async fn start(
        &self,
        data: &Data<MyFederationData>,
        cancel: &CancellationToken,
    ) -> ServiceResult<()> {
        let apub = self.qconn.get_apub_delivery_stream().await?;
        let consumer = apub.create_consumer().await?;
        consumer.process_loop(data, cancel).await
    }
}
