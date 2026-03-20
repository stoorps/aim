# CLI UX And Progress Design

## Goal

Bring the terminal UX back in line with spec 000 by making the CLI visibly active during long-running work and by replacing the current plain-text summaries with a styled, intentional presentation layer.

## Problem Statement

The current CLI has two obvious gaps:

- long-running flows such as `aim <QUERY>` stay silent until the operation is effectively complete
- terminal output across add, update, list, remove, review, and prompts is still raw and inconsistent

This is a product and architecture drift issue relative to the original design. Spec 000 explicitly called for:

- `dialoguer` for prompts
- `console` for styled terminal summaries
- `indicatif` for spinners and progress bars
- terminal rendering in `aim-cli`
- typed progress or interaction models in `aim-core`

## Design Goals

- show immediate feedback for long-running operations
- make all CLI commands feel like one coherent tool instead of separate text dumps
- keep `aim-core` reusable by a future GUI client
- avoid pushing terminal-specific logic into `aim-core`
- add real progress where possible and honest staged progress elsewhere

## Non-Goals

- a full-screen TUI
- async runtime migration across the entire application
- provider-specific live rich progress beyond what the current transport layers can expose cleanly
- redesigning core application behavior or registry semantics

## Architectural Decision

Use an event-driven CLI boundary.

`aim-core` will emit typed operation events for add, update, and remove flows. `aim-cli` will consume those events and render them using `console`, `indicatif`, and `dialoguer`.

This preserves the intended layering:

- `aim-core` owns workflow and operation semantics
- `aim-cli` owns prompts, colors, layout, spinners, and progress bars

## Command UX Shape

### `aim <QUERY>` add/install

This becomes the richest progress flow because it is the most visibly long-running command.

Target behavior:

- render an immediate spinner as soon as the operation begins
- update stage text as the operation advances through:
  - resolving source
  - discovering releases
  - selecting artifact
  - downloading artifact
  - staging payload
  - writing desktop entry
  - extracting icon
  - refreshing integration
  - saving registry
- when the transport can report total bytes, upgrade the spinner to a byte progress bar during download
- end with a styled success or failure summary

### `aim update`

Target behavior:

- show a batch-level progress indicator immediately
- emit per-app status rows as each update starts and completes
- show a styled final summary with updated count, failed count, and warnings

### `aim remove <QUERY>`

Target behavior:

- short-lived spinner while resolving and deleting managed files
- styled completion summary including warnings when integration refresh is degraded

### `aim list`

Target behavior:

- styled header and aligned entries
- proper empty state when no apps are registered

### bare `aim`

Target behavior:

- remain review-only
- render a styled update review summary instead of a raw single-line counter

## Prompt Strategy

`dialoguer` remains the prompt mechanism, but prompt rendering becomes centralized in `aim-cli`.

Design rules:

- use one shared prompt theme definition
- standardize prompt titles, selected-item labels, and cancel behavior
- render non-interactive fallback text using the same wording used in interactive mode

This keeps prompt copy and prompt appearance consistent across tracking selection and future artifact or app disambiguation prompts.

## Event Model

Add a small typed event model inside `aim-core`.

Recommended event families:

- `OperationStarted { kind, label }`
- `OperationStageChanged { stage, message }`
- `OperationProgress { current, total }`
- `OperationWarning { message }`
- `OperationFinished { summary }`
- `OperationFailed { stage, reason }`

Recommended operation kinds:

- add
- update-batch
- update-item
- remove

Recommended stages:

- resolve-query
- discover-release
- select-artifact
- download-artifact
- stage-payload
- write-desktop-entry
- extract-icon
- refresh-integration
- save-registry
- finalize

The event model must stay terminal-agnostic. It should not mention spinners, colors, or bars.

## Core Integration Strategy

The current blocking shape is:

- `aim-cli` dispatches
- `aim-core` completes all work synchronously
- `aim-cli` prints one final string

The new shape becomes:

- `aim-cli` constructs an operation reporter
- `aim-core` executes workflows while invoking a callback or reporter trait with typed events
- `aim-cli` renders events live and then prints the final styled summary

The first implementation should avoid invasive redesign beyond what is needed to surface events.

That means:

- keep existing workflow functions where practical
- add event-capable variants where needed
- refactor download and install helpers just enough to emit useful staged progress

## Crate Usage

### `dialoguer`

- keep for interactive prompts
- use a shared theme and prompt formatting helper

### `console`

- use for styled headers, labels, warnings, empty states, and final summaries
- centralize styling tokens in one CLI UI module instead of scattering style calls around render functions

### `indicatif`

- use spinners for staged operations
- use a progress bar for downloads when content length is known
- fall back to spinner plus stage text when byte totals are unavailable

## Testing Strategy

### Core tests

- verify add flow emits ordered stages for fixture-backed installs
- verify update flow emits per-app started and finished events
- verify remove flow emits resolve and cleanup stages
- verify progress events are optional and do not break fixture mode when total size is unavailable

### CLI tests

- assert styled output contains clearer headers and summary markers
- assert prompt text remains stable and intentional
- assert progress-aware commands still finish with the correct final summary

### Scope of snapshotting

Avoid over-snapshotting ANSI-heavy output. Prefer targeted assertions around:

- visible labels
- operation headings
- empty-state text
- summary wording
- warning wording

## Incremental Rollout

Implement in this order:

1. shared CLI styling primitives and prompt theme
2. typed core operation events
3. add/install live progress path
4. update batch progress path
5. remove/list/review restyling
6. README command and flow description refresh

This order fixes the biggest user-visible problem first while still delivering a complete CLI presentation pass in the same slice.