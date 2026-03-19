# Per-Distro Installation Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Implement real AppImage installation in `aim-core` with distro-aware policy resolution, transactional payload and desktop integration, and CLI surfacing of resolved install mode and warnings.

**Architecture:** Add a host detection and policy layer ahead of the existing install scaffolding, then turn `integration/install.rs` into a transactional executor that stages payloads, commits managed artifacts atomically, writes desktop integration into policy-selected locations, and only persists registry state after success. Keep distro-specific behavior declarative through `DistroFamily`, `HostCapabilities`, and `InstallPolicy` instead of branching throughout the pipeline.

**Tech Stack:** Rust, Cargo workspace, std filesystem APIs, existing `aim-core` domain and registry types, `clap`, `dialoguer`, fixture-backed tests in `crates/aim-core/tests` and `crates/aim-cli/tests`.

---

### Task 1: Add distro family detection and host capability probing

**Files:**
- Create: `crates/aim-core/src/platform/distro.rs`
- Create: `crates/aim-core/src/platform/capabilities.rs`
- Modify: `crates/aim-core/src/platform/mod.rs`
- Test: `crates/aim-core/tests/platform_detection.rs`

**Step 1: Write the failing test**

```rust
use aim_core::platform::distro::{detect_distro_family, DistroFamily};

#[test]
fn detects_fedora_family_from_os_release() {
    let distro = detect_distro_family("ID=fedora\nID_LIKE=rhel centos\n");
    assert_eq!(distro, DistroFamily::Fedora);
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test detects_fedora_family_from_os_release --package aim-core --test platform_detection`
Expected: FAIL because distro detection types do not exist yet

**Step 3: Write minimal implementation**

Add:
- `DistroFamily`
- `/etc/os-release` parsing helpers
- immutable and Nix policy markers
- helper availability probing for desktop refresh commands
- directory writability probing for candidate install roots

Keep the probing interfaces small and deterministic so tests can inject fake host facts.

**Step 4: Run test to verify it passes**

Run: `cargo test detects_fedora_family_from_os_release --package aim-core --test platform_detection`
Expected: PASS

**Step 5: Commit**

```bash
git add crates/aim-core/src/platform/distro.rs crates/aim-core/src/platform/capabilities.rs crates/aim-core/src/platform/mod.rs crates/aim-core/tests/platform_detection.rs
git commit -m "feat: add distro and capability detection"
```

### Task 2: Introduce install policy resolution

**Files:**
- Create: `crates/aim-core/src/integration/policy.rs`
- Modify: `crates/aim-core/src/integration/mod.rs`
- Modify: `crates/aim-core/src/platform/mod.rs`
- Test: `crates/aim-core/tests/install_policy.rs`

**Step 1: Write the failing test**

```rust
use aim_core::integration::policy::{resolve_install_policy, IntegrationMode};
use aim_core::platform::{DistroFamily, HostCapabilities, InstallScope};

#[test]
fn immutable_system_request_downgrades_to_user_when_allowed() {
    let capabilities = HostCapabilities::immutable_user_only();
    let policy = resolve_install_policy(DistroFamily::Immutable, InstallScope::System, &capabilities).unwrap();

    assert_eq!(policy.scope, InstallScope::User);
    assert_eq!(policy.integration_mode, IntegrationMode::Degraded);
    assert!(!policy.warnings.is_empty());
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test immutable_system_request_downgrades_to_user_when_allowed --package aim-core --test install_policy`
Expected: FAIL because install policy resolution does not exist yet

**Step 3: Write minimal implementation**

Create:
- `InstallPolicy`
- `IntegrationMode`
- policy resolution for the agreed distro families
- separation of payload, desktop, and icon roots
- warning collection for downgraded or conservative behavior

Implement only the current agreed rules. Do not add speculative distro exceptions.

**Step 4: Run test to verify it passes**

Run: `cargo test immutable_system_request_downgrades_to_user_when_allowed --package aim-core --test install_policy`
Expected: PASS

**Step 5: Commit**

```bash
git add crates/aim-core/src/integration/policy.rs crates/aim-core/src/integration/mod.rs crates/aim-core/src/platform/mod.rs crates/aim-core/tests/install_policy.rs
git commit -m "feat: resolve per-distro install policy"
```

### Task 3: Turn install scaffolding into a staged payload executor

**Files:**
- Modify: `crates/aim-core/src/integration/install.rs`
- Modify: `crates/aim-core/src/integration/paths.rs`
- Modify: `crates/aim-core/src/app/add.rs`
- Test: `crates/aim-core/tests/install_payload.rs`

**Step 1: Write the failing test**

```rust
use aim_core::integration::install::stage_and_commit_payload;

#[test]
fn payload_commit_moves_staged_appimage_into_final_location() {
    let outcome = stage_and_commit_payload(/* fixture inputs */).unwrap();
    assert!(outcome.final_payload_path.ends_with(".AppImage"));
    assert!(outcome.final_payload_path.exists());
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test payload_commit_moves_staged_appimage_into_final_location --package aim-core --test install_payload`
Expected: FAIL because install execution still only exposes path helpers

**Step 3: Write minimal implementation**

Implement:
- staging download target creation
- AppImage validation hook
- executable bit application in staging
- atomic payload replacement into the managed payload root
- minimal rollback for payload commit failure

Do not write registry state yet.

**Step 4: Run test to verify it passes**

Run: `cargo test payload_commit_moves_staged_appimage_into_final_location --package aim-core --test install_payload`
Expected: PASS

**Step 5: Commit**

```bash
git add crates/aim-core/src/integration/install.rs crates/aim-core/src/integration/paths.rs crates/aim-core/src/app/add.rs crates/aim-core/tests/install_payload.rs
git commit -m "feat: add staged payload install executor"
```

