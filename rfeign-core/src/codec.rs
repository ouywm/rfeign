use bytes::Bytes;
use serde::de::DeserializeOwned;

use crate::error::Result;

pub trait Encoder: Send + Sync + 'static {
    fn encode(&self, value: &dyn erased_serde::Serialize) -> Result<Bytes>;
    fn content_type(&self) -> &str;
}

pub trait Decoder: Send + Sync + 'static {
    fn decode(&self, body: &[u8]) -> Result<serde_json::Value>;
}

pub fn deserialize<T: DeserializeOwned>(value: serde_json::Value) -> Result<T> {
    serde_json::from_value(value).map_err(Into::into)
}

pub struct JsonCodec;

impl Encoder for JsonCodec {
    fn encode(&self, value: &dyn erased_serde::Serialize) -> Result<Bytes> {
        let bytes = serde_json::to_vec(value)?;
        Ok(Bytes::from(bytes))
    }

    fn content_type(&self) -> &str {
        "application/json"
    }
}

impl Decoder for JsonCodec {
    fn decode(&self, body: &[u8]) -> Result<serde_json::Value> {
        serde_json::from_slice(body).map_err(Into::into)
    }
}