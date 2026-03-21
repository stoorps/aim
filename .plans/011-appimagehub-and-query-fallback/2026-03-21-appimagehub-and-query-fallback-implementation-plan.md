# AppImageHub Provider And Query Fallback Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add AppImageHub as a first-class provider, remove the `custom-json` stub, and make positional `aim <query>` fall back to cross-provider search when no direct installable match exists.

**Architecture:** Keep strict source classification in `aim-core::app::query`, add AppImageHub through the existing source-adapter and search-provider seams, and move positional install-versus-search behavior into a higher-level orchestration path in `aim-cli` and shared app logic. Reuse existing search models and add-plan flows rather than creating parallel provider plumbing.

**Tech Stack:** Rust workspace, Cargo tests, Clap CLI parsing, existing provider adapter/search abstractions, fixture-driven tests.

---

### Task 1: Extend source identity for AppImageHub

**Files:**
- Modify: `crates/aim-core/src/domain/source.rs`
- Modify: `crates/aim-core/src/app/query.rs`
- Test: `crates/aim-core/tests/query_resolution.rs`

**Step 1: Write the failing tests**

Add tests covering:

- `resolve_query("https://www.appimagehub.com/p/2338455")`
- `resolve_query("appimagehub/2338455")`
- malformed shorthand such as `appimagehub/firefox`

Expected assertions:

- `SourceKind::AppImageHub`
- appropriate `SourceInputKind`
- `NormalizedSourceKind::AppImageHub`
- canonical locator `Some("2338455")`

**Step 2: Run the focused tests to verify failure**

Run: `cargo test --package aim-core --test query_resolution`

Expected: FAIL with unknown source kinds or unsupported AppImageHub inputs.

**Step 3: Implement the minimal source model changes**

Update enums and `as_str()` mappings in `crates/aim-core/src/domain/source.rs`, then extend query classification so AppImageHub URLs and `appimagehub/<id>` normalize into a stable `SourceRef`.

**Step 4: Run the focused tests to verify pass**

Run: `cargo test --package aim-core --test query_resolution`

Expected: PASS for the new AppImageHub cases and existing provider cases.

**Step 5: Commit**

```bash
git add crates/aim-core/src/domain/source.rs crates/aim-core/src/app/query.rs crates/aim-core/tests/query_resolution.rs
git commit -m "feat: classify AppImageHub sources"
```

### Task 2: Add the AppImageHub source adapter and remove `custom-json`

**Files:**
- Create: `crates/aim-core/src/adapters/appimagehub.rs`
- Modify: `crates/aim-core/src/adapters/mod.rs`
- Delete: `crates/aim-core/src/adapters/custom_json.rs`
- Test: `crates/aim-core/tests/adapter_smoke.rs`
- Test: `crates/aim-core/tests/adapter_contract.rs`

**Step 1: Write the failing tests**

Add adapter expectations for:

- `all_adapter_kinds()` contains `"appimagehub"`
- `all_adapter_kinds()` no longer contains `"custom-json"`
- AppImageHub adapter supports exact resolution and search

**Step 2: Run the focused tests to verify failure**

Run: `cargo test --package aim-core --test adapter_smoke --test adapter_contract`

Expected: FAIL because AppImageHub is not registered and `custom-json` is still present.

**Step 3: Implement the adapter registration changes**

Create `appimagehub.rs` with `SourceAdapter` support for AppImageHub sources, wire it into `mod.rs`, and remove `custom-json` from module registration and adapter-kind reporting.

**Step 4: Run the focused tests to verify pass**

Run: `cargo test --package aim-core --test adapter_smoke --test adapter_contract`

Expected: PASS with AppImageHub present and `custom-json` removed.

**Step 5: Commit**

```bash
git add crates/aim-core/src/adapters/appimagehub.rs crates/aim-core/src/adapters/mod.rs crates/aim-core/tests/adapter_smoke.rs crates/aim-core/tests/adapter_contract.rs
git rm crates/aim-core/src/adapters/custom_json.rs
git commit -m "feat: add AppImageHub adapter"
```

### Task 3: Add AppImageHub transport-backed resolution

**Files:**
- Modify: `crates/aim-core/src/adapters/appimagehub.rs`
- Create or Modify: `crates/aim-core/src/source/appimagehub.rs`
- Modify: `crates/aim-core/src/app/add.rs`
- Test: `crates/aim-core/tests/install_payload.rs`
- Test: `crates/aim-core/tests/adapter_contract.rs`

**Step 1: Write the failing tests**

Add fixture-backed tests for:

- resolving `appimagehub/<id>` into an installable AppImage artifact URL
- returning `NoInstallableArtifact` when the item exists but exposes no installable AppImage asset

**Step 2: Run the focused tests to verify failure**

Run: `cargo test --package aim-core --test adapter_contract --test install_payload`

Expected: FAIL because AppImageHub resolution is not implemented.

**Step 3: Implement the transport and resolution path**

Add the minimal AppImageHub/OCS transport wrapper, teach the adapter to resolve the latest installable artifact, and update add-plan selection to route AppImageHub through the adapter path rather than the generic fallback branch.

**Step 4: Run the focused tests to verify pass**

Run: `cargo test --package aim-core --test adapter_contract --test install_payload`

Expected: PASS with deterministic fixture-backed AppImageHub resolution.

**Step 5: Commit**

```bash
git add crates/aim-core/src/adapters/appimagehub.rs crates/aim-core/src/source/appimagehub.rs crates/aim-core/src/app/add.rs crates/aim-core/tests/adapter_contract.rs crates/aim-core/tests/install_payload.rs
git commit -m "feat: resolve AppImageHub artifacts"
```

