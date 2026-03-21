# upm
Universal Package Manager

`upm` is a Rust Cargo workspace for a modular package manager with a shared headless core and provider crates.

## Workspace

- `crates/upm-core`: headless application layer for query normalization, resolution, planning, registry persistence, install/update orchestration, and provider-facing APIs
- `crates/upm`: thin terminal frontend for argument parsing, config loading, prompting, progress reporting, and summary rendering
- `crates/upm-appimage`: AppImageHub transport, search, and add-provider integration composed into the CLI through `ProviderRegistry`

The split is intentional so future frontends can reuse `upm-core`, while package-source behavior stays modular instead of being hardcoded into the core.

## Commands

```text
upm <QUERY>
upm
upm update
upm list
upm search <QUERY>
upm remove <QUERY>
```

## Query Forms

- `owner/repo` for GitHub shorthand
- `appimagehub/<id>` for AppImageHub shorthand
- GitHub repository URLs
- GitHub release URLs
- direct GitHub release asset URLs
- AppImageHub item URLs such as `https://www.appimagehub.com/p/2338455`
- `https://...` direct URLs
- GitLab URLs
- SourceForge URLs
- `file://...` local file imports

## Search

`upm search <QUERY>` is part of the initial modular provider surface.

- search is provider-extensible and currently includes GitHub plus AppImageHub
- search results should resolve to install-ready queries such as `owner/repo` and `appimagehub/<id>`
- provider composition happens in `crates/upm/src/providers.rs`, not through AppImageHub-specific wiring inside `upm-core`

## Scope Overrides

By default `upm` auto-detects whether to use user or system scope. Override that with:

- `--user`
- `--system`

## Config

Runtime config is loaded from `~/.config/upm/config.toml` or `$XDG_CONFIG_HOME/upm/config.toml`.

Example:

```toml
allow_http = true
```

- `allow_http = false` is the default
- `allow_http` only permits user-supplied `http://` inputs such as direct URL installs or updates from previously installed direct HTTP origins
- provider-resolved downloads such as AppImageHub artifacts remain HTTPS-only even when `allow_http = true`

## Breaking Rename

- `upm` is a hard rename from `aim`
- runtime overrides now use `UPM_*` names such as `UPM_CONFIG_PATH` and `UPM_REGISTRY_PATH`
- old `AIM_*` runtime overrides are intentionally ignored
- default config and registry locations now live under `upm` paths

## Current Flow Shape

- `upm <QUERY>` installs direct provider matches when available, otherwise falls back to search results, shows live progress on stderr, prints an `Installation Summary` on stdout for installs, and renders an `Installation Review` when tracking needs confirmation
- bare `upm` prints an `Update Review` without mutating the registry
- `upm update` executes the pending updates, streams live status on stderr, then prints an `Update Summary`
- `upm list` renders either `Installed Apps` or `No installed apps yet`
- `upm remove <QUERY>` resolves a registered application name, streams removal progress on stderr, then prints a `Removal Summary`

## Terminal UX

- prompts use `dialoguer`
- styled summaries use `console`
- live spinners and byte progress use `indicatif`
