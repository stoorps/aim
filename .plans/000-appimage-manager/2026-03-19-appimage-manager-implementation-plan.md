# AppImage Manager Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Build a Rust CLI named `aim` that installs, lists, removes, and review-updates AppImages from multiple source types with full desktop-style integration for user and system scopes.

**Architecture:** Use a single Rust binary with a thin CLI layer over application services, typed source adapters, a normalized registry, and separate installer/integration/update subsystems. Build the project incrementally with test-first steps so the registry model, source resolution, and update planning remain stable as additional adapters land.

**Tech Stack:** Rust, Cargo, clap, dialoguer, console, indicatif, serde, toml or sqlite-backed persistence, reqwest, tokio, tempfile, assert_cmd, predicates, insta or similar snapshot tooling.

---

### Task 1: Scaffold the Cargo project and dependency baseline

**Files:**
- Create: `Cargo.toml`
- Create: `src/main.rs`
- Create: `src/lib.rs`
- Create: `tests/cli_smoke.rs`
- Create: `.gitignore`

**Step 1: Write the failing test**

```rust
use assert_cmd::Command;

#[test]
fn cli_shows_help() {
    let mut cmd = Command::cargo_bin("aim").unwrap();
    cmd.arg("--help").assert().success();
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test cli_shows_help --test cli_smoke`
Expected: FAIL because the crate and binary do not exist yet

**Step 3: Write minimal implementation**

Create a minimal Cargo package with the `aim` binary, library entry point, and an empty `main` using `clap` derive to print help successfully.

**Step 4: Run test to verify it passes**

Run: `cargo test cli_shows_help --test cli_smoke`
Expected: PASS

**Step 5: Commit**

```bash
git add Cargo.toml src/main.rs src/lib.rs tests/cli_smoke.rs .gitignore
git commit -m "chore: scaffold aim cargo project"
```

### Task 2: Add the command surface and top-level CLI parsing

**Files:**
- Modify: `src/main.rs`
- Create: `src/cli/mod.rs`
- Create: `src/cli/args.rs`
- Test: `tests/cli_commands.rs`

**Step 1: Write the failing test**

```rust
use assert_cmd::Command;
use predicates::str::contains;

#[test]
fn help_lists_expected_commands() {
    let mut cmd = Command::cargo_bin("aim").unwrap();
    cmd.arg("--help")
        .assert()
        .success()
        .stdout(contains("remove"))
        .stdout(contains("list"))
        .stdout(contains("update"));
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test help_lists_expected_commands --test cli_commands`
Expected: FAIL because subcommands and positional query parsing are not implemented

**Step 3: Write minimal implementation**

Implement:
- positional optional query for bare `aim {QUERY}`
- `remove {QUERY}`
- `list`
- `update`
- shared `--system` and `--user` scope override flags where appropriate

**Step 4: Run test to verify it passes**

Run: `cargo test help_lists_expected_commands --test cli_commands`
Expected: PASS

**Step 5: Commit**

```bash
git add src/main.rs src/cli/mod.rs src/cli/args.rs tests/cli_commands.rs
git commit -m "feat: add top-level cli command parsing"
```

### Task 3: Define the core domain types and install scope resolution

**Files:**
- Create: `src/domain/mod.rs`
- Create: `src/domain/app.rs`
- Create: `src/domain/source.rs`
- Create: `src/domain/update.rs`
- Create: `src/app/mod.rs`
- Create: `src/app/scope.rs`
- Test: `tests/install_scope.rs`

**Step 1: Write the failing test**

```rust
use aim::app::scope::{resolve_install_scope, ScopeOverride};
use aim::domain::app::InstallScope;

#[test]
fn explicit_scope_override_beats_effective_user() {
    let scope = resolve_install_scope(false, ScopeOverride::System);
    assert_eq!(scope, InstallScope::System);
}
```

**Step 2: Run test to verify it fails**

# AppImage Manager Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Build a Rust workspace where `aim-core` implements AppImage management logic and `aim-cli` provides a thin terminal frontend for install, list, remove, and review-update flows.

**Architecture:** Use a Cargo workspace with `aim-core` holding domain models, services, adapters, registry, installer, and update logic, while `aim-cli` only parses arguments, renders terminal UX, and delegates to `aim-core`. Keep client-facing boundaries explicit so a later GUI crate can reuse `aim-core` without moving logic back out of the library.

