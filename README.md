# aim
AppImage Manager

`aim` is a Rust Cargo workspace for managing AppImages from multiple source types.

## Workspace

- `crates/aim-core`: business logic, source adapters, registry, install/update planning
- `crates/aim-cli`: thin terminal frontend for parsing, prompting, and rendering

The split is intentional so a future GUI client can reuse `aim-core` without moving logic out of the shared library.

## Commands

```text
aim <QUERY>
aim
aim update
aim list
aim search <QUERY>
aim remove <QUERY>
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

`aim search <QUERY>` is part of v0.9 finalisation.

- search is provider-extensible and currently includes GitHub plus AppImageHub
- search results should resolve to install-ready queries such as `owner/repo` and `appimagehub/<id>`
- the search model is provider-extensible for future phases

## Scope Overrides

By default `aim` auto-detects whether to use user or system scope. Override that with:

- `--user`
- `--system`

## Config

Runtime config is loaded from `~/.config/aim/config.toml` or `$XDG_CONFIG_HOME/aim/config.toml`.

Example:

```toml
allow_http = true
```

- `allow_http = false` is the default
- `allow_http` only permits user-supplied `http://` inputs such as direct URL installs or updates from previously installed direct HTTP origins
- provider-resolved downloads such as AppImageHub artifacts remain HTTPS-only even when `allow_http = true`

## Current Flow Shape

- `aim <QUERY>` installs direct provider matches when available, otherwise falls back to search results, shows live progress on stderr, prints an `Installation Summary` on stdout for installs, and renders an `Installation Review` when tracking needs confirmation
- bare `aim` prints an `Update Review` without mutating the registry
- `aim update` executes the pending updates, streams live status on stderr, then prints an `Update Summary`
- `aim list` renders either `Installed Apps` or `No installed apps yet`
- `aim remove <QUERY>` resolves a registered application name, streams removal progress on stderr, then prints a `Removal Summary`

## Terminal UX

- prompts use `dialoguer`
- styled summaries use `console`
- live spinners and byte progress use `indicatif`
