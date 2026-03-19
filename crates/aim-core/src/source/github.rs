use std::env;

use crate::domain::source::{ResolvedRelease, SourceRef};
use crate::metadata::MetadataDocument;

const DEFAULT_GITHUB_API_BASE: &str = "https://api.github.com";
const FIXTURE_MODE_ENV: &str = "AIM_GITHUB_FIXTURE_MODE";

pub trait GitHubTransport {
    fn fetch_releases(&self, repo: &str) -> Result<Vec<TransportRelease>, GitHubDiscoveryError>;

    fn fetch_document(
        &self,
        url: &str,
        content_type: Option<&str>,
    ) -> Result<MetadataDocument, GitHubDiscoveryError>;
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TransportAsset {
    pub name: String,
    pub url: String,
    pub content_type: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TransportRelease {
    pub tag: String,
    pub prerelease: bool,
    pub assets: Vec<TransportAsset>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GitHubAsset {
    pub name: String,
    pub url: String,
    pub version: String,
    pub prerelease: bool,
    pub arch: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GitHubRelease {
    pub tag: String,
    pub release: ResolvedRelease,
    pub assets: Vec<GitHubAsset>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GitHubDiscovery {
    pub source: SourceRef,
    pub releases: Vec<GitHubRelease>,
    pub assets: Vec<GitHubAsset>,
    pub metadata_documents: Vec<MetadataDocument>,
    pub requested_is_older_release: bool,
}

pub fn discover_github_candidates(
    source: &SourceRef,
) -> Result<GitHubDiscovery, GitHubDiscoveryError> {
    let transport = default_transport();
    discover_github_candidates_with(source, transport.as_ref())
}

pub fn discover_github_candidates_with<T: GitHubTransport + ?Sized>(
    source: &SourceRef,
    transport: &T,
) -> Result<GitHubDiscovery, GitHubDiscoveryError> {
    let repo = source
        .canonical_locator
        .clone()
        .unwrap_or_else(|| source.locator.clone());

    let transport_releases = transport.fetch_releases(&repo)?;
    if transport_releases.is_empty() {
        return Err(GitHubDiscoveryError::NoReleases { repo });
    }

    let releases = transport_releases
        .iter()
        .map(|release| GitHubRelease {
            tag: release.tag.clone(),
            release: ResolvedRelease {
                version: release.tag.trim_start_matches('v').to_owned(),
                prerelease: release.prerelease,
            },
            assets: release
                .assets
                .iter()
                .filter(|asset| is_appimage_asset(&asset.name))
                .map(|asset| GitHubAsset {
                    name: asset.name.clone(),
                    url: asset.url.clone(),
                    version: release.tag.trim_start_matches('v').to_owned(),
                    prerelease: release.prerelease,
                    arch: Some(infer_architecture(&asset.name)),
                })
                .collect(),
        })
        .collect::<Vec<_>>();

    let metadata_documents = transport_releases
        .iter()
        .flat_map(|release| release.assets.iter())
        .filter(|asset| is_metadata_document(&asset.name))
        .filter_map(|asset| {
            transport
                .fetch_document(&asset.url, asset.content_type.as_deref())
                .ok()
        })
        .collect::<Vec<_>>();

    let assets = releases
        .iter()
        .flat_map(|release| release.assets.iter().cloned())
        .collect::<Vec<_>>();

    let requested_is_older_release = source
        .requested_tag
        .as_ref()
        .map(|requested| requested != &releases[0].tag)
        .unwrap_or(false);

    Ok(GitHubDiscovery {
        source: source.clone(),
        releases,
        assets,
        metadata_documents,
        requested_is_older_release,
    })
}

pub fn default_transport() -> Box<dyn GitHubTransport> {
    if env::var(FIXTURE_MODE_ENV).ok().as_deref() == Some("1") {
        Box::new(FixtureGitHubTransport)
    } else {
        Box::new(ReqwestGitHubTransport::new())
    }
}

pub struct ReqwestGitHubTransport {
    client: reqwest::blocking::Client,
    api_base: String,
}

impl Default for ReqwestGitHubTransport {
    fn default() -> Self {
        Self::new()
    }
}

impl ReqwestGitHubTransport {
    pub fn new() -> Self {
        let mut default_headers = reqwest::header::HeaderMap::new();
        default_headers.insert(
            reqwest::header::USER_AGENT,
            reqwest::header::HeaderValue::from_static("aim/0.1"),
        );
        default_headers.insert(
            reqwest::header::ACCEPT,
            reqwest::header::HeaderValue::from_static("application/vnd.github+json"),
        );
        if let Some(token) = env::var("AIM_GITHUB_TOKEN")
            .ok()
            .or_else(|| env::var("GITHUB_TOKEN").ok())
            && let Ok(value) = reqwest::header::HeaderValue::from_str(&format!("Bearer {token}"))
        {
            default_headers.insert(reqwest::header::AUTHORIZATION, value);
        }

        Self {
            client: reqwest::blocking::Client::builder()
                .default_headers(default_headers)
                .build()
                .expect("reqwest client should build"),
            api_base: env::var("AIM_GITHUB_API_BASE")
                .unwrap_or_else(|_| DEFAULT_GITHUB_API_BASE.to_owned()),
        }
    }
}

impl GitHubTransport for ReqwestGitHubTransport {
    fn fetch_releases(&self, repo: &str) -> Result<Vec<TransportRelease>, GitHubDiscoveryError> {
        let url = format!("{}/repos/{repo}/releases?per_page=10", self.api_base);
        let releases = self
            .client
            .get(url)
            .send()
            .map_err(GitHubDiscoveryError::Transport)?
            .error_for_status()
            .map_err(GitHubDiscoveryError::Transport)?
            .json::<Vec<ApiRelease>>()
            .map_err(GitHubDiscoveryError::Transport)?;

        Ok(releases
            .into_iter()
            .map(|release| TransportRelease {
                tag: release.tag_name,
                prerelease: release.prerelease,
                assets: release
                    .assets
                    .into_iter()
                    .map(|asset| TransportAsset {
                        name: asset.name,
                        url: asset.browser_download_url,
                        content_type: asset.content_type,
                    })
                    .collect(),
            })
            .collect())
    }

    fn fetch_document(
        &self,
        url: &str,
        content_type: Option<&str>,
    ) -> Result<MetadataDocument, GitHubDiscoveryError> {
        let response = self
            .client
            .get(url)
            .send()
            .map_err(GitHubDiscoveryError::Transport)?
            .error_for_status()
            .map_err(GitHubDiscoveryError::Transport)?;
        let header_content_type = response
            .headers()
            .get(reqwest::header::CONTENT_TYPE)
            .and_then(|value| value.to_str().ok())
            .map(ToOwned::to_owned)
            .or_else(|| content_type.map(ToOwned::to_owned));
        let contents = response.bytes().map_err(GitHubDiscoveryError::Transport)?;

        Ok(MetadataDocument {
            url: url.to_owned(),
            content_type: header_content_type,
            contents: contents.to_vec(),
        })
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct FixtureGitHubTransport;

impl GitHubTransport for FixtureGitHubTransport {
    fn fetch_releases(&self, repo: &str) -> Result<Vec<TransportRelease>, GitHubDiscoveryError> {
        Ok(fixture_releases(repo))
    }

    fn fetch_document(
        &self,
        url: &str,
        content_type: Option<&str>,
    ) -> Result<MetadataDocument, GitHubDiscoveryError> {
        let contents = fixture_document(url)
            .ok_or_else(|| GitHubDiscoveryError::FixtureDocumentMissing(url.to_owned()))?;
        Ok(MetadataDocument {
            url: url.to_owned(),
            content_type: content_type.map(ToOwned::to_owned),
            contents,
        })
    }
}

#[derive(Debug)]
pub enum GitHubDiscoveryError {
    Unsupported,
    FixtureDocumentMissing(String),
    NoReleases { repo: String },
    Transport(reqwest::Error),
}

#[derive(serde::Deserialize)]
struct ApiRelease {
    tag_name: String,
    prerelease: bool,
    assets: Vec<ApiAsset>,
}

#[derive(serde::Deserialize)]
struct ApiAsset {
    name: String,
    browser_download_url: String,
    content_type: Option<String>,
}

fn is_appimage_asset(name: &str) -> bool {
    name.ends_with(".AppImage")
}

fn is_metadata_document(name: &str) -> bool {
    name.ends_with("latest-linux.yml")
        || name.ends_with("latest-linux.yaml")
        || name.ends_with(".zsync")
}

fn infer_architecture(name: &str) -> String {
    if name.contains("aarch64") || name.contains("arm64") {
        "aarch64".to_owned()
    } else {
        "x86_64".to_owned()
    }
}

fn fixture_releases(repo: &str) -> Vec<TransportRelease> {
    match repo {
        "pingdotgg/t3code" => vec![
            fixture_release(repo, "v0.0.12", "T3-Code-0.0.12-x86_64.AppImage"),
            fixture_release(repo, "v0.0.11", "T3-Code-0.0.11-x86_64.AppImage"),
        ],
        "sharkdp/bat" => vec![fixture_release(repo, "v1.0.0", "Bat-1.0.0-x86_64.AppImage")],
        _ => {
            let repo_name = repo.split('/').next_back().unwrap_or("app");
            let title = title_case(repo_name);
            vec![fixture_release(
                repo,
                "v1.0.0",
                &format!("{title}-1.0.0-x86_64.AppImage"),
            )]
        }
    }
}

fn fixture_release(repo: &str, tag: &str, asset_name: &str) -> TransportRelease {
    TransportRelease {
        tag: tag.to_owned(),
        prerelease: false,
        assets: vec![
            TransportAsset {
                name: asset_name.to_owned(),
                url: format!("https://github.com/{repo}/releases/download/{tag}/{asset_name}"),
                content_type: Some("application/octet-stream".to_owned()),
            },
            TransportAsset {
                name: "latest-linux.yml".to_owned(),
                url: format!("https://github.com/{repo}/releases/download/{tag}/latest-linux.yml"),
                content_type: Some("application/yaml".to_owned()),
            },
        ],
    }
}

fn fixture_document(url: &str) -> Option<Vec<u8>> {
    let tag = url.split("/releases/download/").nth(1)?.split('/').next()?;
    let name = url.split('/').next_back()?;
    match name {
        "latest-linux.yml" => {
            let appimage = match tag {
                "v0.0.11" => "T3-Code-0.0.11-x86_64.AppImage",
                "v0.0.12" => "T3-Code-0.0.12-x86_64.AppImage",
                "v1.0.0" => "Bat-1.0.0-x86_64.AppImage",
                _ => return None,
            };
            let version = tag.trim_start_matches('v');
            Some(
                format!("version: {version}\npath: {appimage}\nsha512: fixture-sha\n").into_bytes(),
            )
        }
        _ => None,
    }
}

fn title_case(value: &str) -> String {
    value
        .split(['-', '_'])
        .filter(|segment| !segment.is_empty())
        .map(|segment| {
            let mut chars = segment.chars();
            let Some(first) = chars.next() else {
                return String::new();
            };
            format!("{}{}", first.to_ascii_uppercase(), chars.as_str())
        })
        .collect::<Vec<_>>()
        .join("-")
}