**Tech Stack:** Rust, Cargo, clap, dialoguer, console, indicatif, serde, toml or sqlite-backed persistence, reqwest, tokio, tempfile, assert_cmd, predicates, insta or similar snapshot tooling.

---

### Task 1: Scaffold the Cargo workspace baseline

**Files:**
- Create: `Cargo.toml`
- Create: `crates/aim-core/Cargo.toml`
- Create: `crates/aim-core/src/lib.rs`
- Create: `crates/aim-cli/Cargo.toml`
- Create: `crates/aim-cli/src/lib.rs`
- Create: `crates/aim-cli/src/main.rs`
- Create: `tests/cli_smoke.rs`
- Create: `.gitignore`

**Step 1: Write the failing test**

```rust
use assert_cmd::Command;

#[test]
fn cli_shows_help() {
    let mut cmd = Command::cargo_bin("aim").unwrap();
    cmd.arg("--help").assert().success();
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test cli_shows_help --test cli_smoke`
Expected: FAIL because the workspace and binary do not exist yet

**Step 3: Write minimal implementation**

Create a minimal Cargo workspace with `aim-core` and `aim-cli`, wiring the `aim` binary through `aim-cli` and exposing a library entry point from `aim-core`.

**Step 4: Run test to verify it passes**

Run: `cargo test cli_shows_help --test cli_smoke`
Expected: PASS

**Step 5: Commit**

```bash
git add Cargo.toml crates/aim-core/Cargo.toml crates/aim-core/src/lib.rs crates/aim-cli/Cargo.toml crates/aim-cli/src/lib.rs crates/aim-cli/src/main.rs tests/cli_smoke.rs .gitignore
git commit -m "chore: scaffold aim workspace"
```

### Task 2: Add the thin CLI command surface

**Files:**
- Modify: `crates/aim-cli/src/main.rs`
- Create: `crates/aim-cli/src/cli/mod.rs`
- Create: `crates/aim-cli/src/cli/args.rs`
- Test: `tests/cli_commands.rs`

**Step 1: Write the failing test**

```rust
use assert_cmd::Command;
use predicates::str::contains;

#[test]
fn help_lists_expected_commands() {
    let mut cmd = Command::cargo_bin("aim").unwrap();
    cmd.arg("--help")
        .assert()
        .success()
        .stdout(contains("remove"))
        .stdout(contains("list"))
        .stdout(contains("update"));
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test help_lists_expected_commands --test cli_commands`
Expected: FAIL because subcommands and positional query parsing are not implemented

**Step 3: Write minimal implementation**

Implement only:
- positional optional query for bare `aim {QUERY}`
- `remove {QUERY}`
- `list`
- `update`
- shared `--system` and `--user` scope override flags where appropriate

Do not add business logic here beyond command parsing and delegation stubs.

**Step 4: Run test to verify it passes**

Run: `cargo test help_lists_expected_commands --test cli_commands`
Expected: PASS

**Step 5: Commit**

```bash
git add crates/aim-cli/src/main.rs crates/aim-cli/src/cli/mod.rs crates/aim-cli/src/cli/args.rs tests/cli_commands.rs
git commit -m "feat: add thin cli command parsing"
```

### Task 3: Define the core domain types and install scope resolution

**Files:**
- Create: `crates/aim-core/src/domain/mod.rs`
- Create: `crates/aim-core/src/domain/app.rs`
- Create: `crates/aim-core/src/domain/source.rs`
- Create: `crates/aim-core/src/domain/update.rs`
- Create: `crates/aim-core/src/app/mod.rs`
- Create: `crates/aim-core/src/app/scope.rs`
- Test: `tests/install_scope.rs`

**Step 1: Write the failing test**

```rust
use aim_core::app::scope::{resolve_install_scope, ScopeOverride};
use aim_core::domain::app::InstallScope;

#[test]
fn explicit_scope_override_beats_effective_user() {
    let scope = resolve_install_scope(false, ScopeOverride::System);
    assert_eq!(scope, InstallScope::System);
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test explicit_scope_override_beats_effective_user --test install_scope`
Expected: FAIL because core domain types and scope logic do not exist yet

**Step 3: Write minimal implementation**

Add domain types for:
- `InstallScope`
- `AppRecord`
- `SourceKind`
- `SourceRef`
- `ResolvedRelease`
- `UpdatePlan`

Add scope resolution logic that:
- auto-detects by effective privileges
- honors `--system` and `--user` overrides

**Step 4: Run test to verify it passes**

Run: `cargo test explicit_scope_override_beats_effective_user --test install_scope`
Expected: PASS

