# Remove Uninstall Metadata Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Make `aim remove` uninstall `aim`-managed payload, desktop, and icon artifacts by persisting install metadata for new installs and falling back to derived managed paths for legacy registry entries.

**Architecture:** Extend `AppRecord` with optional install metadata, populate it from the successful install result, then upgrade `app/remove.rs` from registry filtering to an uninstall executor that resolves managed targets, deletes artifacts, refreshes desktop integration best-effort, and only then persists the updated registry. Keep `aim-cli` thin by rendering the richer remove outcome returned by `aim-core`.

**Tech Stack:** Rust, Cargo workspace, serde-backed registry persistence, std filesystem APIs, existing integration path and refresh helpers, fixture-backed tests in `crates/aim-core/tests` and `crates/aim-cli/tests`.

---

### Task 1: Persist install metadata in the registry model

**Files:**
- Modify: `crates/aim-core/src/domain/app.rs`
- Modify: `crates/aim-core/src/app/add.rs`
- Test: `crates/aim-core/tests/registry_roundtrip.rs`

**Step 1: Write the failing test**

Add a registry round-trip test that stores an app record with install metadata and asserts scope and file paths survive serialization.

**Step 2: Run test to verify it fails**

Run: `cargo test registry_round_trips_install_metadata --package aim-core --test registry_roundtrip`
Expected: FAIL because `AppRecord` does not yet have install metadata.

**Step 3: Write minimal implementation**

Add optional `InstallMetadata` and populate it from `InstalledApp` during add/install completion.

**Step 4: Run test to verify it passes**

Run: `cargo test registry_round_trips_install_metadata --package aim-core --test registry_roundtrip`
Expected: PASS.

**Step 5: Commit**

```bash
git add crates/aim-core/src/domain/app.rs crates/aim-core/src/app/add.rs crates/aim-core/tests/registry_roundtrip.rs
git commit -m "feat: persist install metadata for installed apps"
```

### Task 2: Add failing remove tests for uninstall behavior

**Files:**
- Modify: `crates/aim-core/tests/remove_flow.rs`
- Modify: `crates/aim-cli/tests/end_to_end_cli.rs`

**Step 1: Write the failing tests**

Add tests that assert:
- remove deletes persisted payload, desktop entry, and icon files
- remove falls back to derived managed paths for legacy records without install metadata
- CLI remove leaves no managed artifacts behind after add + remove

**Step 2: Run tests to verify they fail**

Run: `cargo test remove_deletes_installed_artifacts_from_metadata --package aim-core --test remove_flow`
Run: `cargo test remove_command_uninstalls_managed_files --package aim-cli --test end_to_end_cli`
Expected: FAIL because current remove only unregisters apps.

**Step 3: Keep tests minimal and precise**

Use fixture tempdirs and concrete file existence assertions. Avoid broad integration scaffolding beyond the exact managed artifact set.

**Step 4: Re-run to confirm the red state is correct**

Expected: still FAIL for missing uninstall behavior, not for unrelated setup issues.

**Step 5: Commit**

```bash
git add crates/aim-core/tests/remove_flow.rs crates/aim-cli/tests/end_to_end_cli.rs
git commit -m "test: cover uninstall behavior in remove flow"
```

### Task 3: Implement uninstall planning and execution in aim-core

**Files:**
- Modify: `crates/aim-core/src/app/remove.rs`
- Modify: `crates/aim-core/src/integration/refresh.rs`
- Modify: `crates/aim-core/src/integration/paths.rs`
- Test: `crates/aim-core/tests/remove_flow.rs`

**Step 1: Write the minimal implementation**

Implement:
- uninstall target resolution from persisted metadata
- derived fallback targets for legacy records
- deletion of managed payload, desktop entry, and icon
- deleted-path reporting
- best-effort refresh warnings after deletion

**Step 2: Run the focused core remove tests**

Run: `cargo test remove_deletes_installed_artifacts_from_metadata --package aim-core --test remove_flow`
Run: `cargo test remove_falls_back_to_derived_managed_paths --package aim-core --test remove_flow`
Expected: PASS.

**Step 3: Refine failure handling**

Ensure:
- missing files are ignored
- deletion IO failures stop removal
- refresh failures become warnings only

**Step 4: Re-run the remove test file**

Run: `cargo test --package aim-core --test remove_flow`
Expected: PASS.

**Step 5: Commit**

```bash
git add crates/aim-core/src/app/remove.rs crates/aim-core/src/integration/refresh.rs crates/aim-core/src/integration/paths.rs crates/aim-core/tests/remove_flow.rs
git commit -m "feat: uninstall managed artifacts during remove"
```

### Task 4: Surface uninstall results through the CLI

**Files:**
- Modify: `crates/aim-cli/src/lib.rs`
- Modify: `crates/aim-cli/src/ui/render.rs`
- Test: `crates/aim-cli/tests/end_to_end_cli.rs`

**Step 1: Write the minimal CLI integration**

Return and render uninstall details from `aim-core`, including warnings when refresh helpers are unavailable or fail.

**Step 2: Run the focused CLI tests**

Run: `cargo test remove_command_uninstalls_managed_files --package aim-cli --test end_to_end_cli`
Expected: PASS.

**Step 3: Preserve existing UX shape**

Keep the CLI thin and avoid duplicating uninstall logic in `aim-cli`.

**Step 4: Re-run the full CLI end-to-end test file**

Run: `cargo test --package aim-cli --test end_to_end_cli`
Expected: PASS.

**Step 5: Commit**

```bash
git add crates/aim-cli/src/lib.rs crates/aim-cli/src/ui/render.rs crates/aim-cli/tests/end_to_end_cli.rs
git commit -m "feat: render uninstall results from remove"
```

### Task 5: Run full workspace verification

**Files:**
- Modify: none expected
- Test: workspace-wide verification

**Step 1: Run formatting check**

Run: `cargo fmt --check`
Expected: PASS.

**Step 2: Run lints**

Run: `cargo clippy --workspace --all-targets --all-features -- -D warnings`
Expected: PASS.

**Step 3: Run tests**

Run: `cargo test --workspace`
Expected: PASS.

**Step 4: If any failure appears, fix only the uninstall-metadata related issue and re-run verification**

**Step 5: Commit**

```bash
git add -A
git commit -m "test: verify uninstall metadata remove flow"
```
