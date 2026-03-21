# Architecture Overview

## Workspace Shape

`upm` is a Rust workspace with three main crates:

- `crates/upm-core`: source normalization, add/update orchestration, registry persistence, install policies, desktop integration, and the provider-composition seam.
- `crates/upm`: argument parsing, config loading, terminal UX, prompting, progress reporting, summary rendering, and provider assembly.
- `crates/upm-appimage`: AppImageHub transport, search-provider behavior, and exact add-resolution for AppImage-backed installs.

The split keeps frontend-agnostic logic in `upm-core`, while concrete package-source behavior is composed at the CLI boundary. That keeps the headless layer reusable for future frontends without making provider behavior a permanent core dependency.

## Core Flow

The main execution path is:

1. Parse CLI input and load runtime config in `upm`.
2. Assemble a `ProviderRegistry` in `crates/upm/src/providers.rs`.
3. Resolve the query into a normalized source in `upm-core`.
4. Build an add or update plan through core orchestration plus any registered external providers.
4. Download the selected AppImage into a staged path.
5. Verify integrity metadata when available.
6. Commit the payload into the managed install location.
7. Write desktop integration artifacts and refresh helper caches.
8. Persist registry state atomically.

## Source And Provider Model

Supported source classes currently include:

- GitHub repository and release forms
- GitLab repository forms
- AppImageHub item forms
- SourceForge release and download forms
- direct URLs
- local file imports

Core source normalization and orchestration live in `crates/upm-core`. AppImageHub-specific transport and provider behavior live in `crates/upm-appimage` and are injected through `ProviderRegistry` rather than hardcoded into core entrypoints.

## Runtime Interface

The rename to `upm` is a hard cutover:

- runtime overrides use `UPM_*`
- legacy `AIM_*` runtime overrides are not read
- default config, registry, payload, and desktop-entry paths use `upm` names
- helper audit logging uses `UPM_DEBUG_EXTERNAL_HELPERS=1`

## Security Hardening State

The current workspace enforces the following download and install boundaries:

- user-supplied `http://` inputs are rejected by default
- runtime opt-in is available through `allow_http = true`
- that opt-in applies only to user-supplied sources, including update flows derived from stored direct HTTP origins
- AppImageHub provider-returned download URLs must remain HTTPS
- AppImageHub MD5 metadata is verified as weak integrity before payload commit
- desktop entry display names are sanitized to prevent newline and control-character field injection
- stable identifiers that normalize to empty or contain `..` are rejected

The remaining deferred AppImageHub host-trust concern is tracked in `security-issues.md`.

## Persistence And Integration

- Registry writes are atomic and live under the registry store implementation in `upm-core`.
- Managed payload, desktop entry, and icon paths are resolved from install policy and scope.
- Desktop integration refresh uses external helpers when available and now supports env-gated audit logging through `UPM_DEBUG_EXTERNAL_HELPERS=1`.

## Planning And Audit Artifacts

- implementation plans live under `.plans/`
- audit reports live under `.audits/`
- architecture state and tracked security issues live under `.architecture/`