**Step 5: Commit**

```bash
git add crates/aim-core/src/domain crates/aim-core/src/app tests/install_scope.rs
git commit -m "feat: add core domain types and scope resolution"
```

### Task 4: Implement query parsing and source reference resolution in `aim-core`

**Files:**
- Create: `crates/aim-core/src/app/query.rs`
- Modify: `crates/aim-core/src/domain/source.rs`
- Test: `tests/query_resolution.rs`

**Step 1: Write the failing test**

```rust
use aim_core::app::query::resolve_query;
use aim_core::domain::source::SourceKind;

#[test]
fn owner_repo_defaults_to_github() {
    let source = resolve_query("sharkdp/bat").unwrap();
    assert_eq!(source.kind, SourceKind::GitHub);
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test owner_repo_defaults_to_github --test query_resolution`
Expected: FAIL because query resolution is not implemented

**Step 3: Write minimal implementation**

Support parsing for:
- `owner/repo` as GitHub by default
- GitHub URLs
- GitLab URLs and explicit `gitlab:` prefix
- direct URLs
- `file://` URIs

Return a normalized `SourceRef` without triggering downloads or installation.

**Step 4: Run test to verify it passes**

Run: `cargo test owner_repo_defaults_to_github --test query_resolution`
Expected: PASS

**Step 5: Commit**

```bash
git add crates/aim-core/src/app/query.rs crates/aim-core/src/domain/source.rs tests/query_resolution.rs
git commit -m "feat: resolve user queries into source references"
```

### Task 5: Add registry persistence and migration-friendly app records in `aim-core`

**Files:**
- Create: `crates/aim-core/src/registry/mod.rs`
- Create: `crates/aim-core/src/registry/store.rs`
- Create: `crates/aim-core/src/registry/model.rs`
- Test: `tests/registry_roundtrip.rs`

**Step 1: Write the failing test**

```rust
use aim_core::registry::store::RegistryStore;
use tempfile::tempdir;

#[test]
fn registry_round_trips_app_records() {
    let dir = tempdir().unwrap();
    let store = RegistryStore::new(dir.path().join("registry.toml"));
    let loaded = store.load().unwrap();
    assert!(loaded.apps.is_empty());
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test registry_round_trips_app_records --test registry_roundtrip`
Expected: FAIL because no registry store exists

**Step 3: Write minimal implementation**

Implement a registry store with:
- serialized root structure
- normalized `AppRecord` persistence
- version field for future migrations
- read and write APIs

Choose a storage format that is easy to inspect and migrate, such as TOML or SQLite.

**Step 4: Run test to verify it passes**

Run: `cargo test registry_round_trips_app_records --test registry_roundtrip`
Expected: PASS

**Step 5: Commit**

```bash
git add crates/aim-core/src/registry tests/registry_roundtrip.rs
git commit -m "feat: add persistent core registry store"
```

### Task 6: Build the source adapter trait and contract harness in `aim-core`

**Files:**
- Create: `crates/aim-core/src/adapters/mod.rs`
- Create: `crates/aim-core/src/adapters/traits.rs`
- Create: `crates/aim-core/src/adapters/test_support.rs`
- Test: `tests/adapter_contract.rs`

**Step 1: Write the failing test**

```rust
use aim_core::adapters::traits::AdapterCapabilities;

#[test]
fn adapter_capabilities_can_report_exact_resolution_only() {
    let capabilities = AdapterCapabilities::exact_resolution_only();
    assert!(!capabilities.supports_search);
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test adapter_capabilities_can_report_exact_resolution_only --test adapter_contract`
Expected: FAIL because adapter abstractions do not exist

**Step 3: Write minimal implementation**

Define:
- `SourceAdapter` trait
- capability flags
- normalized adapter response types
- reusable test helpers for contract behavior

Do not implement network-backed adapters yet. Focus on the stable core trait surface.

**Step 4: Run test to verify it passes**

Run: `cargo test adapter_capabilities_can_report_exact_resolution_only --test adapter_contract`
Expected: PASS

**Step 5: Commit**

```bash
git add crates/aim-core/src/adapters tests/adapter_contract.rs
git commit -m "feat: add source adapter trait and contract surface"
```

### Task 7: Define client interaction models in `aim-core` and thin terminal rendering in `aim-cli`

**Files:**
- Create: `crates/aim-core/src/app/interaction.rs`
- Create: `crates/aim-cli/src/ui/mod.rs`
- Create: `crates/aim-cli/src/ui/render.rs`
- Create: `crates/aim-cli/src/ui/prompt.rs`
- Test: `tests/ui_summary.rs`

