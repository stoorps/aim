# CLI UX And Progress Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Redesign the terminal UX across all CLI commands so prompts use `dialoguer`, summaries use `console`, and long-running operations show live `indicatif` progress driven by typed events from `aim-core`.

**Architecture:** Add a terminal-agnostic operation event model in `aim-core`, thread it through add, update, and remove workflows, then render those events in `aim-cli` through centralized styling and progress helpers. Keep the current business logic in `aim-core` and keep all terminal behavior in `aim-cli`.

**Tech Stack:** Rust, Cargo workspace, clap, dialoguer, console, indicatif, reqwest blocking client, existing fixture-backed tests in `crates/aim-core/tests` and `crates/aim-cli/tests`.

---

### Task 1: Add failing CLI presentation tests

**Files:**
- Modify: `crates/aim-cli/tests/end_to_end_cli.rs`
- Modify: `crates/aim-cli/tests/ui_summary.rs`
- Modify: `crates/aim-cli/Cargo.toml`
- Modify: `Cargo.toml`

**Step 1: Write the failing tests**

Add focused tests that assert:
- add/install output uses clearer sectioned summary wording instead of the current raw line bundle
- list empty state and update review output use improved labels
- prompt rendering keeps explicit tracking wording while moving to shared prompt formatting

**Step 2: Run tests to verify they fail**

Run: `cargo test --package aim-cli --test ui_summary`
Expected: FAIL because the current renderer still emits plain legacy text.

**Step 3: Add the missing CLI dependencies**

Add `console` and `indicatif` to the workspace and `aim-cli` crate so the new UI helpers can be implemented cleanly.

**Step 4: Re-run the focused tests**

Run: `cargo test --package aim-cli --test ui_summary`
Expected: still FAIL for behavior, not setup.

**Step 5: Commit**

```bash
git add Cargo.toml crates/aim-cli/Cargo.toml crates/aim-cli/tests/end_to_end_cli.rs crates/aim-cli/tests/ui_summary.rs
git commit -m "test: cover cli presentation refresh"
```

### Task 2: Add typed operation events in aim-core

**Files:**
- Create: `crates/aim-core/src/app/progress.rs`
- Modify: `crates/aim-core/src/app/mod.rs`
- Modify: `crates/aim-core/src/app/add.rs`
- Modify: `crates/aim-core/src/app/update.rs`
- Modify: `crates/aim-core/src/app/remove.rs`
- Test: `crates/aim-core/tests/install_integration.rs`
- Test: `crates/aim-core/tests/update_planning.rs`
- Test: `crates/aim-core/tests/remove_flow.rs`

**Step 1: Write the failing core tests**

Add tests that assert event order for:
- add/install fixture flow
- update execution over one app
- remove flow for managed artifacts

**Step 2: Run the focused tests to verify they fail**

Run: `cargo test --package aim-core --test install_integration`
Expected: FAIL because no operation event model exists.

**Step 3: Implement the minimal event model**

Add:
- typed operation and stage enums
- event payload structs
- a reporter callback or trait that defaults to no-op when not supplied

Do not introduce terminal dependencies into `aim-core`.

**Step 4: Thread events through add, update, and remove flows**

Use event-capable variants or optional reporter parameters so existing business logic stays intact.

**Step 5: Run the focused core tests**

Run: `cargo test --package aim-core --test install_integration`
Expected: PASS.

**Step 6: Commit**

```bash
git add crates/aim-core/src/app/progress.rs crates/aim-core/src/app/mod.rs crates/aim-core/src/app/add.rs crates/aim-core/src/app/update.rs crates/aim-core/src/app/remove.rs crates/aim-core/tests/install_integration.rs crates/aim-core/tests/update_planning.rs crates/aim-core/tests/remove_flow.rs
git commit -m "feat: add core operation progress events"
```

### Task 3: Emit real add/install progress, including download progress when possible

**Files:**
- Modify: `crates/aim-core/src/app/add.rs`
- Modify: `crates/aim-core/src/integration/install.rs`
- Test: `crates/aim-core/tests/install_integration.rs`
- Test: `crates/aim-cli/tests/end_to_end_cli.rs`

**Step 1: Write the failing tests**

Add tests that assert the add/install path emits:
- stage transitions before final success
- download progress events when content length is known or simulated
- fallback stage events when byte totals are unavailable

**Step 2: Run the focused tests to verify they fail**

Run: `cargo test --package aim-core --test install_integration`
Expected: FAIL because download and install helpers do not yet emit progress.

**Step 3: Implement minimal progress emission**

Refactor download and install helpers just enough to report:
- download started
- download byte progress when available
- payload staging
- desktop integration
- icon extraction
- refresh and finalize

Keep the blocking transport for now.

**Step 4: Run focused tests**

Run: `cargo test --package aim-core --test install_integration`
Expected: PASS.

**Step 5: Commit**

