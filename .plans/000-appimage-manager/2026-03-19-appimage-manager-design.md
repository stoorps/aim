# AppImage Manager Design

## Goal

Build AppImage Manager as a Rust workspace where `aim-core` contains the business logic and reusable APIs, and `aim-cli` is a thin terminal client. The first shipped application is the CLI, but the architecture must leave a clean path for a later GUI client to consume the same core install, update, registry, and adapter logic.

## Agreed Product Shape

### Command surface

- `aim {QUERY}`: search and add from a query source
- `aim`: review-first update flow; aliases `aim update`
- `aim update`: explicit update flow
- `aim remove {QUERY}`: remove by registered app name
- `aim list`: list installed AppImages

### Supported source types

1. GitHub Releases
2. Direct URL / generic website downloads
3. GitLab Releases
4. zsync / embedded AppImage update info
5. SourceForge
6. Custom JSON feed adapters

### Install behavior

- Default installation mode is auto-detected by effective privileges
- `--system` and `--user` override the auto-detected scope
- The tool supports both user and system installations
- The tool performs full desktop-style integration for installed apps

### Identity and update behavior

- The system should infer app identity and version when possible
- If confidence is low, the client should prompt interactively for confirmation or edits
- If identity still cannot be stabilized, the registry should fall back to the raw URL as the last-resort key
- Running `aim` with no query should discover updates, present a review list, and then apply only selected updates
- Architecture handling should remain generic: `aim-core` manages whatever AppImage artifact is resolved, while validating obvious mismatches at install time

## Recommended Architecture

Use typed source adapters behind a common update engine, packaged in `aim-core` and consumed by thin frontend clients.

This architecture fits the source diversity without forcing a plugin runtime into v1. Each upstream source gets an explicit Rust adapter that implements a shared contract for identity resolution, release discovery, artifact selection, and update metadata extraction. The shared update engine operates on normalized internal types rather than source-specific details.

This approach was selected over:

- a registry-centric-first design, which risks smearing source-specific logic across storage and service layers
- a plugin-first design, which adds packaging, security, and testing complexity too early

## Workspace Architecture

The project should be a Cargo workspace with frontend clients over a shared core crate.

### Workspace crates

- `crates/aim-core`: all business logic and reusable APIs
- `crates/aim-cli`: thin terminal frontend for the initial shipped application
- `crates/aim-gui`: deferred future GUI client, planned but not implemented in v1

The critical rule is that `aim-cli` must not become the home for install, update, registry, or source logic. If behavior should be reusable by a future GUI, it belongs in `aim-core`.

## Architecture Layers

The system should be organized into four layers, with the bottom three living in `aim-core`.

### 1. Client layer

- Implemented first in `aim-cli`
- Parses commands, flags, and defaults
- Owns presentation only: prompts, colors, spinners, progress bars, and terminal summaries
- Uses:
  - `clap` for CLI parsing
  - `dialoguer` for interactive prompts and multi-select review flows
  - `console` for styled output and readable summaries
  - `indicatif` for progress bars and spinners

This layer should translate user intent into calls into `aim-core` and render responses. It should not contain source-specific business logic, registry mutation logic, or install/update decision logic.

### 2. Application/service layer

- Lives in `aim-core`
- Coordinates workflows like add, remove, list, and update
- Applies product rules such as scope selection, update review behavior, and low-confidence identity confirmation
- Suggested services:
  - `AddService`
  - `UpdateService`
  - `RegistryService`
  - `IntegrationService`

### 3. Domain model layer

- Lives in `aim-core`
- Holds the canonical source-agnostic types used across the system

Suggested domain types:

- `AppRecord`
- `InstallScope`
- `SourceRef`
- `SourceKind`
- `ResolvedRelease`
- `InstalledArtifact`
- `UpdatePlan`
- `DesktopIntegration`
- `InteractionRequest`
- `InteractionResponse`

### 4. Infrastructure layer

- Lives in `aim-core`
- Source adapters
- Filesystem and install location management
- Registry persistence
- Desktop integration helpers
- Download and HTTP client behavior
- Optional subprocess wrappers for system integration tasks

## Suggested Module Layout

Suggested workspace layout:

- `Cargo.toml`
- `crates/aim-core/Cargo.toml`
- `crates/aim-core/src/lib.rs`
- `crates/aim-core/src/app/`
- `crates/aim-core/src/domain/`
- `crates/aim-core/src/adapters/`
- `crates/aim-core/src/integration/`
- `crates/aim-core/src/registry/`
- `crates/aim-core/src/platform/`
- `crates/aim-cli/Cargo.toml`
- `crates/aim-cli/src/lib.rs`
- `crates/aim-cli/src/main.rs`
- `crates/aim-cli/src/cli/`
- `crates/aim-cli/src/ui/`

