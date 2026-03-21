use upm_appimage::AppImageHubAddProvider;
use upm_appimage::AppImageHubSearchProvider;
use upm_appimage::source::appimagehub;
use upm_core::ProviderRegistry;

pub fn with_provider_registry<T>(build: impl FnOnce(&ProviderRegistry<'_>) -> T) -> T {
    let appimagehub_transport = appimagehub::default_transport();
    let appimagehub_search = AppImageHubSearchProvider::new(appimagehub_transport.as_ref());
    let appimagehub_add = AppImageHubAddProvider::new(appimagehub_transport.as_ref());
    let providers = ProviderRegistry {
        search_providers: vec![&appimagehub_search],
        external_add_providers: vec![&appimagehub_add],
    };

    build(&providers)
}
