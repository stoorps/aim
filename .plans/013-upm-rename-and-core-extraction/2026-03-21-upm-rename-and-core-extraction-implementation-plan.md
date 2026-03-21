# UPM Rename And Core Extraction Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Rename the product from `aim` to `upm`, remove legacy `aim` runtime interfaces, extract the shared headless backend into `upm-core`, and move AppImage-specific transport and provider logic into a separate `upm-appimage` module without regressing current AppImage workflows.

**Architecture:** Execute this in vertical slices. First rename the workspace, binary, paths, environment interfaces, and tests to `upm` without carrying legacy `aim` compatibility. Next introduce a narrow provider-composition seam in `upm-core` so AppImage-specific add and search logic can move into `upm-appimage` without creating a dependency cycle. Finally rewire the `upm` CLI to assemble built-in providers, update docs, and run full verification.

**Tech Stack:** Rust workspace, Cargo manifests, clap CLI, ratatui frontend crate, core domain/app modules, fixture-backed provider tests, workspace-wide `cargo test` and `cargo clippy`.

---

### Task 1: Rename the workspace, binary, and default runtime paths to `upm`

**Files:**
- Modify: `Cargo.toml`
- Rename: `crates/aim-cli` -> `crates/upm`
- Rename: `crates/aim-core` -> `crates/upm-core`
- Modify: `crates/upm/Cargo.toml`
- Modify: `crates/upm/src/main.rs`
- Modify: `crates/upm/src/lib.rs`
- Modify: `crates/upm/src/cli/args.rs`
- Modify: `crates/upm/src/config.rs`
- Modify: `crates/upm/src/cli/config.rs`
- Modify: `crates/upm-core/Cargo.toml`
- Modify: `crates/upm-core/src/platform/mod.rs`
- Modify: `crates/upm-core/src/integration/paths.rs`
- Modify: `crates/upm-core/src/integration/policy.rs`
- Test: `crates/upm/tests/cli_smoke.rs`
- Test: `crates/upm/tests/cli_commands.rs`
- Test: `crates/upm/tests/config_loading.rs`
- Test: `crates/upm-core/tests/install_paths.rs`
- Test: `crates/upm-core/tests/install_policy.rs`

**Step 1: Write the failing rename expectations**

Update the selected tests to assert:

- the binary name is `upm`
- clap parses `upm` instead of `aim`
- default config path is `~/.config/upm/config.toml`
- default registry path is `~/.local/share/upm/registry.toml`
- default managed payload roots are `.local/lib/upm/appimages` and `/opt/upm/appimages`
- desktop entry filenames use `upm-<stable-id>.desktop`

**Step 2: Run the focused tests to verify failure**

Run:

```bash
cargo test --package aim-cli --test cli_smoke
cargo test --package aim-cli --test config_loading
cargo test --package aim-core --test install_paths
cargo test --package aim-core --test install_policy
```

Expected: FAIL because the workspace still exposes `aim`, `aim-cli`, `aim-core`, and `aim` default paths.

**Step 3: Perform the crate and manifest rename**

Run:

```bash
git mv crates/aim-cli crates/upm
git mv crates/aim-core crates/upm-core
```

Then update:

- workspace members and default members in `Cargo.toml`
- package names to `upm` and `upm-core`
- binary name to `upm`
- crate imports from `aim_core` to `upm_core`
- crate imports from `aim_cli` to `upm`
- clap command name from `aim` to `upm`
- default config, registry, payload-root, and desktop-entry paths to `upm`

**Step 4: Run the focused tests to verify pass**

Run:

```bash
cargo test --package upm --test cli_smoke
cargo test --package upm --test cli_commands
cargo test --package upm --test config_loading
cargo test --package upm-core --test install_paths
cargo test --package upm-core --test install_policy
```

Expected: PASS.

**Step 5: Commit**

```bash
git add Cargo.toml crates/upm crates/upm-core
git commit -m "refactor: rename workspace to upm"
```

### Task 2: Remove remaining `aim`-named runtime interfaces

**Files:**
- Modify: `crates/upm/src/config.rs`
- Modify: `crates/upm/src/cli/config.rs`
- Modify: `crates/upm/src/lib.rs`
- Modify: `crates/upm/src/ui/prompt.rs`
- Modify: `crates/upm-core/src/platform/mod.rs`
- Modify: `crates/upm-core/src/source/github.rs`
- Modify: `crates/upm-core/src/source/appimagehub.rs`
- Modify: `crates/upm-core/src/integration/refresh.rs`
- Test: `crates/upm/tests/config_loading.rs`

**Step 1: Write the failing strict-rename expectations**

Update representative tests to cover:

- config lookup uses `UPM_CONFIG_PATH`
- registry lookup uses `UPM_REGISTRY_PATH`
- old `AIM_*` config and registry overrides are ignored
- tracking preference uses `UPM_TRACKING_PREFERENCE`
- old `AIM_TRACKING_PREFERENCE` is ignored
- provider fixture execution uses the renamed `UPM_*` interfaces through CLI-facing tests
- managed install and summary output use `upm` paths and desktop prefixes

