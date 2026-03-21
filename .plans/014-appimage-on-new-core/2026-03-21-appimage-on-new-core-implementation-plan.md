# AppImage On The New Core Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Validate AppImage as the first real provider module by making `upm-core` treat provider composition as the normal path for capability discovery, remote show resolution, and update execution.

**Architecture:** Keep AppImage-specific transport and resolution logic in `upm-appimage`. Extend `upm-core` only with generic provider-registry contracts and plumbing so the CLI can compose providers once and reuse that composition across `search`, `add`, `show`, and `update`. Avoid introducing speculative provider hooks for `remove` or `list`; those flows are already generic and should stay that way until a real provider needs more surface area.

**Tech Stack:** Rust workspace, `upm`, `upm-core`, `upm-appimage`, Cargo integration tests, fixture-backed provider tests, CLI end-to-end tests.

---

### Task 1: Add capability discovery to `ProviderRegistry`

**Files:**
- Modify: `crates/upm-core/src/app/providers.rs`
- Modify: `crates/upm-core/src/lib.rs`
- Test: `crates/upm-core/tests/provider_registry.rs`

**Step 1: Write the failing capability-discovery expectations**

Extend `crates/upm-core/tests/provider_registry.rs` to assert:

- the registry can report which provider ids are registered
- the registry can report whether a provider supports search and/or external add
- empty registries report no capabilities

Keep the expectations generic. Do not encode AppImage-only behavior into the registry API.

**Step 2: Run the focused test to verify failure**

Run:

```bash
cargo test --package upm-core --test provider_registry
```

Expected: FAIL because `ProviderRegistry` is still only a passive bag of references.

**Step 3: Implement minimal capability discovery**

Update `crates/upm-core/src/app/providers.rs` so `ProviderRegistry` exposes a small, stable query surface such as:

- a way to enumerate registered provider ids
- a way to report capabilities for a provider id
- a small capability record rather than operation-specific booleans scattered around callers

Do not add plugin loading, global registries, or dynamic configuration yet.

**Step 4: Run the focused test to verify pass**

Run:

```bash
cargo test --package upm-core --test provider_registry
```

Expected: PASS.

**Step 5: Commit**

```bash
git add crates/upm-core/src/app/providers.rs crates/upm-core/src/lib.rs crates/upm-core/tests/provider_registry.rs
git commit -m "feat: add provider capability discovery"
```

### Task 2: Route remote `show` through registered providers

**Files:**
- Modify: `crates/upm-core/src/app/show.rs`
- Modify: `crates/upm/src/lib.rs`
- Test: `crates/upm-core/tests/show_resolution.rs`
- Test: `crates/upm/tests/end_to_end_cli.rs`

**Step 1: Write the failing `show` expectations**

Add coverage proving that:

- remote `show appimagehub/<id>` resolves through the registered provider path
- installed-app `show` behavior remains unchanged
- unsupported queries still fail distinctly from provider-backed remote queries

Prefer one new `upm-core` test for remote resolution and one CLI-facing assertion in `crates/upm/tests/end_to_end_cli.rs`.

**Step 2: Run the focused tests to verify failure**

Run:

```bash
cargo test --package upm-core --test show_resolution
cargo test --package upm --test end_to_end_cli
```

Expected: FAIL because the remote show path still calls the add planner without a `ProviderRegistry`.

**Step 3: Thread provider composition through `show`**

Update the show pipeline so:

- `upm-core` exposes provider-aware show entrypoints alongside the current defaults
- remote show resolution uses `build_add_plan_with_registered_providers` rather than the provider-blind path
- the CLI wraps remote show dispatch in `providers::with_provider_registry(...)`

Keep installed-record rendering and summary formatting unchanged.

**Step 4: Run the focused tests to verify pass**

Run:

```bash
cargo test --package upm-core --test show_resolution
cargo test --package upm --test end_to_end_cli
```

Expected: PASS.

**Step 5: Commit**

```bash
git add crates/upm-core/src/app/show.rs crates/upm/src/lib.rs crates/upm-core/tests/show_resolution.rs crates/upm/tests/end_to_end_cli.rs
git commit -m "feat: route show through provider registry"
```

### Task 3: Route `update` execution through registered providers

**Files:**
- Modify: `crates/upm-core/src/app/update.rs`
- Modify: `crates/upm/src/lib.rs`
- Test: `crates/upm-core/tests/update_planning.rs`
- Test: `crates/upm/tests/end_to_end_cli.rs`

