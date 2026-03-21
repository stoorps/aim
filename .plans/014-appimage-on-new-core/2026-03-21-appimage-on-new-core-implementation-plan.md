# AppImage On The New Core Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Make `upm-appimage` the first real package-manager module by moving AppImage-specific acquisition paths behind a module boundary and exposing a single application facade from `upm-core` to the CLI and future GUI.

**Architecture:** `upm-core` becomes the application boundary. It should own a public facade, internal orchestration services, and module registration and composition. `upm` stays a thin frontend and must stop composing AppImage behavior directly. `upm-appimage` becomes the AppImage package-manager module and should absorb AppImageHub plus the other AppImage-producing backends that are still modeled as top-level source concepts in the core.

**Tech Stack:** Rust workspace, `upm`, `upm-core`, `upm-appimage`, Cargo integration tests, fixture-backed provider tests, CLI end-to-end tests.

---

### Task 1: Define the public application facade in `upm-core`

**Files:**
- Modify: `crates/upm-core/src/app/`
- Modify: `crates/upm-core/src/lib.rs`
- Test: `crates/upm-core/tests/`

**Step 1: Write the failing facade expectations**

Add focused tests proving that:

- `upm-core` exposes one public application-facing entrypoint for frontend consumers
- that entrypoint can be constructed without the CLI owning module composition
- the public surface delegates to internal services instead of exposing module wiring details

Keep the assertions about API shape and ownership, not AppImage specifics.

**Step 2: Run the focused tests to verify failure**

Run:

```bash
cargo test --package upm-core
```

Expected: FAIL because the current public surface still assumes narrower provider plumbing and does not expose the intended facade cleanly.

**Step 3: Implement the minimal facade**

Introduce or reshape the public API so `upm-core` exposes a single high-level application facade.

The public boundary should:

- represent product operations such as search, add, show, update, remove, and config handling
- hide module registry and orchestration details from frontends
- stay thin and delegate work to internal services

Do not add dynamic plugin loading yet.

**Step 4: Run the focused tests to verify pass**

Run:

```bash
cargo test --package upm-core
```

Expected: PASS.

**Step 5: Commit**

```bash
git add crates/upm-core/src crates/upm-core/tests
git commit -m "feat: add application facade to upm-core"
```

### Task 2: Move module composition out of the CLI and into `upm-core`

**Files:**
- Modify: `crates/upm-core/src/app/`
- Modify: `crates/upm/src/lib.rs`
- Modify: `crates/upm/src/providers.rs`
- Test: `crates/upm-core/tests/`
- Test: `crates/upm/tests/end_to_end_cli.rs`

**Step 1: Write the failing ownership expectations**

Add coverage proving that:

- the CLI does not assemble AppImage module composition directly
- the application facade can build or receive module composition internally
- CLI command paths still behave the same through the new boundary

Prefer one core ownership test and one CLI integration test.

**Step 2: Run the focused tests to verify failure**

Run:

```bash
cargo test --package upm --test end_to_end_cli
```

Expected: FAIL because the CLI still owns direct provider assembly.

**Step 3: Move composition into `upm-core`**

Update the architecture so:

- `upm-core` owns module registration and composition
- the CLI constructs the application facade rather than AppImage-specific registries
- direct module composition in `crates/upm/src/providers.rs` is removed or reduced to generic application bootstrapping

Keep CLI UX, rendering, and summary formatting unchanged.

**Step 4: Run the focused tests to verify pass**

Run:

```bash
cargo test --package upm --test end_to_end_cli
```

Expected: PASS.

**Step 5: Commit**

```bash
git add crates/upm-core/src crates/upm/src/lib.rs crates/upm/src/providers.rs crates/upm/tests/end_to_end_cli.rs
git commit -m "refactor: move module composition into upm-core"
```

### Task 3: Turn `upm-appimage` into the AppImage package-manager boundary

**Files:**
- Modify: `crates/upm-appimage/src/`
- Modify: `crates/upm-core/src/domain/source.rs`
- Modify: `crates/upm-core/src/app/add.rs`
- Modify: `crates/upm-core/src/app/show.rs`
- Modify: `crates/upm-core/src/app/update.rs`
- Test: `crates/upm-appimage/tests/`
- Test: `crates/upm-core/tests/`

