use std::{future::Future, marker::PhantomData};

use activitypub_federation::kinds::collection::{OrderedCollectionPageType, OrderedCollectionType};
use derive_builder::Builder;
use serde::{Deserialize, Serialize};
use url::Url;

use super::{decode_cursor, encode_cursor, PaginationError};
use lightpub_service::services::{ServiceError, ServiceResult};
use serde::de::DeserializeOwned;

#[derive(Debug, Clone, Serialize, Deserialize, Builder)]
#[serde(rename_all = "camelCase")]
pub struct OrderedCollection {
    #[serde(rename = "type")]
    #[builder(default = "OrderedCollectionType::OrderedCollection")]
    pub kind: OrderedCollectionType,
    pub total_items: u64,
    pub first: Url,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default)]
    pub last: Option<Url>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Builder)]
#[serde(rename_all = "camelCase")]
pub struct OrderedCollectionPage<T> {
    #[serde(rename = "type")]
    #[builder(default = "OrderedCollectionPageType::OrderedCollectionPage")]
    pub kind: OrderedCollectionPageType,
    pub part_of: Url,
    #[serde(rename = "orderedItems")]
    pub items: Vec<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default)]
    pub next: Option<Url>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default)]
    pub prev: Option<Url>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ApubPaginatorResponse<T> {
    Collection(OrderedCollection),
    CollectionPage(OrderedCollectionPage<T>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApubPaginationKeyBase<T> {
    pub page: bool,
    #[serde(flatten)]
    pub key: Option<T>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApubCollectionQuery {
    pub key: Option<String>,
}

pub struct ApubPaginator<F, C, CF, K, Fut, CFFut, T>
where
    F: Fn(usize, Option<K>) -> Fut,
    C: Fn(T) -> K,
    CF: Fn() -> CFFut,
    Fut: Future<Output = ServiceResult<Vec<T>>>,
    CFFut: Future<Output = ServiceResult<usize>>,
    K: Serialize + DeserializeOwned + Clone,
{
    endpoint: Url,
    page_size: usize,
    fetch_fn: F,
    key_fn: C,
    count_fn: CF,
    _key_type: PhantomData<K>,
    _item_type: PhantomData<T>,
}

impl<F, C, CF, K, Fut, CFFut, T> ApubPaginator<F, C, CF, K, Fut, CFFut, T>
where
    F: Fn(usize, Option<K>) -> Fut,
    C: Fn(T) -> K,
    CF: Fn() -> CFFut,
    Fut: Future<Output = ServiceResult<Vec<T>>>,
    CFFut: Future<Output = ServiceResult<usize>>,
    K: Serialize + DeserializeOwned + Clone,
{
    pub fn new(endpoint: Url, page_size: usize, fetch_fn: F, key_fn: C, count_fn: CF) -> Self {
        Self {
            endpoint,
            page_size,
            fetch_fn,
            key_fn,
            count_fn,
            _key_type: PhantomData,
            _item_type: PhantomData,
        }
    }

    async fn build_collection(&self) -> ServiceResult<ApubPaginatorResponse<T>> {
        let total_items = (self.count_fn)().await?;
        let first_url = self.build_url(&ApubCollectionQuery {
            key: Some(encode_cursor(&ApubPaginationKeyBase::<K> {
                page: true,
                key: None,
            })?),
        });

        Ok(ApubPaginatorResponse::Collection(OrderedCollection {
            total_items: total_items as u64,
            kind: OrderedCollectionType::OrderedCollection,
            first: first_url,
            last: None,
        }))
    }

    pub async fn response(
        &self,
        query: &ApubCollectionQuery,
    ) -> ServiceResult<ApubPaginatorResponse<T>> {
        let parsed_key = match query.key.as_ref() {
            None => {
                // No key provided, return collection
                return self.build_collection().await;
            }
            Some(key_str) => decode_cursor::<ApubPaginationKeyBase<K>>(key_str)?,
        };

        // Now handle the parsed key cases
        match (parsed_key.page, parsed_key.key) {
            (true, key) => {
                // Handle both page=true, key=None and page=true, key=Some cases
                let page = self.fetch_page(key).await?;
                Ok(ApubPaginatorResponse::CollectionPage(page))
            }
            (false, Some(_)) => {
                // Invalid case: page=false, key=Some
                Err(ServiceError::known(PaginationError::BadQuery))
            }
            (false, None) => {
                // page=false, key=None - return the collection
                self.build_collection().await
            }
        }
    }

    async fn fetch_page(&self, key: Option<K>) -> ServiceResult<OrderedCollectionPage<T>> {
        // Fetch page_size + 1 to determine if there's a next page
        let fetch_size = self.page_size + 1;
        let mut items = (self.fetch_fn)(fetch_size, key.clone()).await?;

        let next_key = if items.len() > self.page_size {
            // Remove the extra item we fetched
            items
                .pop()
                .map(|last_item| {
                    let key = (self.key_fn)(last_item);
                    encode_cursor(&ApubPaginationKeyBase {
                        page: true,
                        key: Some(key),
                    })
                })
                .transpose()?
        } else {
            None
        };

        let next_url = next_key.map(|key| self.build_url(&ApubCollectionQuery { key: Some(key) }));

        Ok(OrderedCollectionPage {
            kind: OrderedCollectionPageType::OrderedCollectionPage,
            items,
            part_of: self.endpoint.clone(),
            next: next_url,
            prev: None,
        })
    }

    fn build_url(&self, query: &ApubCollectionQuery) -> Url {
        if query.key.is_none() {
            self.endpoint.clone()
        } else {
            let mut url = self.endpoint.clone();
            let query_str = serde_qs::to_string(&query).unwrap();
            url.set_query(Some(&query_str));
            url
        }
    }
}
