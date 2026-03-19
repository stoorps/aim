use aim_core::registry::store::RegistryStore;
use tempfile::tempdir;

#[test]
fn registry_round_trips_app_records() {
    let dir = tempdir().unwrap();
    let store = RegistryStore::new(dir.path().join("registry.toml"));
    let loaded = store.load().unwrap();
    assert!(loaded.apps.is_empty());
}