**Step 2: Run the focused tests to verify failure**

Run:

```bash
cargo test --package upm --test config_loading
cargo test --package upm --test search_cli
cargo test --package upm --test end_to_end_cli
cargo test --package upm --test ui_summary
```

Expected: FAIL because representative CLI and config flows still depend on old `aim` names.

**Step 3: Remove the remaining `aim` interfaces**

Update the codebase so renamed runtime interfaces are consistently `upm`:

- environment variable names use `UPM_*`
- helper/debug prefixes print `[upm]`
- GitHub user agent identifies as `upm/0.1`
- old `aim` compatibility reads are removed instead of preserved

**Step 4: Run the focused tests to verify pass**

Run:

```bash
cargo test --package upm --test config_loading
cargo test --package upm --test search_cli
cargo test --package upm --test end_to_end_cli
cargo test --package upm --test ui_summary
```

Expected: PASS.

**Step 5: Commit**

```bash
git add crates/upm/src/config.rs crates/upm/src/cli/config.rs crates/upm/src/lib.rs crates/upm/src/ui/prompt.rs crates/upm-core/src/platform/mod.rs crates/upm-core/src/source/github.rs crates/upm-core/src/source/appimagehub.rs crates/upm-core/src/integration/refresh.rs crates/upm/tests/config_loading.rs
git commit -m "refactor: remove remaining aim runtime interfaces"
```

### Task 3: Add a provider-composition seam in `upm-core`

**Files:**
- Create: `crates/upm-core/src/app/providers.rs`
- Modify: `crates/upm-core/src/app/mod.rs`
- Modify: `crates/upm-core/src/app/add.rs`
- Modify: `crates/upm-core/src/app/search.rs`
- Modify: `crates/upm-core/src/lib.rs`
- Create: `crates/upm-core/tests/provider_registry.rs`

**Step 1: Write the failing provider-composition tests**

Create `crates/upm-core/tests/provider_registry.rs` with two focused tests:

- `build_search_results_with_registered_providers_uses_external_hits` using a stub external search provider
- `build_add_plan_with_registered_providers_delegates_appimagehub_like_sources` using a stub external add provider that returns a fixed artifact and release

The tests should prove that `upm-core` orchestration can consume provider-supplied search and add behavior without hardcoding AppImage-specific modules.

**Step 2: Run the focused tests to verify failure**

Run:

```bash
cargo test --package upm-core --test provider_registry
```

Expected: FAIL because the orchestration layer still hardcodes AppImageHub in `app/add.rs` and `app/search.rs`.

**Step 3: Introduce the narrow composition API**

Create `crates/upm-core/src/app/providers.rs` with minimal types:

- `pub trait ExternalAddProvider`
- `pub struct ExternalAddResolution`
- `pub struct ProviderRegistry<'a>`

Requirements:

- `ProviderRegistry` carries `search_providers: Vec<&'a dyn SearchProvider>` and `external_add_providers: Vec<&'a dyn ExternalAddProvider>`
- `build_search_results` can delegate to `build_search_results_with` using providers supplied by the caller
- `build_add_plan_with_reporter_and_policy` gets a sibling entrypoint that accepts a `ProviderRegistry`
- core built-ins remain in `upm-core`; only AppImage-specific exact-resolution and search logic should move behind the new registry seam

Keep the interface intentionally small. Do not attempt plugin loading or dynamic discovery yet.

**Step 4: Run the focused tests to verify pass**

Run:

```bash
cargo test --package upm-core --test provider_registry
```

Expected: PASS.

**Step 5: Commit**

```bash
git add crates/upm-core/src/app/providers.rs crates/upm-core/src/app/mod.rs crates/upm-core/src/app/add.rs crates/upm-core/src/app/search.rs crates/upm-core/src/lib.rs crates/upm-core/tests/provider_registry.rs
git commit -m "refactor: add provider composition seam to upm-core"
```

### Task 4: Extract AppImage-specific logic into `upm-appimage`

**Files:**
- Modify: `Cargo.toml`
- Create: `crates/upm-appimage/Cargo.toml`
- Create: `crates/upm-appimage/src/lib.rs`
- Create: `crates/upm-appimage/src/add.rs`
- Create: `crates/upm-appimage/src/search.rs`
- Create: `crates/upm-appimage/src/source/mod.rs`
- Create: `crates/upm-appimage/src/source/appimagehub.rs`
- Modify: `crates/upm-core/src/adapters/mod.rs`
- Modify: `crates/upm-core/src/source/mod.rs`
- Modify: `crates/upm-core/src/app/add.rs`
- Modify: `crates/upm-core/src/app/search.rs`
- Create: `crates/upm-appimage/tests/appimagehub_search.rs`
- Modify: `crates/upm-core/tests/adapter_contract.rs`
- Modify: `crates/upm-core/tests/adapter_smoke.rs`

**Step 1: Write the failing extracted-module test**

Create `crates/upm-appimage/tests/appimagehub_search.rs` by moving the current AppImageHub search expectations out of `upm-core` and updating imports to target the new crate.

