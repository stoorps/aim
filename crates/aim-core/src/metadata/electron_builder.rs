use crate::domain::update::{MetadataHints, ParsedMetadata, ParsedMetadataKind};
use crate::metadata::document::MetadataDocument;

pub fn parse(document: &MetadataDocument) -> ParsedMetadata {
    let contents = String::from_utf8_lossy(&document.contents);
    let version = extract_value(&contents, "version:");
    let path = extract_value(&contents, "path:").or_else(|| extract_value(&contents, "url:"));
    let checksum = extract_value(&contents, "sha512:");

    ParsedMetadata {
        kind: ParsedMetadataKind::ElectronBuilder,
        hints: MetadataHints {
            version,
            primary_download: path,
            checksum,
            architecture: Some("x86_64".to_owned()),
            channel_label: Some("latest".to_owned()),
        },
        warnings: Vec::new(),
        confidence: 90,
    }
}

fn extract_value(contents: &str, prefix: &str) -> Option<String> {
    contents
        .lines()
        .find_map(|line| line.trim().strip_prefix(prefix).map(str::trim))
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
}