Future-facing placeholder:

- `crates/aim-gui/`

This keeps terminal UX separate from the install/update engine and ensures the later GUI can reuse the same core APIs.

## Core Components

### Query resolver

Lives in `aim-core` and turns user input into a normalized `SourceRef`.

Accepted input forms:

- URL
- `user_or_org/project`
- file URI
- bare `aim` with no query

Behavior:

- Resolve GitHub URLs and `owner/repo` forms to GitHub when unambiguous
- Resolve GitLab URLs and explicit `gitlab:` references to GitLab
- Resolve direct URLs and generic web pages to the direct URL / web adapter
- Resolve `file://` inputs into local import flow

The query resolver should not perform install logic.

### Source adapter layer

Lives in `aim-core`, with one typed adapter per source:

- GitHub Releases adapter
- GitLab Releases adapter
- Direct URL / generic web adapter
- zsync / embedded update info adapter
- SourceForge adapter
- Custom JSON feed adapter

Each adapter should expose a shared capability shape:

- identify app
- enumerate candidate releases
- choose preferred artifact
- expose update metadata
- download or resolve the artifact for download

Not every source needs to support true search. Some only support exact resolution. The contract should represent those differences honestly.

### Registry

Lives in `aim-core` and stores normalized installed app records across user and system scopes.

It should track:

- canonical app identity
- display name
- install scope
- source type
- source locator and source-specific update hints
- installed version
- installed artifact path
- artifact fingerprint or hash
- release metadata
- integration artifact paths
- timestamps

The registry is the bridge between one-time install and repeatable updates, so it must be migration-friendly.

### Installer and integrator

Live in `aim-core`.

Installer responsibilities:

- staging downloads
- validating artifacts
- moving binaries into managed locations
- setting permissions
- replacing installed artifacts atomically where possible

Integrator responsibilities:

- `.desktop` entry generation
- icon extraction or acquisition
- symlink creation
- MIME and related registration where feasible
- correct handling of user vs system targets

Installer and integration concerns should remain separate so updates can replace binaries without always rebuilding every integration artifact.

### Update planner and executor

Live in `aim-core`.

Planner responsibilities:

- load registry entries
- ask adapters for update candidates
- compare installed state to available state
- build a reviewable `UpdatePlan`

Executor responsibilities:

- apply selected updates
- download and validate updated artifacts
- replace existing artifacts safely
- refresh integration artifacts only when needed
- update registry state
- surface typed results and events for clients

### Client interaction boundary

Terminal-specific UI belongs in `aim-cli`, not `aim-core`.

`aim-core` should expose operation APIs and typed interaction or progress models that clients can render however they want. `aim-cli` should wrap all usage of `dialoguer`, `console`, and `indicatif`.

This keeps business logic testable without terminal coupling and makes a GUI frontend viable later.

### Custom JSON feed support

Custom JSON feeds should be declarative in v1, not arbitrary executable plugins.

The adapter should support field mapping and release selection rules against a constrained schema family, rather than loading arbitrary code. This delivers flexibility without turning the CLI into a plugin host.

## End-to-End Data Flow

### `aim {QUERY}` add flow

1. `aim-cli` parses CLI input and scope override flags
2. `aim-cli` calls `aim-core` with a normalized request
3. `aim-core` resolves the query into a `SourceRef`
4. `aim-core` selects the appropriate adapter
5. `aim-core` identifies the app and candidate releases
6. `aim-core` returns an interaction state if confidence is low
7. `aim-cli` prompts, then sends the decision back to `aim-core`
8. `aim-core` falls back to raw URL identity if needed
9. `aim-core` downloads to staging
10. `aim-core` validates the artifact as an AppImage and inspects update metadata
11. `aim-core` installs into the correct managed location
12. `aim-core` generates integration artifacts and persists a normalized registry entry

### `aim` and `aim update` flow

1. `aim-cli` invokes update discovery in `aim-core`
2. `aim-core` loads relevant registry entries
3. `aim-core` asks each adapter for update candidates
4. `aim-core` builds an `UpdatePlan`
5. `aim-cli` renders the review list and collects selection
6. `aim-core` applies selected updates
7. `aim-core` refreshes registry state and integration artifacts as needed
8. `aim-cli` prints a final success/failure summary

### `aim list` flow

- `aim-cli` requests installed app state from `aim-core` and renders the result grouped by scope, source, and version

### `aim remove {QUERY}` flow

1. `aim-cli` forwards the query to `aim-core`
2. `aim-core` resolves the query against registered app names
3. `aim-core` emits an interaction request if ambiguity must be resolved
4. `aim-cli` prompts if needed and returns the selection
5. `aim-core` removes artifacts and integration files in the correct order
6. `aim-core` removes registry state while preserving uncertain shared resources conservatively

