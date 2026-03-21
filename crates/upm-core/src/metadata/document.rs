#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MetadataDocument {
    pub url: String,
    pub content_type: Option<String>,
    pub contents: Vec<u8>,
}

impl MetadataDocument {
    pub fn plain_text(url: &str, contents: &[u8]) -> Self {
        Self {
            url: url.to_owned(),
            content_type: Some("text/plain".to_owned()),
            contents: contents.to_vec(),
        }
    }

    pub fn yaml(url: &str, contents: &[u8]) -> Self {
        Self {
            url: url.to_owned(),
            content_type: Some("application/yaml".to_owned()),
            contents: contents.to_vec(),
        }
    }
}
