use http::HeaderMap;

use crate::error::Error;

pub trait ErrorDecoder: Send + Sync + 'static {
    fn decode(&self, status: u16, headers: &HeaderMap, body: &[u8]) -> Error;
}

pub struct DefaultErrorDecoder;

impl ErrorDecoder for DefaultErrorDecoder {
    fn decode(&self, status: u16, _headers: &HeaderMap, body: &[u8]) -> Error {
        Error::Status {
            status,
            body: String::from_utf8_lossy(body).into_owned(),
        }
    }
}