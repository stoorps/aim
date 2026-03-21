pub mod custom_json;
pub mod direct_url;
pub mod github;
pub mod gitlab;
pub mod sourceforge;
pub mod test_support;
pub mod traits;
pub mod zsync;

use crate::adapters::traits::SourceAdapter;
use crate::domain::source::SourceRef;

pub fn all_adapter_kinds() -> Vec<&'static str> {
    vec![
        "github",
        "gitlab",
        "direct-url",
        "zsync",
        "sourceforge",
        "custom-json",
    ]
}

pub fn supports_source<A: SourceAdapter + ?Sized>(adapter: &A, source: &SourceRef) -> bool {
    adapter.repository_source_kind() == Some(source.kind)
        || adapter.exact_source_kind() == Some(source.kind)
}
