use crate::domain::update::{
    ArtifactCandidate, ChannelPreference, MetadataHints, UpdateChannel, UpdateChannelKind,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RankedChannel {
    pub channel: UpdateChannel,
    pub reason: String,
    pub score: i32,
}

pub fn rank_channels(channels: &[UpdateChannel]) -> Vec<RankedChannel> {
    let mut ranked = channels
        .iter()
        .cloned()
        .map(|channel| {
            let install_origin_bonus = if channel.matches_install_origin {
                100
            } else {
                0
            };
            let prerelease_penalty = if channel.prerelease { 20 } else { 0 };
            let metadata_bonus = match channel.kind {
                UpdateChannelKind::ElectronBuilder | UpdateChannelKind::Zsync => 25,
                _ => 0,
            };
            let score = channel.confidence as i32 + install_origin_bonus + metadata_bonus
                - prerelease_penalty;
            let reason = if channel.matches_install_origin {
                "install-origin-match"
            } else if metadata_bonus > 0 {
                "metadata-guided"
            } else {
                "heuristic-match"
            };

            RankedChannel {
                channel,
                reason: reason.to_owned(),
                score,
            }
        })
        .collect::<Vec<_>>();

    ranked.sort_by(|left, right| right.score.cmp(&left.score));
    ranked
}

pub fn select_artifact(
    channel: &RankedChannel,
    hints: Option<&MetadataHints>,
) -> ArtifactCandidate {
    let resolved_url = resolve_artifact_url(
        &channel.channel.locator,
        hints.and_then(|value| value.primary_download.as_deref()),
    );
    let selection_reason = if hints
        .and_then(|value| value.primary_download.clone())
        .is_some()
    {
        "metadata-guided"
    } else {
        channel.reason.as_str()
    };
    ArtifactCandidate {
        url: resolved_url,
        version: channel
            .channel
            .version
            .clone()
            .unwrap_or_else(|| "latest".to_owned()),
        arch: Some("x86_64".to_owned()),
        trusted_checksum: hints.and_then(|value| value.checksum.clone()),
        selection_reason: selection_reason.to_owned(),
    }
}

pub fn to_preference(channel: &RankedChannel) -> ChannelPreference {
    ChannelPreference {
        kind: channel.channel.kind,
        locator: channel.channel.locator.clone(),
        reason: channel.reason.clone(),
    }
}

fn resolve_artifact_url(channel_locator: &str, primary_download: Option<&str>) -> String {
    let Some(primary_download) = primary_download else {
        return channel_locator.to_owned();
    };

    if primary_download.contains("://") || primary_download.starts_with("file://") {
        return primary_download.to_owned();
    }

    if (channel_locator.ends_with(".yml") || channel_locator.ends_with(".yaml"))
        && let Some((base, _)) = channel_locator.rsplit_once('/')
    {
        return format!("{base}/{primary_download}");
    }

    primary_download.to_owned()
}
