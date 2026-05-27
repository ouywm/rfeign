use http::HeaderMap;

pub struct ApiResponse<T> {
    pub status: u16,
    pub headers: HeaderMap,
    pub body: T,
}

impl<T> ApiResponse<T> {
    pub fn new(status: u16, headers: HeaderMap, body: T) -> Self {
        Self {
            status,
            headers,
            body,
        }
    }

    pub fn is_success(&self) -> bool {
        (200..300).contains(&self.status)
    }
}