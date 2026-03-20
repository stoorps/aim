# Update Execution And Provider Contract Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Make `aim update` actually apply updates through the existing install engine and tighten the adapter trait so providers share required normalize and resolve operations.

**Architecture:** Add an update executor in `aim-core` that rebuilds install intent from each registered app and reuses `build_add_plan(...)` plus `install_app(...)` for the actual replacement. In parallel, extend the `SourceAdapter` trait with `normalize` and `resolve`, add a shared adapter error type, and migrate GitHub and GitLab adapters to that contract.

**Tech Stack:** Rust, Cargo workspace, existing install engine in `aim-core`, serde registry state, clap CLI, fixture-backed tests in `crates/aim-core/tests` and `crates/aim-cli/tests`.

---

### Task 1: Add red tests for update execution

**Files:**
- Modify: `crates/aim-cli/tests/end_to_end_cli.rs`
- Modify: `crates/aim-core/tests/update_planning.rs`

**Step 1: Write the failing tests**

Add focused tests that assert:
- `aim update` updates installed apps rather than only rendering a summary
- update failures retain the previous app record

**Step 2: Run tests to verify they fail**

Run: `cargo test update_command_applies_updates --package aim-cli --test end_to_end_cli`
Expected: FAIL because `aim update` only reviews.

**Step 3: Keep the tests minimal**

Use fixture mode and registry tempdirs. Assert on command output and registry state.

**Step 4: Re-run to confirm clean red state**

Expected: FAIL for missing update execution, not setup issues.

**Step 5: Commit**

```bash
git add crates/aim-cli/tests/end_to_end_cli.rs crates/aim-core/tests/update_planning.rs
git commit -m "test: cover executable update flow"
```

### Task 2: Implement update execution in aim-core

**Files:**
- Modify: `crates/aim-core/src/app/update.rs`
- Modify: `crates/aim-core/src/app/add.rs`
- Modify: `crates/aim-core/src/domain/update.rs`

**Step 1: Write minimal implementation**

Implement per-app update execution that:
- reconstructs query from stored app data
- determines install scope from persisted install metadata
- reuses `build_add_plan(...)` and `install_app(...)`
- keeps previous app records on failure
- returns structured execution results

**Step 2: Run focused tests**

Run: `cargo test update_command_applies_updates --package aim-cli --test end_to_end_cli`
Expected: PASS.

**Step 3: Add failure reporting**

Capture updated count, failed count, and per-app messages.

**Step 4: Re-run the update-related tests**

Run: `cargo test --package aim-core --test update_planning`
Expected: PASS.

**Step 5: Commit**

```bash
git add crates/aim-core/src/app/update.rs crates/aim-core/src/app/add.rs crates/aim-core/src/domain/update.rs crates/aim-core/tests/update_planning.rs crates/aim-cli/tests/end_to_end_cli.rs
git commit -m "feat: execute updates through install engine"
```

### Task 3: Wire update execution through the CLI

**Files:**
- Modify: `crates/aim-cli/src/cli/args.rs`
- Modify: `crates/aim-cli/src/lib.rs`
- Modify: `crates/aim-cli/src/ui/render.rs`
- Test: `crates/aim-cli/tests/end_to_end_cli.rs`

**Step 1: Implement CLI execution behavior**

Keep no-arg review behavior, but make `aim update` execute the updates and render a result summary.

**Step 2: Run focused CLI tests**

Run: `cargo test update_command_applies_updates --package aim-cli --test end_to_end_cli`
Expected: PASS.

**Step 3: Preserve review-only path**

Ensure plain `aim` with no args still shows the update review summary.

**Step 4: Re-run the CLI end-to-end test file**

Run: `cargo test --package aim-cli --test end_to_end_cli`
Expected: PASS.

**Step 5: Commit**

```bash
git add crates/aim-cli/src/cli/args.rs crates/aim-cli/src/lib.rs crates/aim-cli/src/ui/render.rs crates/aim-cli/tests/end_to_end_cli.rs
git commit -m "feat: run updates from cli update command"
```

### Task 4: Tighten the provider contract

**Files:**
- Modify: `crates/aim-core/src/adapters/traits.rs`
- Modify: `crates/aim-core/src/adapters/github.rs`
- Modify: `crates/aim-core/src/adapters/gitlab.rs`
- Modify: `crates/aim-core/tests/adapter_contract.rs`

**Step 1: Write the failing tests**

Add contract tests that verify GitHub and GitLab implement normalize and resolve through the shared `SourceAdapter` trait.

**Step 2: Run tests to verify they fail**

Run: `cargo test --package aim-core --test adapter_contract`
Expected: FAIL because the trait does not yet require those methods.

**Step 3: Write minimal implementation**

Add:
- shared adapter error enum
- `normalize` and `resolve` trait methods
- GitHub and GitLab implementations under that shared contract

**Step 4: Run adapter contract tests**

Run: `cargo test --package aim-core --test adapter_contract`
Expected: PASS.

**Step 5: Commit**

```bash
git add crates/aim-core/src/adapters/traits.rs crates/aim-core/src/adapters/github.rs crates/aim-core/src/adapters/gitlab.rs crates/aim-core/tests/adapter_contract.rs
git commit -m "refactor: tighten provider adapter contract"
```

### Task 5: Full verification

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

**Step 4: Fix only update-execution and provider-contract regressions if needed and re-run verification**

**Step 5: Commit**

```bash
git add -A
git commit -m "test: verify update execution and provider contract"
```