### Task 4: Add AppImageHub search provider integration

**Files:**
- Modify: `crates/aim-core/src/app/search.rs`
- Create or Modify: `crates/aim-core/src/source/appimagehub.rs`
- Test: `crates/aim-core/tests/query_resolution.rs`
- Test: `crates/aim-core/tests/github_source_discovery.rs`
- Create or Modify: `crates/aim-core/tests/appimagehub_search.rs`

**Step 1: Write the failing tests**

Add fixture-backed search tests covering:

- AppImageHub hit mapping into `SearchResult`
- `install_query = appimagehub/<id>`
- canonical locator matching for installed-status annotation
- mixed-provider search returning GitHub and AppImageHub hits together

**Step 2: Run the focused tests to verify failure**

Run: `cargo test --package aim-core --test appimagehub_search`

Expected: FAIL because AppImageHub search is not wired into the provider list.

**Step 3: Implement the provider integration**

Add an AppImageHub `SearchProvider`, wire it into `build_search_results(...)`, and update installed-hit annotation logic so AppImageHub-installed apps are recognized by canonical ID.

**Step 4: Run the focused tests to verify pass**

Run: `cargo test --package aim-core --test appimagehub_search`

Expected: PASS with deterministic AppImageHub search coverage.

**Step 5: Commit**

```bash
git add crates/aim-core/src/app/search.rs crates/aim-core/src/source/appimagehub.rs crates/aim-core/tests/appimagehub_search.rs
git commit -m "feat: add AppImageHub search provider"
```

### Task 5: Add positional query fallback from install to search

**Files:**
- Modify: `crates/aim-cli/src/lib.rs`
- Modify: `crates/aim-core/src/app/add.rs`
- Modify: `crates/aim-core/src/app/search.rs`
- Test: `crates/aim-cli/tests/end_to_end_cli.rs`
- Test: `crates/aim-cli/tests/cli_smoke.rs`

**Step 1: Write the failing tests**

Add CLI/app-flow tests covering:

- `aim firefox` falls back to search results when direct resolution is unsupported
- positional query falls back to search results when a provider item has no installable artifact
- positional query still installs directly for valid direct provider inputs

**Step 2: Run the focused tests to verify failure**

Run: `cargo test --package aim-cli --test end_to_end_cli --test cli_smoke`

Expected: FAIL because positional queries still surface add errors instead of search results.

**Step 3: Implement the orchestration change**

Add a small decision path in dispatch or shared app logic that tries the add flow first, then converts `Unsupported` and `NoInstallableArtifact` outcomes into `SearchResults` for positional queries only.

**Step 4: Run the focused tests to verify pass**

Run: `cargo test --package aim-cli --test end_to_end_cli --test cli_smoke`

Expected: PASS with positional-query fallback behavior.

**Step 5: Commit**

```bash
git add crates/aim-cli/src/lib.rs crates/aim-core/src/app/add.rs crates/aim-core/src/app/search.rs crates/aim-cli/tests/end_to_end_cli.rs crates/aim-cli/tests/cli_smoke.rs
git commit -m "feat: fall back to search for positional queries"
```

### Task 6: Update CLI rendering and messaging for fallback search

**Files:**
- Modify: `crates/aim-cli/src/ui/render.rs`
- Modify: `crates/aim-cli/src/ui/theme.rs`
- Test: `crates/aim-cli/tests/ui_summary.rs`
- Test: `crates/aim-cli/tests/cli_commands.rs`

**Step 1: Write the failing tests**

Add renderer expectations for:

- fallback search rendering from positional `aim <query>`
- empty search state instead of `unsupported source query`
- AppImageHub provider labels and install query formatting where they are visible

**Step 2: Run the focused tests to verify failure**

Run: `cargo test --package aim-cli --test ui_summary --test cli_commands`

Expected: FAIL because fallback-search rendering and AppImageHub labels are not represented.

**Step 3: Implement the minimal rendering changes**

Reuse existing search rendering as much as possible, only adjusting dispatch/result handling and any provider-label formatting needed for AppImageHub.

**Step 4: Run the focused tests to verify pass**

Run: `cargo test --package aim-cli --test ui_summary --test cli_commands`

Expected: PASS with stable search output for fallback scenarios.

**Step 5: Commit**

```bash
git add crates/aim-cli/src/ui/render.rs crates/aim-cli/src/ui/theme.rs crates/aim-cli/tests/ui_summary.rs crates/aim-cli/tests/cli_commands.rs
git commit -m "feat: render AppImageHub and fallback search results"
```

### Task 7: Update docs and run full verification

**Files:**
- Modify: `README.md`
- Modify: `.plans/011-appimagehub-and-query-fallback/2026-03-21-appimagehub-and-query-fallback-design.md` if implementation details drift
- Modify: `.plans/011-appimagehub-and-query-fallback/2026-03-21-appimagehub-and-query-fallback-implementation-plan.md` if task wording needs correction

**Step 1: Update user-facing docs**

Document:

- AppImageHub direct query forms
- positional-query fallback-to-search behavior
- removal of `custom-json` from the supported-provider story

**Step 2: Run formatting and full verification**

Run:

```bash
cargo fmt --all
cargo test --workspace
cargo clippy --workspace --all-targets --all-features -- -D warnings
```

Expected: all commands succeed.

**Step 3: Commit**

```bash
git add README.md .plans/011-appimagehub-and-query-fallback/2026-03-21-appimagehub-and-query-fallback-design.md .plans/011-appimagehub-and-query-fallback/2026-03-21-appimagehub-and-query-fallback-implementation-plan.md
git commit -m "docs: document AppImageHub provider and query fallback"
```