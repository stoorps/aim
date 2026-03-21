# Show Command And Update Rollback Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add a read-only `aim show <value>` command that inspects installed apps first and falls back to remote source resolution, while making `aim update` restore the previous installation files when reinstall fails.

**Architecture:** Add a small `aim-core` show service that returns either installed details or resolved add-plan details, then wire a single CLI subcommand and text renderer around that result. Harden update execution in `aim-core` by staging tracked install files before reinstall and restoring them on failure without introducing new registry state.

**Tech Stack:** Rust, clap, serde, toml, tempfile, assert_cmd

---

### Task 1: Add the core `show` domain model and resolution service

**Files:**
- Create: `crates/aim-core/src/app/show.rs`
- Create: `crates/aim-core/src/domain/show.rs`
- Modify: `crates/aim-core/src/app/mod.rs`
- Modify: `crates/aim-core/src/domain/mod.rs`
- Test: `crates/aim-core/tests/show_resolution.rs`

**Step 1: Write the failing test**

Add core tests covering:

- one installed match returns installed details
- no installed match falls back to remote resolution
- ambiguous installed matches return a dedicated error
- unsupported query stays distinct from `NoInstallableArtifact`

Include at least one remote-resolution fixture that proves the `show` result carries artifact URL, selection reason, and trusted checksum.

**Step 2: Run test to verify it fails**

Run: `cargo test --package aim-core --test show_resolution`
Expected: FAIL because the show domain and service do not exist.

**Step 3: Write minimal implementation**

Implement a new `show` service in `aim-core` that accepts the user input and installed app list, applies installed-first resolution, and returns a typed `ShowResult`.

Keep the installed branch focused on existing `AppRecord` data. Keep the remote branch focused on summarizing the already-built add plan instead of exposing all of `AddPlan` directly.

**Step 4: Run test to verify it passes**

Run: `cargo test --package aim-core --test show_resolution`
Expected: PASS

### Task 2: Wire the `show` command through `aim-cli`

**Files:**
- Modify: `crates/aim-cli/src/cli/args.rs`
- Modify: `crates/aim-cli/src/lib.rs`
- Modify: `crates/aim-cli/src/ui/render.rs`
- Test: `crates/aim-cli/tests/cli_commands.rs`
- Test: `crates/aim-cli/tests/ui_summary.rs`

**Step 1: Write the failing test**

Add CLI coverage for:

- `aim show legacy-bat` dispatching successfully and rendering installed details
- `aim show owner/repo` rendering resolved source and artifact details
- ambiguous installed lookup rendering a readable failure

Keep the rendering assertions focused on stable summary lines rather than exact spacing across the entire output block.

**Step 2: Run test to verify it fails**

Run: `cargo test --package aim-cli --test cli_commands --test ui_summary`
Expected: FAIL because the CLI has no `show` command or renderer.

**Step 3: Write minimal implementation**

Add `Show { value: String }` to the clap subcommands, route it through dispatch, convert core `ShowResult` into a new `DispatchResult::Show(...)` variant, and render installed and remote summaries in `ui::render`.

Do not add prompting or mutation to this command.

**Step 4: Run test to verify it passes**

Run: `cargo test --package aim-cli --test cli_commands --test ui_summary`
Expected: PASS

### Task 3: Add rollback staging for update execution

**Files:**
- Modify: `crates/aim-core/src/app/update.rs`
- Modify: `crates/aim-core/src/domain/update.rs`
- Test: `crates/aim-core/tests/update_planning.rs`

**Step 1: Write the failing test**

Add update execution tests covering:

- a failed reinstall restores the original payload file contents
- a failed reinstall keeps returning the previous `AppRecord`
- a successful reinstall removes any rollback staging directory

Use a temporary install home and deterministic fixture inputs so the test does not depend on external services.

**Step 2: Run test to verify it fails**

Run: `cargo test --package aim-core --test update_planning`
Expected: FAIL because update execution does not back up or restore tracked files.

**Step 3: Write minimal implementation**

Add a small rollback helper inside `update.rs` that gathers the tracked install paths, moves existing files into a staging directory under the install home, restores them on failure, and deletes the staging directory on success.

Only enrich `domain::update` if you need a better warning or failure surface for tests and summaries.

**Step 4: Run test to verify it passes**

Run: `cargo test --package aim-core --test update_planning`
Expected: PASS

### Task 4: Cover desktop integration rollback and human-facing failure output

**Files:**
- Modify: `crates/aim-core/src/app/update.rs`
- Modify: `crates/aim-cli/src/ui/render.rs`
- Test: `crates/aim-core/tests/install_failures.rs`
- Test: `crates/aim-cli/tests/end_to_end_cli.rs`

**Step 1: Write the failing test**

Add coverage for:

- update rollback restoring desktop entry and icon files when replacement install fails after file moves
- CLI update summary surfacing a rollback-aware failure reason instead of a generic install error

Use temporary directories and existing fixture-style test seams.

**Step 2: Run test to verify it fails**

Run: `cargo test --package aim-core --test install_failures && cargo test --package aim-cli --test end_to_end_cli`
Expected: FAIL because desktop integration rollback is not restored and the failure output is not rollback-aware.

**Step 3: Write minimal implementation**

Extend the rollback helper to include tracked desktop integration paths and surface a clear failure reason when either backup creation or restore fails.

Keep the CLI output change small: reuse the existing update summary renderer and only improve the failure string content.

**Step 4: Run test to verify it passes**

Run: `cargo test --package aim-core --test install_failures && cargo test --package aim-cli --test end_to_end_cli`
Expected: PASS

### Task 5: Final verification

**Files:**
- Modify as required by prior tasks only

**Step 1: Run focused feature tests**

Run: `cargo test --package aim-core --test show_resolution --test update_planning --test install_failures && cargo test --package aim-cli --test cli_commands --test ui_summary --test end_to_end_cli`
Expected: PASS

**Step 2: Run workspace formatting**

Run: `cargo fmt --all`
Expected: PASS

**Step 3: Run workspace regression and lint checks**

Run: `cargo test --workspace && cargo clippy --workspace --all-targets --all-features -- -D warnings`
Expected: PASS