**Step 1: Write the failing test**

```rust
use aim_cli::ui::render::render_update_summary;

#[test]
fn update_summary_mentions_selected_count() {
    let output = render_update_summary(3, 2, 1);
    assert!(output.contains("selected: 2"));
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test update_summary_mentions_selected_count --test ui_summary`
Expected: FAIL because client rendering helpers do not exist

**Step 3: Write minimal implementation**

Create:
- typed interaction and progress models in `aim-core`
- a thin CLI UI facade in `aim-cli` that centralizes styling with `console`
- prompt orchestration using `dialoguer`

Do not move any business rules into `aim-cli`.

**Step 4: Run test to verify it passes**

Run: `cargo test update_summary_mentions_selected_count --test ui_summary`
Expected: PASS

**Step 5: Commit**

```bash
git add crates/aim-core/src/app/interaction.rs crates/aim-cli/src/ui tests/ui_summary.rs
git commit -m "feat: add core interaction models and thin cli ui"
```

### Task 8: Implement installer and desktop integration path resolution in `aim-core`

**Files:**
- Create: `crates/aim-core/src/integration/mod.rs`
- Create: `crates/aim-core/src/integration/paths.rs`
- Create: `crates/aim-core/src/integration/install.rs`
- Create: `crates/aim-core/src/platform/mod.rs`
- Test: `tests/install_paths.rs`

**Step 1: Write the failing test**

```rust
use aim_core::domain::app::InstallScope;
use aim_core::integration::paths::managed_appimage_path;
use std::path::Path;

#[test]
fn user_scope_path_lands_under_home_managed_dir() {
    let path = managed_appimage_path(Path::new("/home/test"), InstallScope::User, "bat");
    assert!(path.to_string_lossy().contains("bat"));
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test user_scope_path_lands_under_home_managed_dir --test install_paths`
Expected: FAIL because install path logic does not exist

**Step 3: Write minimal implementation**

Implement:
- managed install path resolution for user and system scopes
- integration artifact path calculation
- atomic staging and replacement helpers

Keep actual desktop registration side effects behind abstractions so they remain testable.

**Step 4: Run test to verify it passes**

Run: `cargo test user_scope_path_lands_under_home_managed_dir --test install_paths`
Expected: PASS

**Step 5: Commit**

```bash
git add crates/aim-core/src/integration crates/aim-core/src/platform tests/install_paths.rs
git commit -m "feat: add core install and integration path handling"
```

### Task 9: Implement identity normalization and raw URL fallback in `aim-core`

**Files:**
- Create: `crates/aim-core/src/app/identity.rs`
- Modify: `crates/aim-core/src/domain/app.rs`
- Test: `tests/identity_resolution.rs`

**Step 1: Write the failing test**

```rust
use aim_core::app::identity::{resolve_identity, IdentityFallback};

#[test]
fn unresolved_identity_can_fall_back_to_url() {
    let identity = resolve_identity(None, None, Some("https://example.com/app.AppImage"), IdentityFallback::AllowRawUrl).unwrap();
    assert!(identity.stable_id.contains("example.com"));
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test unresolved_identity_can_fall_back_to_url --test identity_resolution`
Expected: FAIL because identity resolution does not exist

**Step 3: Write minimal implementation**

Implement identity normalization with:
- confident resolution path
- low-confidence state handling
- raw URL fallback when allowed

Keep the prompting decision outside this module so the logic remains deterministic and reusable across CLI and GUI clients.

**Step 4: Run test to verify it passes**

Run: `cargo test unresolved_identity_can_fall_back_to_url --test identity_resolution`
Expected: PASS

**Step 5: Commit**

```bash
git add crates/aim-core/src/app/identity.rs crates/aim-core/src/domain/app.rs tests/identity_resolution.rs
git commit -m "feat: add core identity normalization and fallback logic"
```

### Task 10: Implement update planning in `aim-core` and review-first dispatch in `aim-cli`

**Files:**
- Create: `crates/aim-core/src/app/update.rs`
- Modify: `crates/aim-cli/src/cli/args.rs`
- Modify: `crates/aim-cli/src/main.rs`
- Test: `tests/update_planning.rs`

**Step 1: Write the failing test**

