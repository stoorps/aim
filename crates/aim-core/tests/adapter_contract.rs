use aim_core::adapters::traits::AdapterCapabilities;

#[test]
fn adapter_capabilities_can_report_exact_resolution_only() {
    let capabilities = AdapterCapabilities::exact_resolution_only();
    assert!(!capabilities.supports_search);
}