### Task 4: Add desktop integration and refresh handling

**Files:**
- Create: `crates/aim-core/src/integration/desktop.rs`
- Create: `crates/aim-core/src/integration/refresh.rs`
- Modify: `crates/aim-core/src/integration/install.rs`
- Modify: `crates/aim-core/src/integration/mod.rs`
- Test: `crates/aim-core/tests/install_integration.rs`

**Step 1: Write the failing test**

```rust
use aim_core::integration::install::execute_install;

#[test]
fn install_writes_desktop_entry_and_reports_refresh_warning_only() {
    let outcome = execute_install(/* fixture with missing helper */).unwrap();

    assert!(outcome.desktop_entry_path.unwrap().exists());
    assert!(!outcome.warnings.is_empty());
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test install_writes_desktop_entry_and_reports_refresh_warning_only --package aim-core --test install_integration`
Expected: FAIL because desktop integration and refresh steps do not exist yet

**Step 3: Write minimal implementation**

Add:
- `.desktop` generation from normalized metadata
- icon extraction and placement hooks
- refresh action planning
- best-effort helper execution for desktop database and icon cache refresh
- rollback of generated integration files when required integration fails

Keep helper execution optional and warning-driven.

**Step 4: Run test to verify it passes**

Run: `cargo test install_writes_desktop_entry_and_reports_refresh_warning_only --package aim-core --test install_integration`
Expected: PASS

**Step 5: Commit**

```bash
git add crates/aim-core/src/integration/desktop.rs crates/aim-core/src/integration/refresh.rs crates/aim-core/src/integration/install.rs crates/aim-core/src/integration/mod.rs crates/aim-core/tests/install_integration.rs
git commit -m "feat: add desktop integration and refresh handling"
```

### Task 5: Persist registry state only after successful install and surface policy in the CLI

**Files:**
- Modify: `crates/aim-core/src/app/add.rs`
- Modify: `crates/aim-core/src/registry/mod.rs`
- Modify: `crates/aim-cli/src/lib.rs`
- Modify: `crates/aim-cli/src/ui/prompt.rs`
- Modify: `crates/aim-cli/src/ui/render.rs`
- Test: `crates/aim-cli/tests/end_to_end_cli.rs`

**Step 1: Write the failing test**

```rust
#[test]
fn cli_add_installs_and_renders_resolved_mode() {
    let output = run_cli_add(/* fixture query */);

    assert!(output.contains("installing as user"));
    assert!(output.contains("installed app:"));
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test cli_add_installs_and_renders_resolved_mode --package aim-cli --test end_to_end_cli`
Expected: FAIL because the CLI still performs registry-backed tracking only

**Step 3: Write minimal implementation**

Change the add flow so it:
- builds an install plan instead of only a tracking record
- prints the resolved install mode and warnings before commit
- persists the final `AppRecord` only after successful install completion
- renders installed outcomes rather than tracked-only outcomes

Preserve review prompts where source ambiguity still exists.

**Step 4: Run test to verify it passes**

Run: `cargo test cli_add_installs_and_renders_resolved_mode --package aim-cli --test end_to_end_cli`
Expected: PASS

**Step 5: Commit**

```bash
git add crates/aim-core/src/app/add.rs crates/aim-core/src/registry/mod.rs crates/aim-cli/src/lib.rs crates/aim-cli/src/ui/prompt.rs crates/aim-cli/src/ui/render.rs crates/aim-cli/tests/end_to_end_cli.rs
git commit -m "feat: execute installs from cli add flow"
```

### Task 6: Lock down rollback and failure semantics

**Files:**
- Modify: `crates/aim-core/src/integration/install.rs`
- Modify: `crates/aim-core/src/integration/desktop.rs`
- Test: `crates/aim-core/tests/install_failures.rs`
- Modify: `README.md`

**Step 1: Write the failing test**

```rust
use aim_core::integration::install::execute_install;

#[test]
fn integration_failure_removes_new_payload_and_generated_files() {
    let error = execute_install(/* fixture with forced desktop write failure */).unwrap_err();

    assert!(error.to_string().contains("desktop integration failed"));
    assert_install_root_is_clean();
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test integration_failure_removes_new_payload_and_generated_files --package aim-core --test install_failures`
Expected: FAIL because rollback behavior is not complete yet

**Step 3: Write minimal implementation**

Finish:
- rollback of newly committed payloads on required integration failure
- cleanup of generated desktop and icon artifacts
- warning-only handling for refresh failures
- README updates describing actual install behavior and degraded cases

Do not broaden feature scope beyond the approved design.

**Step 4: Run test to verify it passes**

Run: `cargo test integration_failure_removes_new_payload_and_generated_files --package aim-core --test install_failures`
Expected: PASS

**Step 5: Commit**

```bash
git add crates/aim-core/src/integration/install.rs crates/aim-core/src/integration/desktop.rs crates/aim-core/tests/install_failures.rs README.md
git commit -m "feat: finalize install rollback behavior"
```

### Task 7: Run full workspace verification

**Files:**
- Modify: none unless verification exposes regressions tied to the approved scope

**Step 1: Run formatter**

Run: `cargo fmt --check`
Expected: PASS

**Step 2: Run lints**

Run: `cargo clippy --workspace --all-targets --all-features -- -D warnings`
Expected: PASS

**Step 3: Run full test suite**

Run: `cargo test --workspace`
Expected: PASS

**Step 4: Fix only scoped regressions if any appear**

If verification fails, make the smallest design-consistent change needed and rerun the affected command before rerunning the full suite.

**Step 5: Commit**

```bash
git add -A
git commit -m "chore: verify per-distro installation implementation"
```