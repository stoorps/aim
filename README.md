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
aim remove <QUERY>
```

## Query Forms

- `owner/repo` for GitHub shorthand
- GitHub repository URLs
- GitHub release URLs
- direct GitHub release asset URLs
- `https://...` direct URLs
- GitLab URLs
- `file://...` local file imports

## Scope Overrides

By default `aim` auto-detects whether to use user or system scope. Override that with:

- `--user`
- `--system`

## Current Flow Shape

- `aim <QUERY>` installs unambiguous apps, persists them into the registry after successful install, and renders review prompts when tracking needs confirmation
- bare `aim` and `aim update` build a review-first update plan
- `aim list` renders registered applications
- `aim remove <QUERY>` resolves a registered application name before removal
