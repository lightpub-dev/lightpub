use derive_builder::Builder;

use super::ApubRequestService;

#[derive(Debug, Builder)]
pub struct ApubReqwestConfig {
    timeout: std::time::Duration,
}

#[derive(Debug, Clone)]
pub struct ApubReqwester {
    client: reqwest::Client,
}

#[derive(Debug)]
pub struct ApubReqwest {
    client: ApubReqwester,
    config: ApubReqwestConfig,
}

impl ApubReqwest {
    pub fn new(client: ApubReqwester, config: ApubReqwestConfig) -> Self {
        Self { client, config }
    }

    fn client(&self) -> reqwest::Client {
        self.client.client.clone()
    }
}

impl ApubRequestService for ApubReqwest {
    async fn post_to_inbox(
        &mut self,
        url: impl Into<reqwest::Url>,
        activity: &crate::models::ApubActivity,
        actor: impl Into<crate::models::ApubActor>,
    ) -> Result<(), super::ServiceError<super::PostToInboxError>> {
        let body = activity.to_json();

        let client = self.client();
        let actor = actor.into();

        // send to the inbox
        let res = client
            .post(url)
            .header("Content-Type", "application/activity+json")
            .header("Accept", "application/activity+json")
            .body(body)
            .send()
            .await
            .map_err(super::ServiceError::from)?;

        todo!()
    }

    async fn fetch_user(
        &mut self,
        url: impl Into<reqwest::Url>,
    ) -> Result<crate::models::ApubPerson, super::ServiceError<super::ApubFetchUserError>> {
        todo!()
    }

    async fn fetch_webfinger(
        &mut self,
        username: &str,
        host: &str,
    ) -> Result<crate::models::ApubWebfingerResponse, super::ServiceError<super::WebfingerError>>
    {
        todo!()
    }
}
