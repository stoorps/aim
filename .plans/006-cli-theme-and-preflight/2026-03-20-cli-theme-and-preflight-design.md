# CLI Theme And Preflight Progress Design

## Goal

Polish the CLI install and removal experience so progress stays visibly active before downloads start, transcript output and summaries are better separated, redundant install recap text is removed, and terminal styling is driven by a configurable warm theme instead of ad hoc color choices.

## Problem Statement

The current CLI progress work fixed the large functional gaps, but four UX issues remain:

- install has a silent pre-download period while source resolution and release selection happen
- final install and removal summaries run directly into the transcript without enough visual separation
- the install summary repeats work that the transcript already showed via `Completed steps`
- styling is still thin and hardcoded, with no clear config surface for future CLI presentation settings

The user also wants the styling system to be future-proof:

- coded warm defaults
- external overrides loaded from `config.toml`
- room to extend the same config file beyond theming later
- optional support for hex colors where the terminal supports truecolor

## Design Goals

- show visible work as soon as install begins, not only once bytes start downloading
- keep `stderr` as the live transcript surface and `stdout` as the final summary surface
- ensure a clear blank-line separation between transcript output and final summaries
- remove redundant install recap text from the final summary
- centralize terminal styling behind semantic theme tokens
- support config-driven theme overrides from app-specific system and user paths
- accept both named colors and hex colors while degrading cleanly on limited terminals

## Non-Goals

- a full TUI redesign
- migrating the application to async
- a cross-platform config search redesign beyond the agreed Linux app-specific paths
- full arbitrary terminal capability negotiation beyond practical truecolor or fallback detection
- broader non-theme config features in this slice beyond reserving the config surface for future use

## Approved UX Shape

### Transcript versus summary

- `stderr` remains the place for live progress output
- `stdout` remains the place for final success or completion summaries
- if any transcript lines were emitted, the renderer inserts exactly one blank line before printing the final summary
- install no longer prints a `Completed steps` section in the final summary
- remove keeps the compact summary and removed file list, with the same blank-line separation rule

### Install lifecycle visibility

Install should emit visible progress through the full lifecycle, including the current silent period before download:

- resolving source
- discovering release
- selecting artifact
- downloading artifact
- staging payload
- writing desktop entry
- extracting icon
- refreshing desktop integration
- saving registry

When download byte totals are known, the reporter should continue using the byte progress bar. When byte totals are unavailable, it should still show staged progress honestly.

### Summary shape

Install summary remains compact:

- bold `Installed <name> (<scope>)`
- `Source: ...`
- `Artifact: ...`
- `Installed files:` list when files exist

Remove summary remains compact:

- bold `Removed <name>`
- `Removed files:` list when files exist

The transcript already carries the step-by-step operational recap, so the final install summary should not repeat it.

## Architectural Decision

Keep the existing event-driven split, but make two targeted improvements:

1. extend `aim-core` install reporting so pre-download work emits stages instead of happening silently
2. move CLI presentation onto a theme token layer backed by a loadable config file

This preserves the intended boundary:

- `aim-core` owns workflow semantics and event emission
- `aim-cli` owns config discovery, theme resolution, terminal capability handling, spacing, and rendering

## Theme And Config Model

### Config file locations

The CLI should load configuration from app-specific Linux paths in this order:

- `/etc/aim/config.toml`
- `~/.config/aim/config.toml`

User config overrides system config.

### Config file shape

The file is intentionally broader than theming so it can grow later without another migration. This slice only consumes the `[theme]` table.

Example:

```toml
[theme]
heading = "#d28b26"
accent = "teal"
muted = "dim"
label = "bold #e7c58a"
success = "green"
warning = "yellow"
error = "red"
progress_spinner = "#d28b26"
progress_bar = "#d28b26"
progress_bar_unfilled = "#6f6253"
```

### Theme token layer

Renderers should consume semantic tokens, not direct color selections. The token set should cover at least:

- `heading`
- `accent`
- `muted`
- `label`
- `bullet`
- `success`
- `warning`
- `error`
- `progress_spinner`
- `progress_bar`
- `progress_bar_unfilled`

The built-in default theme is warm:

- amber or gold for headings and active progress
- teal as secondary accent
- warm gray or sand for muted and supporting text
- semantic success, warning, and error colors reserved for status meaning

### Named colors and hex colors

`config.toml` should accept either:

- named values like `amber`, `teal`, `green`, `dim`
- hex values like `#d28b26`
- style combinations such as `bold amber` or `bold #d28b26`

Internally, theme values should normalize into a small style model that can render as:

- plain text
- ANSI basic or 256-style fallback
- truecolor RGB when supported

## Terminal Capability Strategy

Modern terminals can render hex-configured colors by converting them into 24-bit ANSI sequences, but support is not universal. The CLI should therefore:

- use truecolor when terminal capability is available
- fall back to a nearest named or ANSI-safe color when truecolor is unavailable
- fall back to plain text when color is disabled or unsupported

Color should improve presentation, not become a hard dependency for readability.

## Error Handling

Config parsing must be non-fatal.

- missing config files: ignore and use defaults
- partial config: merge valid fields, use defaults for the rest
- invalid values: ignore the bad value, keep defaults, optionally emit a warning
- config load failure must never block installs, updates, listing, or removals

This keeps the CLI robust while still making misconfiguration visible.

## Implementation Boundaries

### `aim-core`

- emit `ResolveQuery`, `DiscoverRelease`, and `SelectArtifact` during install preflight
- keep terminal decisions out of core logic

### `aim-cli`

- load and merge config from agreed paths
- resolve theme tokens from defaults plus config overrides
- detect terminal color capability pragmatically
- render transcript and summaries with a single semantic theme layer
- track whether transcript output occurred so the final summary spacing rule is applied once

## Testing Strategy

### Core tests

- verify install emits the new early stages before download begins
- verify event ordering remains coherent across the full install flow

### CLI renderer tests

- verify exactly one blank line separates transcript output from final summaries
- verify install no longer renders `Completed steps`
- verify compact install and removal summaries remain intact

### Config tests

- system config loads successfully
- user config overrides system config
- invalid config values fall back to defaults without aborting commands
- named colors and hex colors both parse

### Progress tests

- pre-download transcript lines appear before byte download output
- non-interactive mode still records final byte counts correctly
- truecolor-capable styling degrades safely when color support is limited or disabled

## Rollout Order

Implement in this order:

1. config and theme token model in `aim-cli`
2. failing tests for spacing, non-redundant summaries, config loading, and early install stages
3. early install stage emission in `aim-core`
4. progress reporter updates for transcript spacing and themed styling
5. final renderer cleanup and summary simplification
6. documentation refresh if needed once output is final