```bash
git add crates/aim-core/src/app/add.rs crates/aim-core/src/integration/install.rs crates/aim-core/tests/install_integration.rs crates/aim-cli/tests/end_to_end_cli.rs
git commit -m "feat: emit live add install progress"
```

### Task 4: Build shared CLI styling and prompt primitives

**Files:**
- Create: `crates/aim-cli/src/ui/theme.rs`
- Create: `crates/aim-cli/src/ui/progress.rs`
- Modify: `crates/aim-cli/src/ui/mod.rs`
- Modify: `crates/aim-cli/src/ui/prompt.rs`
- Modify: `crates/aim-cli/src/ui/render.rs`
- Test: `crates/aim-cli/tests/ui_summary.rs`

**Step 1: Write the failing tests**

Add focused tests that assert:
- section headers and empty states use new wording
- warnings are rendered consistently
- prompt copy still contains the tracking choice details

**Step 2: Run the focused tests to verify they fail**

Run: `cargo test --package aim-cli --test ui_summary`
Expected: FAIL because there is no shared styling layer.

**Step 3: Implement minimal CLI UI helpers**

Add:
- shared console styles and header helpers
- shared prompt theme for `dialoguer`
- a thin indicatif wrapper for spinners and progress bars

**Step 4: Migrate prompt and summary rendering onto the shared helpers**

Do not change core logic in this step.

**Step 5: Re-run the focused tests**

Run: `cargo test --package aim-cli --test ui_summary`
Expected: PASS.

**Step 6: Commit**

```bash
git add crates/aim-cli/src/ui/theme.rs crates/aim-cli/src/ui/progress.rs crates/aim-cli/src/ui/mod.rs crates/aim-cli/src/ui/prompt.rs crates/aim-cli/src/ui/render.rs crates/aim-cli/tests/ui_summary.rs
git commit -m "feat: add shared cli styling and prompt helpers"
```

### Task 5: Wire live progress rendering through CLI dispatch

**Files:**
- Modify: `crates/aim-cli/src/lib.rs`
- Modify: `crates/aim-cli/src/main.rs`
- Modify: `crates/aim-cli/src/ui/progress.rs`
- Test: `crates/aim-cli/tests/end_to_end_cli.rs`

**Step 1: Write the failing tests**

Add tests that assert:
- add/install commands emit status output before the final summary marker
- update execution emits per-app progress lines and a final styled summary
- remove emits status plus final completion output

**Step 2: Run the focused tests to verify they fail**

Run: `cargo test --package aim-cli --test end_to_end_cli`
Expected: FAIL because dispatch still returns only a final renderable result.

**Step 3: Implement minimal dispatch integration**

Thread a CLI reporter through dispatch so long-running operations can stream progress while preserving the final typed result for summary rendering.

**Step 4: Re-run the CLI tests**

Run: `cargo test --package aim-cli --test end_to_end_cli`
Expected: PASS.

**Step 5: Commit**

```bash
git add crates/aim-cli/src/lib.rs crates/aim-cli/src/main.rs crates/aim-cli/src/ui/progress.rs crates/aim-cli/tests/end_to_end_cli.rs
git commit -m "feat: render live cli progress"
```

### Task 6: Restyle all command summaries and refresh documentation

**Files:**
- Modify: `crates/aim-cli/src/ui/render.rs`
- Modify: `README.md`
- Test: `crates/aim-cli/tests/end_to_end_cli.rs`
- Test: `crates/aim-cli/tests/ui_summary.rs`

**Step 1: Write the failing assertions**

Add or tighten tests for:
- list empty state and populated list presentation
- remove completion summary
- update review summary for bare `aim`
- README text matching actual `aim` versus `aim update` behavior

**Step 2: Run focused tests**

Run: `cargo test --package aim-cli --test end_to_end_cli`
Expected: FAIL for updated wording.

**Step 3: Implement minimal rendering refresh**

Use the shared theme helpers so every command output looks coherent.

**Step 4: Re-run focused tests**

Run: `cargo test --package aim-cli --test end_to_end_cli`
Expected: PASS.

**Step 5: Commit**

```bash
git add crates/aim-cli/src/ui/render.rs README.md crates/aim-cli/tests/end_to_end_cli.rs crates/aim-cli/tests/ui_summary.rs
git commit -m "docs: refresh cli command presentation"
```

### Task 7: Full verification

**Files:**
- Modify: none expected

**Step 1: Run format check**

Run: `cargo fmt --check`
Expected: PASS.

**Step 2: Run clippy**

Run: `cargo clippy --workspace --all-targets --all-features -- -D warnings`
Expected: PASS.

**Step 3: Run tests**

Run: `cargo test --workspace`
Expected: PASS.

**Step 4: Fix only regressions caused by the CLI UX and progress work, then re-run verification**

**Step 5: Commit**

```bash
git add -A
git commit -m "test: verify cli ux and progress redesign"
```