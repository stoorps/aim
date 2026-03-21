use upm_core::domain::update::ParsedMetadataKind;
use upm_core::metadata::{MetadataDocument, parse_document};

#[test]
fn unknown_document_returns_typed_warning_not_panic() {
    let doc = MetadataDocument::plain_text("https://example.test/notes.txt", b"not metadata");
    let result = parse_document(&doc).unwrap();

    assert_eq!(result.kind, ParsedMetadataKind::Unknown);
    assert!(!result.warnings.is_empty());
}