**Step 1: Write the failing `update` expectations**

Add coverage proving that:

- AppImage-backed records can be refreshed through the update path with registered providers
- existing GitHub and direct-url update behavior remains unchanged
- the update execution path still restores previous payloads on failure

Prefer focused `upm-core` tests plus one CLI integration assertion for an AppImage-backed update review or execution path.

**Step 2: Run the focused tests to verify failure**

Run:

```bash
cargo test --package upm-core --test update_planning
cargo test --package upm --test end_to_end_cli
```

Expected: FAIL because update execution still rebuilds add plans without a `ProviderRegistry`.

**Step 3: Thread provider composition through `update`**

Update the update pipeline so:

- provider-aware update entrypoints exist in `upm-core`
- `execute_update` rebuilds plans through the provider-aware add planner
- the CLI wraps `update` execution in `providers::with_provider_registry(...)`

Do not generalize the update-channel model yet. Reuse the current channel semantics and only change how provider-backed plans are rebuilt.

**Step 4: Run the focused tests to verify pass**

Run:

```bash
cargo test --package upm-core --test update_planning
cargo test --package upm --test end_to_end_cli
```

Expected: PASS.

**Step 5: Commit**

```bash
git add crates/upm-core/src/app/update.rs crates/upm/src/lib.rs crates/upm-core/tests/update_planning.rs crates/upm/tests/end_to_end_cli.rs
git commit -m "feat: route updates through provider registry"
```

### Task 4: Lock AppImage in as the reference provider module

**Files:**
- Modify: `crates/upm-appimage/tests/appimagehub_search.rs`
- Modify: `crates/upm/tests/end_to_end_cli.rs`
- Modify: `crates/upm/tests/ui_summary.rs`
- Test: `crates/upm-core/tests/provider_registry.rs`

**Step 1: Write the failing reference-provider expectations**

Add end-to-end coverage proving that AppImage support is fully module-driven:

- `search` still surfaces AppImageHub hits through the registry
- `show` for AppImageHub remote queries works through the registry
- `update` can refresh AppImage-backed records through the registry
- user-facing summaries still render truthful `upm` paths and origins

Keep the assertions focused on module composition rather than UI restyling.

**Step 2: Run the focused tests to verify failure**

Run:

```bash
cargo test --package upm-appimage --test appimagehub_search
cargo test --package upm --test end_to_end_cli
cargo test --package upm --test ui_summary
```

Expected: FAIL until the new `show` and `update` registry plumbing is complete.

**Step 3: Tighten provider-contract validation**

Update the tests so they prove:

- AppImage is composed only through `upm-appimage`
- `ProviderRegistry` is the shared composition point for all AppImage-facing command paths
- AppImage is still not reintroduced as a hardcoded built-in inside `upm-core`

Do not move AppImageHub back into `all_adapter_kinds()`.

**Step 4: Run the focused tests to verify pass**

Run:

```bash
cargo test --package upm-appimage --test appimagehub_search
cargo test --package upm --test end_to_end_cli
cargo test --package upm --test ui_summary
```

Expected: PASS.

**Step 5: Commit**

```bash
git add crates/upm-appimage/tests/appimagehub_search.rs crates/upm/tests/end_to_end_cli.rs crates/upm/tests/ui_summary.rs crates/upm-core/tests/provider_registry.rs
git commit -m "test: validate appimage as reference provider module"
```

### Task 5: Update architecture docs and run full verification

**Files:**
- Modify: `.architecture/overview.md`
- Modify: `.architecture/roadmap.md`
- Modify: `README.md`

**Step 1: Update docs for Milestone 1 completion state**

Document:

- capability discovery in `ProviderRegistry`
- provider-aware `show` and `update` execution paths
- AppImage as the first validated provider module on the new architecture
- the explicit non-goal that `remove` and `list` remain generic until a provider needs extra hooks

**Step 2: Verify the docs mention the Milestone 1 state**

Run:

```bash
rg -n "ProviderRegistry|capabilit|upm-appimage|show|update" README.md .architecture/overview.md .architecture/roadmap.md
```

Expected: matches describing the expanded provider surface.

**Step 3: Run full verification**

Run:

```bash
cargo fmt --all
cargo test --workspace
cargo clippy --workspace --all-targets --all-features -- -D warnings
```

Expected: PASS.

**Step 4: Commit**

```bash
git add README.md .architecture/overview.md .architecture/roadmap.md
git commit -m "docs: describe appimage provider milestone"
```