Also update the affected `upm-core` tests so they no longer import `AppImageHubAdapter` from `upm-core` directly.

**Step 2: Run the focused tests to verify failure**

Run:

```bash
cargo test --package upm-appimage --test appimagehub_search
```

Expected: FAIL because the new crate does not exist yet.

**Step 3: Create the new crate and move AppImageHub implementation into it**

Move the AppImageHub-specific code into the new crate:

- transport and fixture logic from `upm-core/src/source/appimagehub.rs`
- AppImage-backed exact-resolution logic into `crates/upm-appimage/src/add.rs` implementing `ExternalAddProvider`
- AppImageHub search provider logic out of `upm-core/src/app/search.rs` into `crates/upm-appimage/src/search.rs`

`upm-appimage` should depend on `upm-core`, not the other way around.

Leave `SourceKind::AppImageHub`, `SourceInputKind::AppImageHub*`, and `NormalizedSourceKind::AppImageHub` in `upm-core` for this milestone. The deeper provider/domain generalization belongs to the next milestone.

**Step 4: Remove direct AppImageHub wiring from `upm-core`**

Update `upm-core` so it no longer declares:

- `pub mod appimagehub;` in `src/adapters/mod.rs`
- `pub mod appimagehub;` in `src/source/mod.rs`
- built-in AppImageHub search-provider construction in `src/app/search.rs`
- direct `AppImageHubAdapter` imports in `src/app/add.rs`

After this step, AppImage behavior should exist only through the provider registry seam from Task 3.

**Step 5: Run the focused tests to verify pass**

Run:

```bash
cargo test --package upm-appimage --test appimagehub_search
cargo test --package upm-core --test adapter_contract
cargo test --package upm-core --test adapter_smoke
```

Expected: PASS.

**Step 6: Commit**

```bash
git add Cargo.toml crates/upm-appimage crates/upm-core/src/adapters/mod.rs crates/upm-core/src/source/mod.rs crates/upm-core/src/app/add.rs crates/upm-core/src/app/search.rs crates/upm-core/tests/adapter_contract.rs crates/upm-core/tests/adapter_smoke.rs
git commit -m "refactor: extract appimage support into upm-appimage"
```

### Task 5: Rewire the `upm` CLI to assemble built-in providers from modules

**Files:**
- Modify: `crates/upm/Cargo.toml`
- Create: `crates/upm/src/providers.rs`
- Modify: `crates/upm/src/lib.rs`
- Test: `crates/upm/tests/search_cli.rs`
- Test: `crates/upm/tests/end_to_end_cli.rs`
- Test: `crates/upm/tests/ui_summary.rs`

**Step 1: Write the failing CLI integration expectations**

Update CLI integration tests to prove that:

- `upm search firefox` still includes AppImageHub results
- direct `upm appimagehub/2338455` install flow still succeeds through the CLI
- the final summary output still renders the new `upm`-prefixed paths and desktop-entry names

**Step 2: Run the focused tests to verify failure**

Run:

```bash
cargo test --package upm --test search_cli
cargo test --package upm --test end_to_end_cli
cargo test --package upm --test ui_summary
```

Expected: FAIL because `upm` does not yet assemble AppImage providers through the extracted module.

**Step 3: Add CLI-side provider assembly**

Create `crates/upm/src/providers.rs` that:

- builds the `ProviderRegistry` for `upm-core`
- registers the `upm-appimage` search provider
- registers the `upm-appimage` external add provider

Update `crates/upm/src/lib.rs` so dispatch paths call the provider-aware core entrypoints instead of hardcoded core defaults.

Do not move progress rendering or config loading into `upm-core`; the CLI remains the presentation layer.

**Step 4: Run the focused tests to verify pass**

Run:

```bash
cargo test --package upm --test search_cli
cargo test --package upm --test end_to_end_cli
cargo test --package upm --test ui_summary
```

Expected: PASS.

**Step 5: Commit**

```bash
git add crates/upm/Cargo.toml crates/upm/src/providers.rs crates/upm/src/lib.rs crates/upm/tests/search_cli.rs crates/upm/tests/end_to_end_cli.rs crates/upm/tests/ui_summary.rs
git commit -m "refactor: compose providers from upm modules"
```

### Task 6: Update docs and run full workspace verification

**Files:**
- Modify: `README.md`
- Modify: `.architecture/overview.md`
- Modify: `.architecture/roadmap.md`

**Step 1: Update product and architecture docs**

Document:

- the workspace rename to `upm`
- `upm-core` as the headless application layer
- `upm-appimage` as the first installable provider module
- compatibility behavior for existing `aim` config and registry locations
- the fact that provider composition now happens in the CLI rather than through hardcoded AppImage paths in `upm-core`

**Step 2: Verify the docs mention the new structure**

Run:

```bash
rg -n "upm-core|upm-appimage|legacy aim|ProviderRegistry|upm" README.md .architecture/overview.md .architecture/roadmap.md
```

Expected: matches showing the renamed crates, provider split, and compatibility note.

**Step 3: Run the full verification suite**

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
git commit -m "docs: describe upm core and module split"
```