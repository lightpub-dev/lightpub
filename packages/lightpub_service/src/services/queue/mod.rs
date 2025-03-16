use std::time::Duration;

use activitypub_federation::{
    activity_sending::SendActivityTask, config::Data, protocol::context::WithContext,
    traits::ActivityHandler,
};
use async_nats::{
    Client,
    jetstream::{self, Message, stream},
};
use delay::APUB_DELIVERY_DELAY_TIMES;
use expected_error::StatusCode;
use expected_error_derive::ExpectedError;
use futures_util::StreamExt;
use itertools::Itertools;
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use thiserror::Error;
use tokio::select;
use tokio_util::sync::CancellationToken;
use tracing::{debug, error, info};
use url::Url;

mod delay;
mod worker;
use crate::{MyFederationData, services::ServiceError};
pub use worker::ApubWorker;

use super::{
    MapToUnknown, ServiceResult,
    apub::{
        AcceptActivity, AnnounceActivity, CreateActivity, DeleteActivity, FollowActivity,
        LikeActivity, RejectActivity, UndoActivity, UpdateActivity,
    },
    user::UserWithApubModel,
};

#[derive(Debug, Clone)]
pub struct QConn {
    client: Client,
}

const APUB_STREAM_NAME: &str = "APUB";
const APUB_DELIVERY_SUBJECT: &str = "apub.delivery";
const APUB_DELIVERY_CONSUMER: &str = "apub-delivery-processor";

#[derive(Debug, Clone, Error, ExpectedError)]
pub enum NatsError {
    #[ee(status(StatusCode::INTERNAL_SERVER_ERROR))]
    #[error("request timed out")]
    Timeout,
    #[ee(status(StatusCode::INTERNAL_SERVER_ERROR))]
    #[error("no request backend available")]
    NoRequestBackend,
    #[ee(status(StatusCode::INTERNAL_SERVER_ERROR))]
    #[error("Internal Server Error")]
    DeserializationFailed,
}

impl QConn {
    pub async fn connect(addr: impl Into<String>) -> ServiceResult<Self> {
        let client = async_nats::connect(addr.into()).await.map_err_unknown()?;
        Ok(Self { client })
    }

    pub async fn get_apub_delivery_stream(&self) -> ServiceResult<ApubQ> {
        let jetstream = jetstream::new(self.client.clone());
        let stream = jetstream
            .get_or_create_stream(jetstream::stream::Config {
                name: APUB_STREAM_NAME.to_owned(),
                retention: stream::RetentionPolicy::WorkQueue,
                subjects: vec![APUB_DELIVERY_SUBJECT.to_owned()],
                storage: stream::StorageType::File,
                ..Default::default()
            })
            .await
            .map_err_unknown()?;

        Ok(ApubQ { jetstream, stream })
    }

    pub async fn queue_activity<A>(
        &self,
        activity: A,
        actor: UserWithApubModel,
        inboxes: Vec<Url>,
    ) -> ServiceResult<()>
    where
        A: Into<SendableActivity> + Clone + std::fmt::Debug,
    {
        self.get_apub_delivery_stream()
            .await?
            .enqueue(activity, actor, inboxes)
            .await
    }

    pub async fn request<S: Serialize, R: DeserializeOwned>(
        &self,
        subject: impl Into<String>,
        req: &S,
    ) -> ServiceResult<R> {
        let payload = serde_json::to_vec(req).map_err_unknown()?;
        let payload = tokio_util::bytes::Bytes::from_owner(payload);
        let response = match self.client.request(subject.into(), payload).await {
            Ok(response) => response,
            Err(e) => match e.kind() {
                async_nats::RequestErrorKind::TimedOut => {
                    return Err(ServiceError::known(NatsError::Timeout));
                }
                async_nats::RequestErrorKind::NoResponders => {
                    return Err(ServiceError::known(NatsError::NoRequestBackend));
                }
                _ => return Err(ServiceError::unknown(e)),
            },
        };
        let response = serde_json::from_slice(&response.payload).map_err(|e| {
            error!("Failed to deserialize response: {:#?}", e);
            ServiceError::known(NatsError::DeserializationFailed)
        })?;
        Ok(response)
    }
}

#[derive(Clone, Debug)]
pub struct ApubQ {
    jetstream: jetstream::Context,
    stream: stream::Stream,
}

