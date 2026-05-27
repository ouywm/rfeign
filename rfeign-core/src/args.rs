use bytes::Bytes;

pub trait ArgsProvider {
    fn path_params(&self) -> Vec<(&str, String)> {
        vec![]
    }

    fn query_pairs(&self) -> Vec<(String, String)> {
        vec![]
    }

    fn headers(&self) -> Vec<(&str, String)> {
        vec![]
    }

    fn body_bytes(&self) -> Option<Bytes> {
        None
    }
}
