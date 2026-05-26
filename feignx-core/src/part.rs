use bytes::Bytes;

pub struct Part {
    pub filename: String,
    pub content_type: String,
    pub data: Bytes,
}

impl Part {
    pub fn from_bytes(
        filename: impl Into<String>,
        content_type: impl Into<String>,
        data: Vec<u8>,
    ) -> Self {
        Self {
            filename: filename.into(),
            content_type: content_type.into(),
            data: Bytes::from(data),
        }
    }
}