```rust
use aim_core::app::update::build_update_plan;

#[test]
fn empty_registry_produces_empty_plan() {
    let plan = build_update_plan(&[]).unwrap();
    assert!(plan.items.is_empty());
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test empty_registry_produces_empty_plan --test update_planning`
Expected: FAIL because update planning does not exist

**Step 3: Write minimal implementation**

Implement:
- update plan model in `aim-core`
- comparison of installed state against adapter-provided candidate data
- bare `aim` dispatch in `aim-cli` into the `aim-core` update planning path when no positional query is present

Do not execute downloads yet in this task. Focus on planning and command dispatch.

**Step 4: Run test to verify it passes**

Run: `cargo test empty_registry_produces_empty_plan --test update_planning`
Expected: PASS

**Step 5: Commit**

```bash
git add crates/aim-core/src/app/update.rs crates/aim-cli/src/cli/args.rs crates/aim-cli/src/main.rs tests/update_planning.rs
git commit -m "feat: add core update planning and cli dispatch"
```

### Task 11: Add the GitHub adapter and one core add flow

**Files:**
- Create: `crates/aim-core/src/adapters/github.rs`
- Create: `crates/aim-core/src/app/add.rs`
- Modify: `crates/aim-core/src/adapters/mod.rs`
- Modify: `crates/aim-cli/src/main.rs`
- Test: `tests/github_add_flow.rs`

**Step 1: Write the failing test**

```rust
#[test]
fn github_adapter_can_normalize_owner_repo_source() {
    let source = aim_core::app::query::resolve_query("sharkdp/bat").unwrap();
    assert_eq!(source.kind.as_str(), "github");
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test github_adapter_can_normalize_owner_repo_source --test github_add_flow`
Expected: FAIL because the add flow and GitHub adapter are not wired into the core services

**Step 3: Write minimal implementation**

Implement:
- GitHub adapter skeleton in `aim-core`
- add orchestration flow in `aim-core` from query resolution to normalized release selection
- minimal `aim-cli` wiring to invoke the add flow
- fixture-backed or mocked HTTP path for tests

**Step 4: Run test to verify it passes**

Run: `cargo test github_adapter_can_normalize_owner_repo_source --test github_add_flow`
Expected: PASS

**Step 5: Commit**

```bash
git add crates/aim-core/src/adapters/github.rs crates/aim-core/src/app/add.rs crates/aim-core/src/adapters/mod.rs crates/aim-cli/src/main.rs tests/github_add_flow.rs
git commit -m "feat: add github source adapter and core add flow"
```

### Task 12: Add remaining adapters behind the same core contract

**Files:**
- Create: `crates/aim-core/src/adapters/gitlab.rs`
- Create: `crates/aim-core/src/adapters/direct_url.rs`
- Create: `crates/aim-core/src/adapters/zsync.rs`
- Create: `crates/aim-core/src/adapters/sourceforge.rs`
- Create: `crates/aim-core/src/adapters/custom_json.rs`
- Modify: `crates/aim-core/src/adapters/mod.rs`
- Test: `tests/adapter_smoke.rs`

**Step 1: Write the failing test**

```rust
use aim_core::adapters::all_adapter_kinds;

#[test]
fn all_expected_adapter_kinds_are_registered() {
    let kinds = all_adapter_kinds();
    assert!(kinds.contains(&"gitlab"));
    assert!(kinds.contains(&"direct-url"));
    assert!(kinds.contains(&"zsync"));
    assert!(kinds.contains(&"sourceforge"));
    assert!(kinds.contains(&"custom-json"));
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test all_expected_adapter_kinds_are_registered --test adapter_smoke`
Expected: FAIL because the additional adapters do not exist

**Step 3: Write minimal implementation**

Add adapter modules and register them behind the shared core trait. Keep each adapter bootstrapped with contract-valid behavior and fixture-friendly parsing paths before adding richer source-specific behaviors.

**Step 4: Run test to verify it passes**

Run: `cargo test all_expected_adapter_kinds_are_registered --test adapter_smoke`
Expected: PASS

**Step 5: Commit**

```bash
git add crates/aim-core/src/adapters tests/adapter_smoke.rs
git commit -m "feat: add remaining core source adapter skeletons"
```

### Task 13: Implement list and remove in `aim-core`, keep `aim-cli` thin

**Files:**
- Create: `crates/aim-core/src/app/list.rs`
- Create: `crates/aim-core/src/app/remove.rs`
- Modify: `crates/aim-cli/src/main.rs`
- Test: `tests/remove_flow.rs`

**Step 1: Write the failing test**

