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
- GitHub repository URLs
- GitHub release URLs
- direct GitHub release asset URLs
- `https://...` direct URLs
- GitLab URLs
- SourceForge URLs
- `file://...` local file imports

## Search

`aim search <QUERY>` is part of v0.9 finalisation.

- v0.9 search is GitHub-backed first
- search results should resolve to install-ready GitHub shorthand such as `owner/repo`
- the search model is provider-extensible for future phases
- `custom-json` is deferred and is not part of the v0.9 search or install contract

## Scope Overrides

By default `aim` auto-detects whether to use user or system scope. Override that with:

- `--user`
- `--system`

## Current Flow Shape

- `aim <QUERY>` installs unambiguous apps, shows live progress on stderr, prints an `Installation Summary` on stdout, and renders an `Installation Review` when tracking needs confirmation
- bare `aim` prints an `Update Review` without mutating the registry
- `aim update` executes the pending updates, streams live status on stderr, then prints an `Update Summary`
- `aim list` renders either `Installed Apps` or `No installed apps yet`
- `aim remove <QUERY>` resolves a registered application name, streams removal progress on stderr, then prints a `Removal Summary`

## Terminal UX

- prompts use `dialoguer`
- styled summaries use `console`
- live spinners and byte progress use `indicatif`
