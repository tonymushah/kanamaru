pub mod request;

use base64::prelude::*;
use prost::Message;
use serde::{Deserialize, Serialize};
use tauri::http::HeaderMap;

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Default)]
pub struct IpcBody {
    #[serde(with = "self::header_map")]
    pub metadata: HeaderMap,
    pub body: Option<String>,
}

mod header_map {
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

impl<M: Message> From<M> for IpcBody {
    fn from(value: M) -> Self {
        let body = Some(BASE64_STANDARD.encode(value.encode_to_vec()));
        IpcBody {
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

impl IpcBody {
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
