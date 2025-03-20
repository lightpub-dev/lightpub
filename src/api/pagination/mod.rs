/*
Lightpub: a simple ActivityPub server
Copyright (C) 2025 tinaxd

This program is free software: you can redistribute it and/or modify
it under the terms of the GNU Affero General Public License as published
by the Free Software Foundation, either version 3 of the License, or
(at your option) any later version.

This program is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
GNU Affero General Public License for more details.

You should have received a copy of the GNU Affero General Public License
along with this program.  If not, see <https://www.gnu.org/licenses/>.
*/

use base64::{engine::general_purpose::URL_SAFE as base64_engine, Engine};
use expected_error::StatusCode;
use expected_error_derive::ExpectedError;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::{future::Future, marker::PhantomData};
use thiserror::Error;

use lightpub_service::services::{MapToUnknown, ServiceError, ServiceResult};

mod apub;

pub use apub::{
    ApubCollectionQuery, ApubPaginator, ApubPaginatorResponse, OrderedCollection,
    OrderedCollectionPage,
};

#[derive(Debug, Error, Clone, ExpectedError)]
pub enum PaginationError {
    #[error("invalid next_key")]
    #[ee(status(StatusCode::BAD_REQUEST))]
    BadKey,
    #[error("invalid query")]
    #[ee(status(StatusCode::BAD_REQUEST))]
    BadQuery,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PaginatedResponse<T, K = String> {
    pub data: Vec<T>,
    pub next_key: Option<K>,
}

pub struct Paginator<F, C, K, Fut, T>
where
    F: Fn(usize, Option<K>) -> Fut,
    C: Fn(T) -> K,
    Fut: Future<Output = ServiceResult<Vec<T>>>,
    K: Serialize + DeserializeOwned,
{
    page_size: usize,
    fetch_fn: F,
    key_fn: C,
    _key_type: PhantomData<K>,
    _item_type: PhantomData<T>,
}

impl<F, C, K, Fut, T> Paginator<F, C, K, Fut, T>
where
    F: Fn(usize, Option<K>) -> Fut,
    C: Fn(T) -> K,
    Fut: Future<Output = ServiceResult<Vec<T>>>,
    K: Serialize + DeserializeOwned,
{
    pub fn new(page_size: usize, fetch_fn: F, key_fn: C) -> Self {
        Self {
            page_size,
            fetch_fn,
            key_fn,
            _key_type: PhantomData,
            _item_type: PhantomData,
        }
    }

    // Move type parameters that are only used in the method to the method
    pub async fn fetch_page(
        &self,
        cursor: Option<String>,
    ) -> ServiceResult<PaginatedResponse<T, String>> {
        // Decode cursor if provided
        let decoded_cursor = match cursor {
            Some(encoded) => Some(decode_cursor(&encoded)?),
            None => None,
        };

        // Fetch page_size + 1 to determine if there's a next page
        let fetch_size = self.page_size + 1;
        let mut items = (self.fetch_fn)(fetch_size, decoded_cursor).await?;

        let next_key = if items.len() > self.page_size {
            // Remove the extra item we fetched
            items
                .pop()
                .map(|last_item| {
                    let cursor = (self.key_fn)(last_item);
                    encode_cursor(&cursor)
                })
                .transpose()?
        } else {
            None
        };

        Ok(PaginatedResponse {
            data: items,
            next_key,
        })
    }
}

fn encode_cursor<T: Serialize>(cursor: &T) -> ServiceResult<String> {
    let json = serde_json::to_string(cursor).map_err_unknown()?;
    Ok(base64_engine.encode(json))
}

fn decode_cursor<T: DeserializeOwned>(encoded: &str) -> ServiceResult<T> {
    let json_bytes = base64_engine
        .decode(encoded)
        .map_err(|_| ServiceError::known(PaginationError::BadKey))?;
    let json_str =
        String::from_utf8(json_bytes).map_err(|_| ServiceError::known(PaginationError::BadKey))?;
    serde_json::from_str(&json_str).map_err(|_| ServiceError::known(PaginationError::BadKey))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct URLBasedPaginatedResponse<T> {
    pub data: Vec<T>,
    pub next_url: Option<String>,
}

impl<T> URLBasedPaginatedResponse<T> {
    pub fn from_paginated<K>(p: PaginatedResponse<T, K>, f: impl FnOnce(K) -> String) -> Self {
        let next_url = p.next_key.map(f);
        Self {
            data: p.data,
            next_url,
        }
    }

    pub fn map_data<R>(self, f: impl Fn(T) -> R) -> URLBasedPaginatedResponse<R> {
        URLBasedPaginatedResponse {
            data: self.data.into_iter().map(f).collect(),
            next_url: self.next_url,
        }
    }
}
