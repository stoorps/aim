use crate::domain::source::{SourceInputKind, SourceRef};
use crate::domain::update::{ParsedMetadata, UpdateChannel, UpdateChannelKind};
use crate::source::github::GitHubDiscovery;

pub fn build_channels(
    discovery: &GitHubDiscovery,
    metadata: &[ParsedMetadata],
) -> Vec<UpdateChannel> {
    let mut channels = Vec::new();

    if let Some(asset) = discovery.assets.first() {
        channels.push(UpdateChannel {
            kind: UpdateChannelKind::GitHubReleases,
            locator: discovery
                .source
                .canonical_locator
                .clone()
                .unwrap_or_else(|| discovery.source.locator.clone()),
            version: Some(asset.version.clone()),
            artifact_name: Some(asset.name.clone()),
            confidence: 60,
            matches_install_origin: matches!(
                discovery.source.input_kind,
                SourceInputKind::RepoShorthand
                    | SourceInputKind::GitHubRepositoryUrl
                    | SourceInputKind::GitHubReleaseUrl
            ),
            prerelease: asset.prerelease,
        });
    }

    if let Some(parsed) = metadata
        .iter()
        .find(|item| item.kind == crate::domain::update::ParsedMetadataKind::ElectronBuilder)
    {
        channels.push(UpdateChannel {
            kind: UpdateChannelKind::ElectronBuilder,
            locator: discovery
                .metadata_documents
                .iter()
                .find(|doc| doc.url.ends_with("latest-linux.yml"))
                .map(|doc| doc.url.clone())
                .unwrap_or_else(|| discovery.source.locator.clone()),
            version: parsed.hints.version.clone(),
            artifact_name: parsed.hints.primary_download.clone(),
            confidence: parsed.confidence,
            matches_install_origin: discovery.source.tracks_latest,
            prerelease: false,
        });
    }

    if let Some(parsed) = metadata
        .iter()
        .find(|item| item.kind == crate::domain::update::ParsedMetadataKind::Zsync)
    {
        channels.push(UpdateChannel {
            kind: UpdateChannelKind::Zsync,
            locator: parsed.hints.primary_download.clone().unwrap_or_default(),
            version: parsed.hints.version.clone(),
            artifact_name: parsed.hints.primary_download.clone(),
            confidence: parsed.confidence,
            matches_install_origin: false,
            prerelease: false,
        });
    }

    if matches!(
        discovery.source.input_kind,
        SourceInputKind::GitHubReleaseAssetUrl
    ) {
        channels.push(UpdateChannel {
            kind: UpdateChannelKind::DirectAsset,
            locator: discovery.source.locator.clone(),
            version: discovery
                .source
                .requested_tag
                .clone()
                .map(|value| value.trim_start_matches('v').to_owned()),
            artifact_name: discovery.source.requested_asset_name.clone(),
            confidence: 85,
            matches_install_origin: true,
            prerelease: false,
        });
    }

    channels
}

pub fn source_ref_from_channel(source: &SourceRef, channel: &UpdateChannel) -> SourceRef {
    let mut value = source.clone();
    value.locator = channel.locator.clone();
    value
}
