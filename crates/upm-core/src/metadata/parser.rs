use crate::domain::update::{MetadataHints, ParsedMetadata, ParsedMetadataKind};
use crate::metadata::document::MetadataDocument;

pub fn parse_document(document: &MetadataDocument) -> Result<ParsedMetadata, MetadataParseError> {
    if document.url.ends_with("latest-linux.yml") || document.url.ends_with("latest-linux.yaml") {
        return Ok(super::electron_builder::parse(document));
    }

    if document.url.ends_with(".zsync") {
        return Ok(super::zsync::parse(document));
    }

    Ok(ParsedMetadata {
        kind: ParsedMetadataKind::Unknown,
        hints: MetadataHints::default(),
        warnings: vec!["unsupported metadata document".to_owned()],
        confidence: 0,
    })
}

#[derive(Debug, Eq, PartialEq)]
pub enum MetadataParseError {}