## Registry Data Shape

Each registry record should contain enough source-specific state to make updates reliable without re-deriving identity from filenames.

Recommended fields:

- stable app id
- display name
- install scope
- source kind
- source locator
- installed version
- installed file path
- file hash or fingerprint
- release metadata
- updater metadata
- integration artifact paths
- created and updated timestamps

Examples of source-specific metadata:

- GitHub/GitLab: owner, repo, release/tag, asset selection hints
- Direct URL: original URL, resolved URL, etag, last-modified when available
- zsync: zsync URL or embedded update info extracted from the AppImage
- SourceForge: project and file path hints
- Custom JSON feed: feed URL plus mapping profile

## Error Handling Model

Error handling should be structured internally and concise externally.

`aim-core` should own structured error types and machine-readable outcomes. `aim-cli` should map those into concise terminal messages. A future GUI should be able to present the same failures without reparsing CLI text.

Suggested error categories:

- query resolution error
- source adapter error
- network/download error
- artifact validation error
- install permission or scope error
- desktop integration error
- registry persistence error
- update planning error

Behavioral expectations:

- prompt on low-confidence identity rather than silently guessing
- fail clearly on insufficient privileges for system install unless explicit elevation behavior is designed later
- continue update processing across apps when one app fails
- fail-fast within a single app transaction unless a step is intentionally non-fatal
- either roll back on integration failure or explicitly record the app as installed-but-needing-repair

For v1, prefer:

- best-effort continuation across apps during update runs
- fail-fast inside a single app update or install transaction
- atomic replacement where possible
- future room for an `aim repair` command, even if not implemented in v1

## Testing Strategy

Testing should map directly to the architecture layers.

### `aim-core` unit tests

- query parsing and source resolution
- identity normalization and fallback logic
- version comparison and update selection logic
- install scope resolution
- registry serialization and migrations
- adapter-specific parsing helpers

### Shared adapter contract tests

Every adapter should pass a common behavior suite where applicable:

- can identify app
- can resolve latest candidate
- reports unsupported capabilities honestly
- produces normalized release metadata

This is the primary protection against drift across heterogeneous source implementations.

### `aim-core` integration tests

- add flow per source type using fixtures or mocked HTTP
- update planning across mixed registry entries
- remove flow cleaning registry and integration artifacts
- user vs system path resolution
- registry migration compatibility

### Filesystem tests

Use temp directories to simulate:

- user install roots
- system install roots
- desktop entry locations
- icon and symlink generation

### `aim-cli` client behavior tests

- snapshot or golden tests for key terminal flows
- update review list interaction
- low-confidence identity prompt
- success and failure summaries

Most behavioral coverage should target `aim-core`, with only thin client verification in `aim-cli`.

Avoid relying on live network tests in the main suite. Keep those as optional smoke coverage.

### Main risks the test plan must cover

1. Incorrect identity causing duplicate or non-updatable entries
2. Source-specific regressions hidden behind a shared API surface
3. Incomplete rollback leaving broken installs
4. Scope confusion causing files to land in the wrong locations
5. Business logic leaking into `aim-cli` and diverging from future GUI needs

## Recommended Persisted Formats And Key Decisions

### Persisted formats

- Use a structured registry file or registry store that is easy to migrate and inspect
- Keep source-specific update metadata embedded in each app record rather than scattered across auxiliary files
- Store integration artifact paths explicitly so removal and repair remain deterministic

### Key design decisions

- Use a Cargo workspace with `aim-core` and `aim-cli`
- Put all business logic in `aim-core`
- Keep `aim-cli` as a thin terminal adapter over `aim-core`
- Design `aim-core` to be reusable by a future `aim-gui`
- Use typed Rust adapters behind a common update engine
- Normalize identity early and once
- Separate update planning from update execution
- Treat custom JSON feeds as declarative adapters, not executable plugins
- Auto-detect scope by effective privileges, with `--system` and `--user` overrides
- Make bare `aim` a review-first update path

## Explicit v1 Boundaries

Included in v1:

- Cargo workspace with `aim-core` and `aim-cli`
- multi-source AppImage add flow
- user and system scope support
- update planning and selected update execution
- desktop-style integration
- typed adapters for the agreed source list
- declarative custom JSON feed support

Deferred from v1:

- `aim-gui` implementation
- general plugin runtime
- arbitrary executable custom adapters
- broad distro-specific deep integration beyond the agreed desktop registration model
- live network-dependent test suite as the main verification strategy
- repair and doctor commands, though the design should leave room for them

## Open Implementation Notes

- Because the current workspace is not a git repository, the design document can be saved but not committed yet
- The next step should be an implementation plan that breaks this design into small TDD-oriented tasks