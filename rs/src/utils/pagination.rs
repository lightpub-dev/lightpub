use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
pub struct PaginatedResponse<T> {
    pub result: Vec<T>,
    pub next: Option<String>,
}

pub trait PaginatableItem {
    type Key;
    fn pkey(&self) -> Self::Key;
}

#[derive(Debug)]
pub struct PaginatableWrapper<V, K> {
    pub value: V,
    pub key: K,
}

impl<V, K> Serialize for PaginatableWrapper<V, K>
where
    V: Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.value.serialize(serializer)
    }
}

impl<V, K> PaginatableWrapper<V, K> {
    pub fn new(value: V, key: K) -> Self {
        Self { value, key }
    }
}

impl<V, K> PaginatableItem for PaginatableWrapper<V, K>
where
    K: Clone,
{
    type Key = K;
    fn pkey(&self) -> K {
        self.key.clone()
    }
}

impl<T> PaginatedResponse<T>
where
    T: PaginatableItem,
{
    pub fn from_result<G>(result: Vec<T>, page_size: usize, next_gen: G) -> Self
    where
        G: FnOnce(&<T as PaginatableItem>::Key) -> String,
    {
        if result.len() <= page_size {
            return Self { result, next: None };
        }
        let last = result.get(page_size).unwrap();
        let next = Some(next_gen(&last.pkey()));
        let result = result.into_iter().take(page_size).collect();
        Self { result, next }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum CollectionType {
    #[serde(rename = "Collection")]
    Collection,
    #[serde(rename = "OrderedCollection")]
    OrderedCollection,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum CollectionPageType {
    #[serde(rename = "CollectionPage")]
    CollectionPage,
    #[serde(rename = "OrderedCollectionPage")]
    OrderedCollectionPage,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CollectionResponse {
    #[serde(rename = "type")]
    pub collection_type: CollectionType,
    pub total_items: Option<usize>,
    pub first: Option<String>,
}

impl CollectionResponse {
    pub fn from_first(
        collection_type: CollectionType,
        first_link: String,
        total_items: Option<usize>,
    ) -> Self {
        CollectionResponse {
            collection_type,
            total_items,
            first: Some(first_link),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CollectionPageResponse<T> {
    #[serde(rename = "type")]
    pub collection_type: CollectionPageType,
    pub next: Option<String>,
    pub part_of: String,
    pub total_items: Option<usize>,
    pub items: Vec<T>,
}

impl<T> CollectionPageResponse<T> {
    pub fn from_paginated_response(
        collection_type: CollectionPageType,
        paginated_response: PaginatedResponse<T>,
        part_of: String,
        total_items: Option<usize>,
    ) -> Self {
        CollectionPageResponse {
            collection_type,
            next: paginated_response.next,
            part_of,
            total_items,
            items: paginated_response.result,
        }
    }
}
