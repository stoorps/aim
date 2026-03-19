use aim_core::domain::update::ParsedMetadataKind;
use aim_core::metadata::{MetadataDocument, parse_document};

#[test]
fn parses_zsync_document_into_channel_hints() {
    let raw = include_bytes!("fixtures/example.zsync");
    let doc = MetadataDocument::plain_text("https://example.test/app.AppImage.zsync", raw);
    let result = parse_document(&doc).unwrap();

    assert_eq!(result.kind, ParsedMetadataKind::Zsync);
    assert!(result.hints.primary_download.is_some());
}
