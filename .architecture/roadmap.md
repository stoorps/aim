# UPM Roadmap

## Direction

The project is evolving from a focused AppImage manager into `upm`, a modular universal package manager. The target system manages multiple package sources through a shared headless core, keeps the CLI thin, and leaves room for future GUI frontends.

The initial rename-and-extraction slice has now landed as a hard cutover: the runtime surface is `upm`, the shared core is `upm-core`, and AppImage support is composed through `upm-appimage` instead of being embedded directly into the core.

The near-term goal is a Linux-first platform with honest cross-platform architecture. Phase 2 will implement Linux package sources while establishing the abstractions needed for later macOS support and possible Windows support.

## Confirmed Product Decisions

- `aim-cli` becomes `upm`.
- `aim-core` stops being the all-in-one backend and is split.
- AppImage support becomes an installable module named `upm-appimage`.
- Shared orchestration, config, state, resolution, ranking, and frontend-facing APIs move into `upm-core`.
- The `upm` crate becomes a thin CLI client over `upm-core`.
- The rename is a hard cutover. Legacy `AIM_*` runtime interfaces are removed rather than preserved.
- The declarative package file starts in a hybrid mode and is intended to become the source of truth over time.
- Every `upm` invocation should detect drift between declared state and observed system state, then auto-sync metadata as needed.
- Phase 2 is Linux implementation first, with macOS-oriented provider abstractions and packaging seams designed now.
- Feature delivery should happen as vertical slices rather than a single large refactor.
- The `upm` branch is the effective trunk for this evolution work and should be treated as the integration base for future UPM feature branches and worktrees.

## Architectural Destination

### Workspace Shape

The intended workspace shape after the initial refactor is:

- `upm`: thin CLI frontend, ratatui config UI, command routing, presentation.
- `upm-core`: headless application layer, provider registry, resolution pipeline, state model, declarative sync engine, ranking, policies, and frontend-agnostic APIs.
- `upm-appimage`: AppImage provider module extracted from the current `aim-core` implementation.
- future provider modules: `upm-pacman`, `upm-aur`, `upm-flatpak`, `upm-cargo`, `upm-npm`, and later macOS or Windows-specific modules.

### Module Model

UPM should stay modular in both code and packaging:

- modules can be enabled or disabled by config
- providers can be ranked by user priority
- distro packaging can offer grouped installs such as `upm-full`
- lighter installs can ship only the core and selected modules

This means provider capabilities, discovery, search, install, remove, inspect, and sync behavior need stable interfaces in `upm-core` rather than provider-specific branching in the CLI.

### State Model

The long-term model is declarative and config-first, but Phase 2 begins with a hybrid approach:

- `upm`-managed actions update the declarative config directly
- `upm update` and normal command entrypoints can inspect live system state
- drift detection reconciles unmanaged or changed packages into the config representation
- over time the config becomes authoritative and reconciliation becomes stricter

## Phase 2 Milestones

### Milestone 0: Rename And Split Foundation

Deliver the naming and ownership transition without changing product scope yet.

Goals:

- rename workspace crates and package outputs from `aim` to `upm`
- create `upm-core` by extracting reusable infrastructure from `aim-core`
- reduce the CLI crate to a frontend over headless APIs
- isolate current AppImage-specific logic into `upm-appimage`
- compose provider behavior in the CLI through `ProviderRegistry` rather than hardcoded AppImage paths in `upm-core`
- preserve current AppImage functionality and tests during the move

Exit criteria:

- `upm` binary replaces `aim`
- workspace builds under new crate names
- AppImage flows still work end-to-end through the new layering

### Milestone 1: AppImage On The New Core

Make the current AppImage implementation the first real module on the modular architecture.

Goals:

- validate the provider module contract using AppImage as the reference implementation
- move search, add, install, update, show, and remove behaviors behind core provider APIs
- prove the CLI can treat AppImage as just one enabled source

Exit criteria:

- AppImage support is no longer special-cased as the whole product
- provider registration and capability discovery exist in `upm-core`

### Milestone 2: Linux Native Sources

Add the first non-AppImage providers as Linux-focused vertical slices.

Initial supported sources:

- pacman
- AUR
- Flatpak
- cargo global installs
- npm global installs

Goals:

- implement provider discovery and capability coverage for each source
- normalize package identity, version, installed state, and update candidates across providers
- support search, inspect, install, remove, and update planning where each provider can do so safely
- capture provider limitations explicitly rather than faking uniformity

Exit criteria:

- multi-source search works across enabled providers
- installs and removals work through a consistent command model
- provider-specific metadata is normalized enough for ranking and sync

### Milestone 3: Provider Priority And Config UX

Expose modularity directly in the terminal interface.

Goals:

- build a ratatui configuration menu
- allow enablement and disablement per provider
- allow explicit search and install priority ordering
- allow configuration of module weight, source preference, and future policy toggles
- make search ranking obey configured priority instead of hard-coded source bias

Exit criteria:

- users can manage provider selection and ranking from the TUI
- search results are explainable in terms of configured preference order

### Milestone 4: Declarative Package State And Drift Sync

Introduce the nix-like experience incrementally.

Goals:

- define the declarative package file format in `config.toml` or a closely related managed file
- track installed packages by provider in a normalized state model
- implement `upm update` as both package refresh and state reconciliation
- scan currently installed packages from supported providers and build or refresh the declared package set
- auto-sync detected drift during any `upm` command invocation

Phase 2 intent:

- begin in hybrid mode
- move steadily toward config-first behavior
- avoid destructive reconciliation until provider semantics are trustworthy

Exit criteria:

- users can bootstrap a declarative package definition from the current machine state
- repeated `upm` runs keep declared and observed state aligned
- state drift is surfaced clearly and reconciled predictably

### Milestone 5: Packaging, Distribution, And Platform Seams

Make the modular architecture real at packaging time, not just in code.

Goals:

- define packaging layout for standalone core, selected modules, and full installs
- support distro-level grouped packages such as `upm-full`
- ensure unsupported modules degrade cleanly on the wrong OS or distro
- add macOS provider and packaging seams to `upm-core` even if Linux remains the only implemented provider set in Phase 2

Exit criteria:

- module packaging strategy is documented and testable
- cross-platform abstractions exist without blocking Linux delivery

## Phase 2 Non-Goals

The following are explicitly not required to complete Phase 2:

- full macOS provider implementation
- Windows provider implementation
- GUI frontend delivery
- forcing strict config-authoritative reconciliation before provider behavior is stable
- shipping every conceivable Linux package manager in the first expansion

## Success Criteria

Phase 2 is successful when the project can credibly be described as a modular package manager rather than an AppImage manager with extra adapters.

That means:

- the product name, workspace shape, and binary identity are all `upm`
- AppImage support is only one module among several
- Linux users can manage packages from the first targeted provider set
- ranking and enablement are user-controlled through config and TUI
- declarative state exists, is importable from the live system, and stays synchronized through normal use
- the architecture is ready for GUI and later platform expansion without another major rewrite

## Immediate Planning Order

Implementation plans should follow this order:

1. rename and crate extraction
2. provider API definition and AppImage migration onto `upm-core`
3. Linux provider onboarding in a stable order, likely `pacman` then `Flatpak`, then `AUR`, then `cargo`, then `npm`
4. ratatui configuration and ranking UX
5. declarative state model, drift detection, and `update` sync behavior
6. packaging layout and `upm-full` distribution strategy

This order keeps the refactor defensible, gives each slice a usable product outcome, and avoids locking future provider work into AppImage-era assumptions.