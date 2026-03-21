use upm_core::domain::update::ParsedMetadataKind;
use upm_core::metadata::{MetadataDocument, parse_document};

#[test]
fn parses_latest_linux_yml_into_download_hints() {
    let raw = include_bytes!("fixtures/latest-linux.yml");
    let doc = MetadataDocument::yaml("https://example.test/latest-linux.yml", raw);
    let result = parse_document(&doc).unwrap();

    assert_eq!(result.kind, ParsedMetadataKind::ElectronBuilder);
    assert_eq!(
        result.hints.primary_download.as_deref(),
        Some("T3-Code-0.0.11-x86_64.AppImage")
    );
}
