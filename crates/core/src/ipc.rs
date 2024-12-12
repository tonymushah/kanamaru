pub mod request;

use base64::prelude::*;
use prost::Message;
use serde::{Deserialize, Serialize};
use tauri::http::HeaderMap;

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Default)]
pub struct IpcMessageBase {
    #[serde(with = "self::header_map")]
    pub metadata: HeaderMap,
    pub body: Option<String>,
}

pub(crate) mod header_map {
    use std::{collections::HashMap, str::FromStr};

    use serde::{ser::SerializeMap, Deserialize, Deserializer, Serializer};
    use tauri::http::{HeaderMap, HeaderName, HeaderValue};

    pub fn serialize<S>(metadata: &HeaderMap, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(Some(metadata.len()))?;
        for (k, v) in metadata {
            map.serialize_entry(k.as_str(), v.to_str().map_err(serde::ser::Error::custom)?)?;
        }
        map.end()
    }
    pub fn deserialize<'de, D>(deserializer: D) -> Result<HeaderMap, D::Error>
    where
        D: Deserializer<'de>,
    {
        let map: Option<HashMap<String, String>> = Deserialize::deserialize(deserializer)?;
        let Some(map) = map else {
            return Ok(HeaderMap::new());
        };
        map.into_iter()
            .map(
                |(name, value)| -> Result<(HeaderName, HeaderValue), D::Error> {
                    Ok((
                        HeaderName::from_str(&name).map_err(serde::de::Error::custom)?,
                        HeaderValue::from_str(&value).map_err(serde::de::Error::custom)?,
                    ))
                },
            )
            .collect()
    }
}

impl<M: Message> From<M> for IpcMessageBase {
    fn from(value: M) -> Self {
        let body = Some(BASE64_STANDARD.encode(value.encode_to_vec()));
        IpcMessageBase {
            metadata: HeaderMap::new(),
            body,
        }
    }
}

#[derive(Debug, thiserror::Error)]
#[error(transparent)]
pub enum IpcBodyExtractMessageError {
    Base64(#[from] base64::DecodeError),
    Prost(#[from] prost::DecodeError),
}

impl IpcMessageBase {
    pub fn is_empty(&self) -> bool {
        self.body.is_none()
    }
    pub fn extract_message<M: Message + Default>(&self) -> Result<M, IpcBodyExtractMessageError> {
        if let Some(body) = self.body.as_ref() {
            Ok(M::decode(bytes::Bytes::from(
                BASE64_STANDARD.decode(body)?,
            ))?)
        } else {
            Ok(Default::default())
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct IpcMessage<M> {
    pub metadata: HeaderMap,
    pub body: M,
}

impl<M> IpcMessage<M> {
    pub fn new(body: M) -> Self {
        Self {
            metadata: Default::default(),
            body,
        }
    }
    pub fn new_with_metadata(body: M, metadata: HeaderMap) -> Self {
        Self { metadata, body }
    }
}

impl<M: Message> From<IpcMessage<M>> for IpcMessageBase {
    fn from(value: IpcMessage<M>) -> Self {
        let mut base = Self::from(value.body);
        base.metadata = value.metadata;
        base
    }
}

impl<M> TryFrom<IpcMessageBase> for IpcMessage<M>
where
    M: Message + Default,
{
    type Error = IpcBodyExtractMessageError;
    fn try_from(value: IpcMessageBase) -> Result<Self, Self::Error> {
        let body = value.extract_message::<M>()?;
        Ok(Self::new_with_metadata(body, value.metadata))
    }
}

pub use request::{StreamingRequest, UnaryRequest};
