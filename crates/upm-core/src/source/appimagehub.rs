use std::env;
use std::time::Duration;

use crate::domain::source::SourceRef;

const DEFAULT_APPIMAGEHUB_API_BASE: &str = "https://api.appimagehub.com/ocs/v1/content";
const GLOBAL_FIXTURE_MODE_ENV: &str = "UPM_FIXTURE_MODE";
const FIXTURE_MODE_ENV: &str = "UPM_APPIMAGEHUB_FIXTURE_MODE";

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AppImageHubDownload {
    pub url: String,
    pub name: String,
    pub package_type: Option<String>,
    pub arch: Option<String>,
    pub md5sum: Option<String>,
    pub version: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AppImageHubItem {
    pub id: String,
    pub name: String,
    pub version: String,
    pub summary: Option<String>,
    pub detail_page: String,
    pub tags: Vec<String>,
    pub downloads: Vec<AppImageHubDownload>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AppImageHubSearchHit {
    pub id: String,
    pub name: String,
    pub version: String,
    pub summary: Option<String>,
    pub detail_page: String,
    pub tags: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResolvedAppImageHubItem {
    pub source: SourceRef,
    pub title: String,
    pub version: String,
    pub download: AppImageHubDownload,
}

pub trait AppImageHubTransport {
    fn fetch_item(&self, id: &str) -> Result<AppImageHubItem, AppImageHubError>;

    fn search_items(
        &self,
        query: &str,
        limit: usize,
    ) -> Result<Vec<AppImageHubSearchHit>, AppImageHubSearchError>;
}

pub fn default_transport() -> Box<dyn AppImageHubTransport> {
    if env::var(GLOBAL_FIXTURE_MODE_ENV).ok().as_deref() == Some("1")
        || env::var(FIXTURE_MODE_ENV).ok().as_deref() == Some("1") {
        Box::new(FixtureAppImageHubTransport)
    } else {
        Box::new(ReqwestAppImageHubTransport::new())
    }
}

pub fn resolve_appimagehub_item(
    source: &SourceRef,
) -> Result<Option<ResolvedAppImageHubItem>, AppImageHubError> {
    let transport = default_transport();
    resolve_appimagehub_item_with(source, transport.as_ref())
}

pub fn resolve_appimagehub_item_with<T: AppImageHubTransport + ?Sized>(
    source: &SourceRef,
    transport: &T,
) -> Result<Option<ResolvedAppImageHubItem>, AppImageHubError> {
    let item = transport.fetch_item(source_id(source)?)?;
    let Some(download) = item
        .downloads
        .iter()
        .find(|download| is_appimage_download(download))
    else {
        return Ok(None);
    };

    validate_download_url(&download.url)?;

    Ok(Some(ResolvedAppImageHubItem {
        source: source.clone(),
        title: item.name.clone(),
        version: resolved_version(&item, download),
        download: download.clone(),
    }))
}

pub fn search_appimagehub(
    query: &str,
    limit: usize,
) -> Result<Vec<AppImageHubSearchHit>, AppImageHubSearchError> {
    let transport = default_transport();
    search_appimagehub_with(query, limit, transport.as_ref())
}

pub fn search_appimagehub_with<T: AppImageHubTransport + ?Sized>(
    query: &str,
    limit: usize,
    transport: &T,
) -> Result<Vec<AppImageHubSearchHit>, AppImageHubSearchError> {
    transport.search_items(query, limit)
}

pub struct ReqwestAppImageHubTransport {
    client: reqwest::blocking::Client,
    api_base: String,
}

impl Default for ReqwestAppImageHubTransport {
    fn default() -> Self {
        Self::new()
    }
}

impl ReqwestAppImageHubTransport {
    pub fn new() -> Self {
        Self {
            client: reqwest::blocking::Client::builder()
                .timeout(Duration::from_secs(30))
                .build()
                .expect("reqwest client should build"),
            api_base: env::var("UPM_APPIMAGEHUB_API_BASE")
                .unwrap_or_else(|_| DEFAULT_APPIMAGEHUB_API_BASE.to_owned()),
        }
    }
}

impl AppImageHubTransport for ReqwestAppImageHubTransport {
    fn fetch_item(&self, id: &str) -> Result<AppImageHubItem, AppImageHubError> {
        let url = format!("{}/data/{id}", self.api_base);
        let xml = self
            .client
            .get(url)
            .send()
            .map_err(AppImageHubError::Transport)?
            .error_for_status()
            .map_err(AppImageHubError::Transport)?
            .text()
            .map_err(AppImageHubError::Transport)?;

        parse_item_xml(&xml)
    }

    fn search_items(
        &self,
        query: &str,
        limit: usize,
    ) -> Result<Vec<AppImageHubSearchHit>, AppImageHubSearchError> {
        let url = format!("{}/data", self.api_base);
        let xml = self
            .client
            .get(url)
            .query(&[("search", query), ("pagesize", &limit.to_string())])
            .send()
            .map_err(AppImageHubSearchError::Transport)?
            .error_for_status()
            .map_err(AppImageHubSearchError::Transport)?
            .text()
            .map_err(AppImageHubSearchError::Transport)?;

        parse_search_xml(&xml)
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct FixtureAppImageHubTransport;

impl AppImageHubTransport for FixtureAppImageHubTransport {
    fn fetch_item(&self, id: &str) -> Result<AppImageHubItem, AppImageHubError> {
        fixture_item(id).ok_or_else(|| AppImageHubError::FixtureItemMissing(id.to_owned()))
    }

    fn search_items(
        &self,
        query: &str,
        limit: usize,
    ) -> Result<Vec<AppImageHubSearchHit>, AppImageHubSearchError> {
        Ok(fixture_search_results(query, limit))
    }
}

#[derive(Debug)]
pub enum AppImageHubError {
    FixtureItemMissing(String),
    InsecureDownloadUrl(String),
    Parse(quick_xml::DeError),
    Transport(reqwest::Error),
    UnsupportedSource(String),
}

#[derive(Debug)]
pub enum AppImageHubSearchError {
    Parse(quick_xml::DeError),
    Transport(reqwest::Error),
}

#[derive(serde::Deserialize)]
struct OcsSingleResponse {
    data: OcsSingleData,
}

#[derive(serde::Deserialize)]
struct OcsSingleData {
    content: OcsContent,
}

#[derive(serde::Deserialize)]
struct OcsSearchResponse {
    data: OcsSearchData,
}

#[derive(serde::Deserialize)]
struct OcsSearchData {
    #[serde(default)]
    content: Vec<OcsContent>,
}

#[derive(serde::Deserialize)]
struct OcsContent {
    id: String,
    name: String,
    version: Option<String>,
    summary: Option<String>,
    detailpage: Option<String>,
    tags: Option<String>,
    downloadlink1: Option<String>,
    downloadname1: Option<String>,
    download_package_type1: Option<String>,
    download_package_arch1: Option<String>,
    downloadmd5sum1: Option<String>,
    download_version1: Option<String>,
    downloadlink2: Option<String>,
    downloadname2: Option<String>,
    download_package_type2: Option<String>,
    download_package_arch2: Option<String>,
    downloadmd5sum2: Option<String>,
    download_version2: Option<String>,
    downloadlink3: Option<String>,
    downloadname3: Option<String>,
    download_package_type3: Option<String>,
    download_package_arch3: Option<String>,
    downloadmd5sum3: Option<String>,
    download_version3: Option<String>,
}

fn parse_item_xml(xml: &str) -> Result<AppImageHubItem, AppImageHubError> {
    let parsed =
        quick_xml::de::from_str::<OcsSingleResponse>(xml).map_err(AppImageHubError::Parse)?;
    Ok(content_to_item(parsed.data.content))
}

fn parse_search_xml(xml: &str) -> Result<Vec<AppImageHubSearchHit>, AppImageHubSearchError> {
    if !xml.contains("<id>") {
        return Ok(Vec::new());
    }

    let parsed =
        quick_xml::de::from_str::<OcsSearchResponse>(xml).map_err(AppImageHubSearchError::Parse)?;
    Ok(parsed
        .data
        .content
        .into_iter()
        .map(|content| AppImageHubSearchHit {
            id: content.id,
            name: content.name,
            version: normalize_version_text(content.version.as_deref()),
            summary: content.summary,
            detail_page: content
                .detailpage
                .unwrap_or_else(|| "https://www.appimagehub.com".to_owned()),
            tags: split_tags(content.tags.as_deref()),
        })
        .collect())
}

fn content_to_item(content: OcsContent) -> AppImageHubItem {
    let detail_page = content
        .detailpage
        .clone()
        .unwrap_or_else(|| "https://www.appimagehub.com".to_owned());
    let summary = content.summary.clone();
    let tags = split_tags(content.tags.as_deref());
    let downloads = collect_downloads(&content);

    AppImageHubItem {
        id: content.id,
        name: content.name,
        version: normalize_version_text(content.version.as_deref()),
        summary,
        detail_page,
        tags,
        downloads,
    }
}

fn validate_download_url(url: &str) -> Result<(), AppImageHubError> {
    if !url.starts_with("https://") {
        return Err(AppImageHubError::InsecureDownloadUrl(url.to_owned()));
    }

    Ok(())
}

fn collect_downloads(content: &OcsContent) -> Vec<AppImageHubDownload> {
    let mut downloads = Vec::new();

    for download in [
        download_slot(
            content.downloadlink1.as_deref(),
            content.downloadname1.as_deref(),
            content.download_package_type1.as_deref(),
            content.download_package_arch1.as_deref(),
            content.downloadmd5sum1.as_deref(),
            content.download_version1.as_deref(),
        ),
        download_slot(
            content.downloadlink2.as_deref(),
            content.downloadname2.as_deref(),
            content.download_package_type2.as_deref(),
            content.download_package_arch2.as_deref(),
            content.downloadmd5sum2.as_deref(),
            content.download_version2.as_deref(),
        ),
        download_slot(
            content.downloadlink3.as_deref(),
            content.downloadname3.as_deref(),
            content.download_package_type3.as_deref(),
            content.download_package_arch3.as_deref(),
            content.downloadmd5sum3.as_deref(),
            content.download_version3.as_deref(),
        ),
    ]
    .into_iter()
    .flatten()
    {
        downloads.push(download);
    }

    downloads
}

fn download_slot(
    link: Option<&str>,
    name: Option<&str>,
    package_type: Option<&str>,
    arch: Option<&str>,
    md5sum: Option<&str>,
    version: Option<&str>,
) -> Option<AppImageHubDownload> {
    let url = link?.trim();
    if url.is_empty() {
        return None;
    }

    Some(AppImageHubDownload {
        url: url.to_owned(),
        name: name.unwrap_or("download").trim().to_owned(),
        package_type: trim_optional(package_type),
        arch: trim_optional(arch),
        md5sum: trim_optional(md5sum),
        version: trim_optional(version),
    })
}

fn trim_optional(value: Option<&str>) -> Option<String> {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
}

fn normalize_version_text(value: Option<&str>) -> String {
    let value = value.map(str::trim).filter(|value| !value.is_empty());
    match value {
        Some("Latest") | Some("latest") | None => "latest".to_owned(),
        Some(other) => other.to_owned(),
    }
}

fn split_tags(tags: Option<&str>) -> Vec<String> {
    tags.unwrap_or_default()
        .split(',')
        .map(str::trim)
        .filter(|tag| !tag.is_empty())
        .map(ToOwned::to_owned)
        .collect()
}

fn source_id(source: &SourceRef) -> Result<&str, AppImageHubError> {
    source
        .canonical_locator
        .as_deref()
        .or_else(|| source.locator.rsplit('/').next())
        .filter(|value| !value.is_empty())
        .ok_or_else(|| AppImageHubError::UnsupportedSource(source.locator.clone()))
}

fn is_appimage_download(download: &AppImageHubDownload) -> bool {
    download
        .package_type
        .as_deref()
        .map(|kind| kind.eq_ignore_ascii_case("appimage"))
        .unwrap_or(false)
        || download.name.ends_with(".AppImage")
}

fn resolved_version(item: &AppImageHubItem, download: &AppImageHubDownload) -> String {
    download
        .version
        .as_deref()
        .map(|value| normalize_version_text(Some(value)))
        .filter(|value| value != "latest")
        .unwrap_or_else(|| item.version.clone())
}

fn fixture_item(id: &str) -> Option<AppImageHubItem> {
    let insecure_http = env::var("UPM_APPIMAGEHUB_FIXTURE_INSECURE_HTTP")
        .ok()
        .as_deref()
        == Some("1");
    let bad_md5 = env::var("UPM_APPIMAGEHUB_FIXTURE_BAD_MD5").ok().as_deref() == Some("1");

    match id {
        "2338455" => Some(AppImageHubItem {
            id: "2338455".to_owned(),
            name: "Firefox by Mozilla - Official AppImage Edition".to_owned(),
            version: "latest".to_owned(),
            summary: Some("Take control of your internet with the Firefox browser".to_owned()),
            detail_page: "https://www.appimagehub.com/p/2338455".to_owned(),
            tags: vec![
                "appimage".to_owned(),
                "x86-64".to_owned(),
                "desktop".to_owned(),
                "release-stable".to_owned(),
            ],
            downloads: vec![AppImageHubDownload {
                url: if insecure_http {
                    "http://files06.pling.com/api/files/download/firefox-x86-64.AppImage".to_owned()
                } else {
                    "https://files06.pling.com/api/files/download/firefox-x86-64.AppImage"
                        .to_owned()
                },
                name: "firefox-x86-64.AppImage".to_owned(),
                package_type: Some("appimage".to_owned()),
                arch: Some("x86-64".to_owned()),
                md5sum: Some(if bad_md5 {
                    "00000000000000000000000000000000".to_owned()
                } else {
                    "2a685cf45213d5a2a243273fa68dafa6".to_owned()
                }),
                version: None,
            }],
        }),
        "2337998" => Some(AppImageHubItem {
            id: "2337998".to_owned(),
            name: "Example Non-AppImage Package".to_owned(),
            version: "latest".to_owned(),
            summary: Some("An item that does not expose an AppImage download".to_owned()),
            detail_page: "https://www.appimagehub.com/p/2337998".to_owned(),
            tags: vec!["desktop".to_owned()],
            downloads: vec![AppImageHubDownload {
                url: "https://files06.pling.com/api/files/download/example.deb".to_owned(),
                name: "example.deb".to_owned(),
                package_type: Some("debian-package".to_owned()),
                arch: Some("x86-64".to_owned()),
                md5sum: None,
                version: Some("2.1.1".to_owned()),
            }],
        }),
        _ => None,
    }
}

fn fixture_search_results(query: &str, limit: usize) -> Vec<AppImageHubSearchHit> {
    let query = query.trim().to_ascii_lowercase();
    let fixtures = [
        AppImageHubSearchHit {
            id: "2338455".to_owned(),
            name: "Firefox by Mozilla - Official AppImage Edition".to_owned(),
            version: "latest".to_owned(),
            summary: Some("Take control of your internet with the Firefox browser".to_owned()),
            detail_page: "https://www.appimagehub.com/p/2338455".to_owned(),
            tags: vec!["browser".to_owned(), "appimage".to_owned()],
        },
        AppImageHubSearchHit {
            id: "2338484".to_owned(),
            name: "Waterfox".to_owned(),
            version: "latest".to_owned(),
            summary: Some("Open Source, Private Browsing".to_owned()),
            detail_page: "https://www.appimagehub.com/p/2338484".to_owned(),
            tags: vec!["browser".to_owned(), "appimage".to_owned()],
        },
    ];

    fixtures
        .into_iter()
        .filter(|item| {
            item.name.to_ascii_lowercase().contains(&query)
                || item
                    .tags
                    .iter()
                    .any(|tag| tag.to_ascii_lowercase().contains(&query))
        })
        .take(limit)
        .collect()
}