#[derive(Clone, Debug)]
pub struct ApubQConsumer {
    consumer: jetstream::consumer::Consumer<jetstream::consumer::pull::Config>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApubDeliveryData {
    activity: SendableActivity,
    actor: UserWithApubModel,
    inbox: Url,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[enum_delegate::implement(ActivityHandler)]
#[serde(untagged)]
pub enum SendableActivityInner {
    Accept(AcceptActivity),
    Announce(AnnounceActivity),
    Create(CreateActivity),
    Delete(DeleteActivity),
    Follow(FollowActivity),
    Like(LikeActivity),
    Reject(RejectActivity),
    Undo(UndoActivity),
    Update(UpdateActivity),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
#[enum_delegate::implement(ActivityHandler)]
pub enum SendableActivity {
    WithContext(WithContext<SendableActivityInner>),
}

impl<A> From<A> for SendableActivity
where
    A: Into<SendableActivityInner>,
{
    fn from(value: A) -> Self {
        SendableActivity::WithContext(WithContext::new_default(value.into()))
    }
}

impl ApubQ {
    pub async fn create_consumer(&self) -> ServiceResult<ApubQConsumer> {
        let consumer = self
            .stream
            .get_or_create_consumer(
                APUB_DELIVERY_CONSUMER,
                jetstream::consumer::pull::Config {
                    max_deliver: delay::APUB_DELIVERY_MAX_ATTEMPTS as i64,
                    durable_name: APUB_DELIVERY_CONSUMER.to_owned().into(),
                    filter_subject: APUB_DELIVERY_SUBJECT.to_owned(),
                    backoff: APUB_DELIVERY_DELAY_TIMES.clone(),
                    ..Default::default()
                },
            )
            .await
            .map_err_unknown()?;

        Ok(ApubQConsumer { consumer })
    }

    pub async fn enqueue<A>(
        &self,
        activity: A,
        actor: UserWithApubModel,
        inboxes: Vec<Url>,
    ) -> ServiceResult<()>
    where
        A: Into<SendableActivity> + Clone + std::fmt::Debug,
    {
        let uniq_inboxes = inboxes.into_iter().unique().collect_vec();
        let activity = activity.into();
        for inbox in uniq_inboxes {
            let data = ApubDeliveryData {
                activity: activity.clone(),
                actor: actor.clone(),
                inbox: inbox.clone(),
            };
            let data = serde_json::to_string(&data).map_err_unknown()?;
            self.jetstream
                .publish(APUB_DELIVERY_SUBJECT, data.into())
                .await
                .map_err_unknown()?
                .await
                .map_err_unknown()?;
            debug!("Published to {APUB_DELIVERY_SUBJECT}: {inbox}");
        }

        Ok(())
    }
}

impl ApubQConsumer {
    pub async fn process_single(
        &self,
        job: ApubDeliveryData,
        data: &Data<MyFederationData>,
    ) -> ServiceResult<()> {
        let tasks =
            SendActivityTask::prepare(&job.activity, &job.actor, vec![job.inbox.clone()], data)
                .await
                .map_err_unknown()?;
        assert!(tasks.len() <= 1, "tasks.len() = {}", tasks.len());
        for task in tasks {
            task.sign_and_send(data).await.map_err_unknown()?;
        }

        Ok(())
    }

    pub async fn process_loop(
        &self,
        data: &Data<MyFederationData>,
        cancel: &CancellationToken,
    ) -> ServiceResult<()> {
        let mut msg_stream = self.consumer.messages().await.map_err_unknown()?;
        loop {
            select! {
                _ = cancel.cancelled() => {
                    info!("APUB_DELIVERY consumer loop cancelled");
                    break;
                }
                Some(Ok(message)) = msg_stream.next() => {
                    debug!("APUB_DELIVERY worker received message");
                    let job = serde_json::from_slice::<ApubDeliveryData>(&message.payload);
                    match job {
                        Ok(job) => match self.process_single(job, data).await {
                            Ok(()) => {
                                debug!("APUB_DELIVERY job processed successfully");
                                message.ack().await.map_err(ServiceError::unknown_box)?;
                            }
                            Err(e) => {
                                error!("APUB_DELIVERY failed to process job: {:#?}", e);
                                let delay = calculate_next_delay(&message);
                                message.ack_with(jetstream::AckKind::Nak(Some(delay))).await.map_err(ServiceError::unknown_box)?;
                            }
                        },
                        Err(e) => {
                            error!("Failed to deserialize APUB_DELIVERY message (discarding job): {:#?}", e);
                            message.ack().await.map_err(ServiceError::unknown_box)?;
                        }
                    }
                }
            }
        }

        info!("APUB_DELIVERY consumer loop ended");
        Ok(())
    }
}

fn calculate_next_delay(msg: &Message) -> Duration {
    let attempts = msg.info().unwrap().delivered;
    if attempts < APUB_DELIVERY_DELAY_TIMES.len() as i64 {
        APUB_DELIVERY_DELAY_TIMES[attempts as usize]
    } else {
        APUB_DELIVERY_DELAY_TIMES.last().copied().unwrap()
    }
}
