# Search Interactive TUI Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add config-backed interactive search to `aim search <QUERY>` with a `ratatui` browser, numeric multi-select, paging, and an optional confirmation skip.

**Architecture:** Keep `aim-core` responsible for search retrieval and ranking. Add a small config loader plus a `ratatui`-backed state machine in `aim-cli`, and gate the interactive path on TTY availability with a safe plain-text fallback.

**Tech Stack:** Rust, clap, serde, toml, ratatui, crossterm, assert_cmd

---

### Task 1: Add CLI config loading for search settings

**Files:**
- Create: `crates/aim-cli/src/config.rs`
- Modify: `crates/aim-cli/src/lib.rs`
- Modify: `crates/aim-cli/src/main.rs`
- Test: `crates/aim-cli/tests/config_loading.rs`

**Step 1: Write the failing test**

Add tests covering:

- missing config returns defaults
- valid `[search]` config overrides defaults
- malformed TOML returns a path-aware error

**Step 2: Run test to verify it fails**

Run: `cargo test --package aim-cli --test config_loading`
Expected: FAIL because `config.rs` and the config loader do not exist.

**Step 3: Write minimal implementation**

Implement a typed `CliConfig` with nested `SearchConfig` and the minimum loader API needed by `aim-cli`.

Defaults:

```rust
SearchConfig {
    bottom_to_top: true,
    skip_confirmation: false,
}
```

The loader must tolerate a missing file and reject malformed TOML with the resolved path in the error.

**Step 4: Run test to verify it passes**

Run: `cargo test --package aim-cli --test config_loading`
Expected: PASS

### Task 2: Add a search browser state machine

**Files:**
- Create: `crates/aim-cli/src/ui/search_browser.rs`
- Modify: `crates/aim-cli/src/ui/mod.rs`
- Test: `crates/aim-cli/tests/search_browser.rs`

**Step 1: Write the failing test**

Add state-level tests for:

- bottom-to-top ordering default
- cursor movement and page movement
- single index selection
- comma-separated and range numeric selection
- invalid numeric input preserving current selection
- confirmation state transitions

**Step 2: Run test to verify it fails**

Run: `cargo test --package aim-cli --test search_browser`
Expected: FAIL because the browser state module does not exist.

**Step 3: Write minimal implementation**

Build a pure Rust state model that does not require a live terminal to test. Keep terminal drawing and key-event adaptation separate from selection and pagination logic.

**Step 4: Run test to verify it passes**

Run: `cargo test --package aim-cli --test search_browser`
Expected: PASS

### Task 3: Wire `ratatui` and TTY-gated interactive search dispatch

**Files:**
- Modify: `crates/aim-cli/Cargo.toml`
- Modify: `crates/aim-cli/src/lib.rs`
- Modify: `crates/aim-cli/src/ui/render.rs`
- Modify: `crates/aim-cli/src/main.rs`
- Modify: `crates/aim-cli/src/ui/prompt.rs`
- Test: `crates/aim-cli/tests/search_cli.rs`

**Step 1: Write the failing test**

Add CLI coverage for:

- non-TTY search stays plain text
- config skip confirmation changes the post-selection path
- empty result sets do not launch the browser

Use deterministic seams rather than a full escape-sequence snapshot test.

**Step 2: Run test to verify it fails**

Run: `cargo test --package aim-cli --test search_cli`
Expected: FAIL because interactive search dispatch is not implemented.

**Step 3: Write minimal implementation**

Add `ratatui` and `crossterm`, initialize the browser only when stdin and stdout are terminals, and fall back cleanly to the existing renderer if terminal setup fails or the result set is empty.

**Step 4: Run test to verify it passes**

Run: `cargo test --package aim-cli --test search_cli`
Expected: PASS

### Task 4: Add row rendering and confirmation summary coverage

**Files:**
- Modify: `crates/aim-cli/src/ui/search_browser.rs`
- Test: `crates/aim-cli/tests/ui_summary.rs`

**Step 1: Write the failing test**

Add focused tests for:

- one-line row formatting
- provider and query columns remaining visible
- confirmation summary content for multi-select

**Step 2: Run test to verify it fails**

Run: `cargo test --package aim-cli --test ui_summary`
Expected: FAIL because the browser summaries are not rendered yet.

**Step 3: Write minimal implementation**

Implement the smallest formatting helpers needed to keep rows stable and the confirmation screen legible.

**Step 4: Run test to verify it passes**

Run: `cargo test --package aim-cli --test ui_summary`
Expected: PASS

### Task 5: Final verification

**Files:**
- Modify as required by prior tasks only

**Step 1: Run focused CLI tests**

Run: `cargo test --package aim-cli --test config_loading --test search_browser --test search_cli --test ui_summary`
Expected: PASS

**Step 2: Run workspace formatting**

Run: `cargo fmt --all`
Expected: PASS

**Step 3: Run workspace linting and regression tests**

Run: `cargo test --workspace && cargo clippy --workspace --all-targets --all-features -- -D warnings`
Expected: PASS