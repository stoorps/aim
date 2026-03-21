# Search Interactive TUI Design

## Summary

This change upgrades `aim search <QUERY>` from a plain text summary into an interactive terminal search flow when stdout and stdin are attached to a TTY. The interactive flow will use `ratatui` for a scrollable result browser, numeric multi-select inspired by `paru`, and an explicit confirmation step before install handoff.

The repository does not currently contain a checked-in config file, but the product already has a user-facing `config.toml` used for themes outside the repository tree. This design extends that existing config contract rather than introducing a second settings path.

## Goals

- Make `aim search <QUERY>` interactive by default on TTY.
- Render one result per row with clear columns for provider, repository, and install query.
- Support bottom-to-top list orientation by default, with config control.
- Support keyboard paging and numeric multi-select in the result browser.
- Show a confirmation step before install handoff unless disabled in config.
- Keep the search domain provider-extensible and avoid coupling core search ranking to terminal UI code.

## Non-Goals

- No `--json` output in this slice.
- No non-interactive rich formatting redesign beyond preserving a readable fallback.
- No general settings overhaul beyond the minimum config foundation needed to read existing theme settings and the new search keys together.

## Recommended Approach

### Option 1: Extend `dialoguer`

This keeps dependencies smaller, but it does not fit the requested behavior well. Paging, bottom-to-top layout, dense row rendering, and `paru`-style numeric selection would have to be simulated awkwardly across prompt screens.

### Option 2: Add a small config layer plus a dedicated `ratatui` search flow

This is the recommended approach. It creates a minimal, reusable settings boundary in `aim-cli`, keeps the current `aim-core` search contract intact, and gives the terminal UI enough control to implement the requested interaction model without twisting `dialoguer` into a pseudo-TUI.

### Option 3: Keep search plain text and launch a second prompt-only selection phase

This is simpler to ship, but it falls short of the requested UX and would likely be replaced immediately. It also duplicates state between the renderer and the selection prompt.

## Architecture

### Config

Add a lightweight config loader in `aim-cli` that reads the existing user `config.toml` location already used for theme settings. The loader should:

- tolerate a missing file by returning defaults
- ignore unknown keys
- treat malformed config as a CLI error with a clear path-aware message
- expose a typed `CliConfig` model for UI code

The search section should be:

```toml
[search]
bottom_to_top = true
skip_confirmation = false
```

This keeps search-specific settings namespaced and avoids a flat `skip_search_confirmation` key that will not scale once more search settings exist.

### Dispatch Flow

`aim search <QUERY>` should continue to build `SearchResults` through `aim-core`. `aim-cli` then chooses one of two render paths:

- TTY path: launch the interactive search browser
- non-TTY path: render the existing plain text summary

The interactive search browser should return one of three outcomes:

- cancelled
- confirmed selection set
- selection set that still requires explicit confirmation

Install execution is not part of this slice. The result of the search browser can remain a terminal-side selection artifact for now, but the code should be shaped so install handoff can be added without reworking the browser state machine.

### TUI Model

Add a dedicated module for interactive search state in `aim-cli`. It should own:

- the ordered result rows
- the highlighted cursor row
- the selected row indices
- the current page and viewport
- the row number buffer for `paru`-style typed numeric selection
- the config-driven orientation flag
- the confirmation mode state

Each visible row should stay one line tall. The row should include:

- numeric index
- provider label
- repository or package identity
- install-ready query

Warnings and installed matches should remain accessible, but the main browser should prioritize remote installable hits. If installed matches are shown in the interactive view, they should render in a distinct section or with a clear marker so they are not confused with remote install targets.

### Key Handling

The search browser should support:

- `j` / `k` and arrow keys for movement
- `Ctrl+d` / `Ctrl+u` or `PageDown` / `PageUp` for paging
- `g` / `G` for jump to top or bottom
- digit entry for numeric selection ranges and comma-separated values
- `Space` to toggle the highlighted row
- `Enter` to continue to confirmation
- `Esc` or `q` to cancel

Numeric selection should accept the same grammar throughout the session, for example `1`, `1,4,7`, and `3-6`. Invalid tokens should not panic; they should produce a small inline validation message and preserve the current selection.

### Confirmation

After the user leaves the browser with at least one selection, `aim-cli` should show a confirmation step by default. That step should summarise the chosen items and require an explicit yes/no confirmation.

If `[search].skip_confirmation = true`, the browser should return the chosen set immediately after selection finalization.

### Error Handling

- Missing or empty search results should not launch the browser; use the existing text renderer.
- Non-TTY stdin or stdout should not attempt `ratatui` initialization.
- Terminal initialization failure should fall back to plain text output rather than aborting the search command.
- Config parse failure should abort with a clear message because silent misconfiguration would be hard to debug.

## Testing Strategy

Follow TDD for each slice:

1. Add config parsing tests for defaults, valid search overrides, and malformed TOML.
2. Add state-machine tests for numeric selection parsing, paging, orientation, and confirmation transitions.
3. Add CLI tests covering TTY-gated fallback behavior and config-driven confirmation skipping.
4. Add a focused rendering test for one-line row formatting to keep the table stable.

Avoid end-to-end terminal snapshot tests that depend on terminal escape sequences unless state-level coverage proves insufficient.

## Delivery Notes

- Keep `aim-core` unchanged unless a search-install handoff type is clearly needed.
- Keep the existing plain text renderer for non-interactive contexts and future `--json` work.
- Preserve current theme behavior and make the new config loader the shared entry point for both theme settings and search settings.