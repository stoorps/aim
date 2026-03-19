use crate::domain::update::{MetadataHints, ParsedMetadata, ParsedMetadataKind};
use crate::metadata::document::MetadataDocument;

pub fn parse(document: &MetadataDocument) -> ParsedMetadata {
    let contents = String::from_utf8_lossy(&document.contents);
    let url = extract_field(&contents, "URL:");
    let filename = extract_field(&contents, "Filename:");

    ParsedMetadata {
        kind: ParsedMetadataKind::Zsync,
        hints: MetadataHints {
            version: filename
                .as_ref()
                .and_then(|value| version_from_filename(value)),
            primary_download: url.or(filename),
            checksum: None,
            architecture: Some("x86_64".to_owned()),
            channel_label: Some("zsync".to_owned()),
        },
        warnings: Vec::new(),
        confidence: 75,
    }
}

fn extract_field(contents: &str, prefix: &str) -> Option<String> {
    contents
        .lines()
        .find_map(|line| line.trim().strip_prefix(prefix).map(str::trim))
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
}

fn version_from_filename(filename: &str) -> Option<String> {
    filename
        .split('-')
        .find(|segment| segment.chars().any(|ch| ch.is_ascii_digit()) && segment.contains('.'))
        .map(|value| value.trim_end_matches(".AppImage").to_owned())
}
