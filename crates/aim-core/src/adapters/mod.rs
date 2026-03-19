pub mod custom_json;
pub mod direct_url;
pub mod github;
pub mod gitlab;
pub mod sourceforge;
pub mod test_support;
pub mod traits;
pub mod zsync;

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