```rust
#[test]
fn remove_flow_rejects_unknown_app_names() {
    let result = aim_core::app::remove::resolve_registered_app("bat", &[]);
    assert!(result.is_err());
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test remove_flow_rejects_unknown_app_names --test remove_flow`
Expected: FAIL because list and remove services do not exist

**Step 3: Write minimal implementation**

Implement in `aim-core`:
- list formatting input model
- registered app name matching
- ambiguity handling hooks through interaction requests
- conservative removal sequencing for artifact and integration cleanup

Add only wiring and rendering in `aim-cli`.

**Step 4: Run test to verify it passes**

Run: `cargo test remove_flow_rejects_unknown_app_names --test remove_flow`
Expected: PASS

**Step 5: Commit**

```bash
git add crates/aim-core/src/app/list.rs crates/aim-core/src/app/remove.rs crates/aim-cli/src/main.rs tests/remove_flow.rs
git commit -m "feat: add core list and remove services"
```

### Task 14: Wire the binary end to end and document the workspace split

**Files:**
- Modify: `crates/aim-cli/src/main.rs`
- Modify: `crates/aim-core/src/lib.rs`
- Test: `tests/end_to_end_cli.rs`
- Modify: `README.md`

**Step 1: Write the failing test**

```rust
use assert_cmd::Command;
use predicates::str::contains;

#[test]
fn list_command_runs_without_registry_entries() {
    let mut cmd = Command::cargo_bin("aim").unwrap();
    cmd.arg("list").assert().success().stdout(contains("installed"));
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test list_command_runs_without_registry_entries --test end_to_end_cli`
Expected: FAIL because services are not fully wired into the binary

**Step 3: Write minimal implementation**

Wire all top-level commands through `aim-core` service APIs and add minimal README usage documentation for:
- add/query flow
- bare update flow
- list
- remove
- scope overrides

Also document that the workspace is intentionally split so a future GUI can reuse `aim-core`.

**Step 4: Run test to verify it passes**

Run: `cargo test list_command_runs_without_registry_entries --test end_to_end_cli`
Expected: PASS

**Step 5: Commit**

```bash
git add crates/aim-cli/src/main.rs crates/aim-core/src/lib.rs tests/end_to_end_cli.rs README.md
git commit -m "feat: wire aim cli to aim-core end to end"
```

### Task 15: Verification sweep and architecture leak check

**Files:**
- Modify: `README.md`
- Modify: `.plans/appimage-manager/2026-03-19-appimage-manager-design.md`
- Modify: `.plans/appimage-manager/2026-03-19-appimage-manager-implementation-plan.md`

**Step 1: Write the failing test**

There is no new product behavior in this task. Instead, identify the highest-risk missing automated check from earlier tasks and add that test first, prioritizing any gap that suggests business logic is drifting into `aim-cli`.

**Step 2: Run test to verify it fails**

Run: `cargo test`
Expected: Identify at least one missing assertion or regression gap before making release-readiness claims

**Step 3: Write minimal implementation**

Close the smallest meaningful remaining gap. Update docs only where behavior has materially changed from the plan.

**Step 4: Run test to verify it passes**

Run: `cargo test`
Expected: PASS

Run: `cargo fmt --check`
Expected: PASS

Run: `cargo clippy --workspace --all-targets --all-features -- -D warnings`
Expected: PASS

**Step 5: Commit**

```bash
git add README.md .plans/appimage-manager/2026-03-19-appimage-manager-design.md .plans/appimage-manager/2026-03-19-appimage-manager-implementation-plan.md
git commit -m "chore: finalize appimage manager workspace implementation"
```

## Notes For Execution

- This workspace is currently empty and not initialized as a git repository, so commit steps will remain blocked until `git init` or an equivalent repository setup occurs.
- The execution session should create a Cargo workspace, not a single binary crate.
- The first adapter should be GitHub because it exercises the `owner/repo` shorthand and the most likely early-user path.
- Keep custom JSON feed support declarative in v1.
- Do not add a plugin runtime.
- Do not let `aim-cli` accumulate business logic; if a behavior could be reused by a future GUI, it belongs in `aim-core`.

Plan complete and saved to `.plans/appimage-manager/2026-03-19-appimage-manager-implementation-plan.md`. Two execution options:

**1. Subagent-Driven (this session)** - I dispatch a fresh subagent per task, review between tasks, and iterate in this session.

**2. Parallel Session (separate)** - Open a new session with executing-plans and execute the plan with checkpoints.

Which approach?