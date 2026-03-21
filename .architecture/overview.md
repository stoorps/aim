# Architecture Overview

## Workspace Shape

`upm` is a Rust workspace with three main crates today and a fourth planned frontend:

- `crates/upm-core`: the application layer for `upm`. It owns command orchestration, module contracts, module registry and composition, registry persistence, install policies, desktop integration, and the unified frontend-facing API that both the CLI and a future GUI will call.
- `crates/upm`: the CLI frontend over `upm-core`. It handles argument parsing, config loading, terminal UX, prompting, progress reporting, summary rendering, and config-driven module presentation.
- `crates/upm-appimage`: the AppImage package-manager module. It should own AppImage-specific acquisition backends, artifact selection, and install-resolution behavior.
- `crates/upm-ui` (planned): a GUI frontend over `upm-core`, not a second application layer.

The intended split is strict:

- `upm-core` is effectively the application
- `upm` is one frontend over that application
- `upm-ui` will be another frontend over that application
- package-manager modules own their own implementation detail and speak to `upm-core` through normalized traits

That keeps frontend-agnostic logic in `upm-core`, makes a future GUI a first-class consumer instead of a later retrofit, and prevents frontend layers from accumulating package-manager-specific behavior.

## Application Boundary

The architectural boundary is:

- `upm` may know which modules exist for configuration, enablement, disablement, priority, and display
- `upm-ui` should operate under the same rule as the CLI: it talks to `upm-core`, not directly to modules
- `upm` must not talk directly to a package-manager module or implement module-specific logic
- `upm-core` owns the unified application interface used by the CLI now and a GUI later
- `upm-core` owns module registration, composition, enablement checks, and request fan-out
- `upm-core` fans requests out to enabled modules and aggregates normalized results
- each module owns its own internal backends, source quirks, artifact selection, and provider-specific rules

In practical terms, `upm-core` is where the product behavior lives. The CLI should remain replaceable.

## Public API Shape

`upm-core` should expose one high-level application facade to frontend crates.

- the public boundary should be an application-facing type such as `UpmApp`
- the facade should present operations like search, add, show, update, remove, and config management in product terms
- frontends should not compose lower-level orchestration services themselves

That public facade should stay thin. The internal implementation in `upm-core` can and should be split into smaller services such as:

- module registry and module loading
- search orchestration
- add planning and execution
- show resolution
- update planning and execution
- configuration and state services

This gives both frontends one stable application boundary without turning the facade into a god object. The orchestration depth stays inside `upm-core`, where it belongs.

## Module Tree

The intended tree is:

- `upm-core`
	- public application facade
	- internal orchestration services
	- module registry and composition
	- normalized contracts for package-manager modules
- frontend crates
	- `upm` for CLI concerns only
	- `upm-ui` for GUI concerns only
- module crates
	- `upm-appimage`
		- AppImageHub backend
		- GitHub-backed AppImage acquisition
		- GitLab-backed AppImage acquisition
		- SourceForge-backed AppImage acquisition
		- direct AppImage URL handling
		- AppImage-specific artifact and metadata rules

The important constraint is that the top layer understands package-manager modules, not the inner mechanics of how each module finds or resolves artifacts.

## Core Flow

The main execution path is:

1. Parse CLI input and load runtime config in `upm`.
2. Call the unified application facade in `upm-core`.
3. Let `upm-core` route the request into internal orchestration services.
4. Let those services select enabled modules and fan the request out through normalized module traits.
5. Aggregate normalized results into an add, show, update, search, or remove flow.
6. Download the selected AppImage into a staged path when the chosen module requires it.
7. Verify integrity metadata when available.
8. Commit the payload into the managed install location.
9. Write desktop integration artifacts and refresh helper caches.
10. Persist registry state atomically.

## Source And Provider Model

Supported source classes currently include:

- GitHub repository and release forms
- GitLab repository forms
- AppImageHub item forms
- SourceForge release and download forms
- direct URLs
- local file imports

Core orchestration and normalized module contracts live in `crates/upm-core`. Package-manager-specific behavior belongs in module crates.

For the AppImage module, that means `crates/upm-appimage` is the package-manager boundary and should grow to own AppImage-specific backing sources internally. `upm-core` should coordinate the module through normalized traits, not absorb AppImageHub, GitHub-backed AppImage discovery, GitLab-backed AppImage discovery, SourceForge-backed AppImage discovery, or direct AppImage URL handling as first-class application concepts.

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