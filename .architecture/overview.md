# Architecture Overview

## Workspace Shape

`aim` is a Rust workspace with two main crates:

- `crates/aim-core`: source normalization, provider adapters, install/update planning, payload installation, registry persistence, and desktop integration.
- `crates/aim-cli`: argument parsing, config loading, terminal UX, prompting, progress reporting, and summary rendering.

The split keeps product logic in `aim-core` so additional frontends can reuse the same install and update pipeline.

## Core Flow

The main execution path is:

1. Parse CLI input and load runtime config in `aim-cli`.
2. Resolve the query into a normalized source in `aim-core`.
3. Build an add or update plan through provider adapters and artifact selection.
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

Provider-specific resolution lives in `crates/aim-core/src/adapters` and `crates/aim-core/src/source`.

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

- Registry writes are atomic and live under the registry store implementation in `aim-core`.
- Managed payload, desktop entry, and icon paths are resolved from install policy and scope.
- Desktop integration refresh uses external helpers when available and now supports env-gated audit logging through `AIM_DEBUG_EXTERNAL_HELPERS=1`.

## Planning And Audit Artifacts

- implementation plans live under `.plans/`
- audit reports live under `.audits/`
- architecture state and tracked security issues live under `.architecture/`