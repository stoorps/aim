use std::path::Path;

use crate::app::add::{
    AddPlan, AddSecurityPolicy, BuildAddPlanError, InstalledApp,
    build_add_plan_with_registered_providers,
    build_add_plan_with_reporter_and_registered_providers, install_app_with_reporter,
};
use crate::app::list::{ListRow, build_list_rows};
use crate::app::progress::ProgressReporter;
use crate::app::providers::ProviderRegistry;
use crate::app::remove::{RemovalResult, RemoveRegisteredAppError, remove_registered_app};
use crate::app::search::{SearchError, SearchProvider, build_search_results_with};
use crate::app::show::build_show_result_with;
use crate::app::update::{
    BuildUpdatePlanError, ExecuteUpdatesError, build_update_plan,
    execute_updates_with_reporter_and_policy,
};
use crate::domain::app::{AppRecord, InstallScope};
use crate::domain::search::{SearchQuery, SearchResults};
use crate::domain::show::{InstalledShow, ShowResult, ShowResultError};
use crate::domain::update::{UpdateExecutionResult, UpdatePlan};
use crate::source::github::{GitHubTransport, default_transport};

pub struct UpmApp<'a> {
    github_transport: Box<dyn GitHubTransport>,
    providers: ProviderRegistry<'a>,
}

pub struct UpmAppBuilder<'a> {
    github_transport: Option<Box<dyn GitHubTransport>>,
    providers: ProviderRegistry<'a>,
}

impl UpmApp<'static> {
    pub fn new() -> Self {
        Self::builder()
            .with_provider_registry(default_provider_registry())
            .build()
    }
}

impl Default for UpmApp<'static> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> UpmApp<'a> {
    pub fn builder() -> UpmAppBuilder<'a> {
        UpmAppBuilder {
            github_transport: None,
            providers: ProviderRegistry::default(),
        }
    }

    pub fn search(
        &self,
        query: &SearchQuery,
        installed_apps: &[AppRecord],
    ) -> Result<SearchResults, SearchError> {
        let github_provider =
            crate::app::search::GitHubSearchProvider::new(self.github_transport.as_ref());
        let mut resolved_providers = vec![&github_provider as &dyn SearchProvider];
        resolved_providers.extend(
            self.providers
                .search_providers
                .iter()
                .map(|provider| provider.as_ref() as &dyn SearchProvider),
        );
        build_search_results_with(query, installed_apps, &resolved_providers)
    }

    pub fn build_add_plan(
        &self,
        query: &str,
        policy: AddSecurityPolicy,
    ) -> Result<AddPlan, BuildAddPlanError> {
        build_add_plan_with_registered_providers(
            query,
            self.github_transport.as_ref(),
            &self.providers,
            policy,
        )
    }

    pub fn build_add_plan_with_reporter(
        &self,
        query: &str,
        reporter: &mut impl ProgressReporter,
        policy: AddSecurityPolicy,
    ) -> Result<AddPlan, BuildAddPlanError> {
        build_add_plan_with_reporter_and_registered_providers(
            query,
            self.github_transport.as_ref(),
            reporter,
            &self.providers,
            policy,
        )
    }

    pub fn install_app(
        &self,
        query: &str,
        plan: &AddPlan,
        install_home: &Path,
        requested_scope: InstallScope,
        reporter: &mut impl ProgressReporter,
    ) -> Result<InstalledApp, crate::app::add::InstallAppError> {
        install_app_with_reporter(query, plan, install_home, requested_scope, reporter)
    }

    pub fn show(
        &self,
        query: &str,
        installed_apps: &[AppRecord],
    ) -> Result<ShowResult, ShowResultError> {
        build_show_result_with(query, installed_apps, self.github_transport.as_ref())
    }

    pub fn show_all(&self, installed_apps: &[AppRecord]) -> Vec<InstalledShow> {
        crate::app::show::build_installed_show_results(installed_apps)
    }

    pub fn list(&self, apps: &[AppRecord]) -> Vec<ListRow> {
        build_list_rows(apps)
    }

    pub fn build_update_plan(
        &self,
        apps: &[AppRecord],
    ) -> Result<UpdatePlan, BuildUpdatePlanError> {
        build_update_plan(apps)
    }

    pub fn execute_updates(
        &self,
        apps: &[AppRecord],
        install_home: &Path,
        reporter: &mut impl ProgressReporter,
        policy: AddSecurityPolicy,
    ) -> Result<UpdateExecutionResult, ExecuteUpdatesError> {
        execute_updates_with_reporter_and_policy(apps, install_home, reporter, policy)
    }

    pub fn remove_registered_app(
        &self,
        query: &str,
        apps: &[AppRecord],
        install_home: &Path,
    ) -> Result<RemovalResult, RemoveRegisteredAppError> {
        remove_registered_app(query, apps, install_home)
    }
}

impl<'a> UpmAppBuilder<'a> {
    pub fn with_github_transport(mut self, github_transport: Box<dyn GitHubTransport>) -> Self {
        self.github_transport = Some(github_transport);
        self
    }

    pub fn with_provider_registry(mut self, providers: ProviderRegistry<'a>) -> Self {
        self.providers = providers;
        self
    }

    pub fn build(self) -> UpmApp<'a> {
        UpmApp {
            github_transport: self.github_transport.unwrap_or_else(default_transport),
            providers: self.providers,
        }
    }
}

fn default_provider_registry() -> ProviderRegistry<'static> {
    ProviderRegistry::default()
        .with_search_provider(upm_appimage::AppImageHubSearchProvider::new(
            upm_appimage::source::appimagehub::default_transport(),
        ))
        .with_external_add_provider(upm_appimage::AppImageHubAddProvider::new(
            upm_appimage::source::appimagehub::default_transport(),
        ))
}
