use aim_core::adapters::all_adapter_kinds;

#[test]
fn all_expected_adapter_kinds_are_registered() {
    let kinds = all_adapter_kinds();

    assert!(kinds.contains(&"gitlab"));
    assert!(kinds.contains(&"direct-url"));
    assert!(kinds.contains(&"zsync"));
    assert!(kinds.contains(&"sourceforge"));
    assert!(kinds.contains(&"custom-json"));
}
