use bytes::Bytes;
use futures_core::Stream;
use std::pin::Pin;

use crate::error::Error;

pub type ByteStream = Pin<Box<dyn Stream<Item = std::result::Result<Bytes, Error>> + Send>>;