**Step 1: Write the failing module-boundary expectations**

Add coverage proving that:

- AppImage-backed acquisition through GitHub, GitLab, SourceForge, direct URLs, and AppImageHub resolves through `upm-appimage`
- `upm-core` no longer treats those AppImage-producing backends as top-level package-manager concepts
- add, show, and update continue to work through normalized module contracts

Prefer module-focused tests plus a small number of core integration tests.

**Step 2: Run the focused tests to verify failure**

Run:

```bash
cargo test --package upm-appimage
cargo test --package upm-core
```

Expected: FAIL because AppImage acquisition paths are still split between the core and the AppImage module.

**Step 3: Move AppImage-specific acquisition logic behind the module**

Reshape the source and module boundary so:

- AppImage-specific GitHub, GitLab, SourceForge, AppImageHub, and direct URL handling lives in `upm-appimage`
- `upm-core` coordinates AppImage work through normalized module contracts
- core source taxonomy is reduced or reframed so package-manager concepts stay above backend minutia

Do not over-generalize for Flatpak or future providers yet. Only extract what the AppImage module demonstrably needs.

**Step 4: Run the focused tests to verify pass**

Run:

```bash
cargo test --package upm-appimage
cargo test --package upm-core
```

Expected: PASS.

**Step 5: Commit**

```bash
git add crates/upm-appimage/src crates/upm-core/src crates/upm-appimage/tests crates/upm-core/tests
git commit -m "refactor: make upm-appimage the appimage module boundary"
```

### Task 4: Route search, add, show, and update through the application facade

**Files:**
- Modify: `crates/upm-core/src/app/`
- Modify: `crates/upm/src/lib.rs`
- Modify: `crates/upm/tests/end_to_end_cli.rs`
- Modify: `crates/upm/tests/ui_summary.rs`
- Test: `crates/upm-core/tests/`

**Step 1: Write the failing facade-routing expectations**

Add end-to-end coverage proving that AppImage support is fully module-driven:

- `search` flows through the public application facade
- `add` flows through the public application facade
- `show` flows through the public application facade
- `update` flows through the public application facade
- user-facing summaries still render truthful `upm` paths and origins

Keep the assertions focused on boundary correctness rather than UI restyling.

**Step 2: Run the focused tests to verify failure**

Run:

```bash
cargo test --package upm --test end_to_end_cli
cargo test --package upm --test ui_summary
```

Expected: FAIL until the facade is the normal command path.

**Step 3: Tighten application-boundary validation**

Update the tests so they prove:

- frontend command handlers call the application facade rather than module-specific helpers
- AppImage is composed only through `upm-core` and `upm-appimage`
- AppImage is not reintroduced as a hardcoded built-in in the CLI

Do not reintroduce package-manager-specific branching in the CLI.

**Step 4: Run the focused tests to verify pass**

Run:

```bash
cargo test --package upm --test end_to_end_cli
cargo test --package upm --test ui_summary
```

Expected: PASS.

**Step 5: Commit**

```bash
git add crates/upm-core/src crates/upm/src/lib.rs crates/upm/tests/end_to_end_cli.rs crates/upm/tests/ui_summary.rs crates/upm-core/tests
git commit -m "refactor: route commands through upm-core facade"
```

### Task 5: Lock the architecture into docs and verify the workspace

**Files:**
- Modify: `.architecture/overview.md`
- Modify: `.architecture/roadmap.md`
- Modify: `README.md`

**Step 1: Update docs for the new module model**

Document:

- `upm-core` as the application boundary
- one public application facade over smaller internal services
- CLI and GUI as thin frontends
- `upm-appimage` as the AppImage package-manager boundary with internal acquisition backends

**Step 2: Verify the docs mention the agreed architecture**

Run:

```bash
rg -n "upm-core|facade|upm-ui|upm-appimage|module" README.md .architecture/overview.md .architecture/roadmap.md
```

Expected: matches describing the application boundary, thin frontends, and module ownership.

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
git commit -m "docs: describe application facade architecture"
```