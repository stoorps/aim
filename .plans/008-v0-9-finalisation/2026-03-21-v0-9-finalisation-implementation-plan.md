# v0.9 Finalisation Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Finalise v0.9 by hardening artifact downloads, enforcing checksum verification, making registry mutation safe, and shipping provider-extensible search with GitHub as the first provider while keeping `custom-json` deferred to v1.0.

**Architecture:** Push download, integrity, and registry safety down into `aim-core`, keep `aim-cli` as a thin dispatch and rendering layer, and add a provider-neutral search app flow that can grow to additional providers in phase 2 without changing the CLI contract.

**Tech Stack:** Rust, Cargo workspace, `aim-core` app/source/registry modules, `aim-cli` command parsing and rendering, reqwest blocking client, existing fixture-backed GitHub tests, CLI integration tests.

---

### Task 1: Lock the v0.9 CLI and docs contract with failing tests

**Files:**
- Modify: `crates/aim-cli/src/cli/args.rs`
- Modify: `crates/aim-cli/tests/cli_commands.rs`
- Modify: `README.md`
- Modify: `.plans/008-v0-9-finalisation/2026-03-21-v0-9-finalisation-design.md`

**Step 1: Write the failing tests**

Extend CLI help coverage so `--help` is expected to include `search`.

**Step 2: Run test to verify it fails**

Run: `cargo test --package aim-cli --test cli_commands`
Expected: FAIL because the CLI does not yet expose `search`.

**Step 3: Write minimal implementation**

Add a `Search { query: String }` subcommand to the CLI args and update the README command/query documentation to reflect:

- `aim search <query>` exists in v0.9
- SourceForge support is documented honestly
- search is GitHub-backed first and provider-extensible, not multi-provider complete
- `custom-json` is not presented as a v0.9 feature

**Step 4: Run test to verify it passes**

Run: `cargo test --package aim-cli --test cli_commands`
Expected: PASS.

**Step 5: Commit**

```bash
git add crates/aim-cli/src/cli/args.rs crates/aim-cli/tests/cli_commands.rs README.md .plans/008-v0-9-finalisation/2026-03-21-v0-9-finalisation-design.md
git commit -m "feat: add v0.9 search command contract"
```

### Task 2: Add failing GitHub search tests and a provider-neutral search model

**Files:**
- Create: `crates/aim-core/src/app/search.rs`
- Modify: `crates/aim-core/src/app/mod.rs`
- Create: `crates/aim-core/src/domain/search.rs`
- Modify: `crates/aim-core/src/domain/mod.rs`
- Modify: `crates/aim-core/src/source/github.rs`
- Create: `crates/aim-core/tests/search_github.rs`

**Step 1: Write the failing tests**

Add search tests that assert:

- GitHub fixtures can return normalized remote search hits
- normalized results include provider id, display name, description, homepage/source locator, and install-ready query
- the app-level search result type can also carry installed/local matches and warnings
- default remote result limit is 10
- the install-ready query is canonical `owner/repo`
- remote hit ordering is stable under fixtures

**Step 2: Run test to verify it fails**

Run: `cargo test --package aim-core --test search_github`
Expected: FAIL because no search domain or app flow exists yet.

**Step 3: Write minimal implementation**

Add provider-neutral search domain types and the app-level search entry point in `aim-core`. Extend the GitHub source transport with the smallest search capability needed for fixture-backed repository search.

Keep the model narrow:

- one provider trait or equivalent provider entry shape
- one normalized result type
- no premature provider-specific knobs beyond what GitHub needs now

For v0.9, make the GitHub provider search repositories only. Preserve provider ranking order from fixtures or transport results, and use canonical locator as the deterministic tie-breaker when a secondary sort is needed.

**Step 4: Run test to verify it passes**

Run: `cargo test --package aim-core --test search_github`
Expected: PASS.

**Step 5: Commit**

```bash
git add crates/aim-core/src/app/search.rs crates/aim-core/src/app/mod.rs crates/aim-core/src/domain/search.rs crates/aim-core/src/domain/mod.rs crates/aim-core/src/source/github.rs crates/aim-core/tests/search_github.rs
git commit -m "feat: add provider-neutral search core"
```

### Task 3: Wire `aim search` through dispatch and rendering

**Files:**
- Modify: `crates/aim-cli/src/lib.rs`
- Modify: `crates/aim-cli/src/ui/render.rs`
- Create: `crates/aim-cli/tests/search_cli.rs`

**Step 1: Write the failing tests**

Add CLI integration tests that assert:

- `aim search bat` prints a `Search Results` heading
- remote GitHub hits render with provider label and install-ready query
- installed/local matches render in a separate section when the registry contains matching apps
- installed/local matches use case-insensitive substring matching across `stable_id` and `display_name`
- installed/local matches are sorted deterministically by exact match, prefix match, substring match, then `stable_id`
- search remains read-only and does not mutate the registry

**Step 2: Run test to verify it fails**

Run: `cargo test --package aim-cli --test search_cli`
Expected: FAIL because dispatch and render do not know about search yet.

**Step 3: Write minimal implementation**

Add search dispatch to `aim-cli`, call into the new `aim-core` search flow, load registry state for local-match context, and render a stable read-only summary.

Do not add interactive selection or install-from-search behavior.

**Step 4: Run test to verify it passes**

Run: `cargo test --package aim-cli --test search_cli`
Expected: PASS.

**Step 5: Commit**

```bash
git add crates/aim-cli/src/lib.rs crates/aim-cli/src/ui/render.rs crates/aim-cli/tests/search_cli.rs
git commit -m "feat: wire search through cli"
```

### Task 4: Add staged-download tests before changing the artifact pipeline

**Files:**
- Modify: `crates/aim-core/src/app/add.rs`
- Modify: `crates/aim-core/src/integration/install.rs`
- Modify: `crates/aim-core/tests/install_integration.rs`
- Create: `crates/aim-core/tests/download_pipeline.rs`

**Step 1: Write the failing tests**

Add tests that assert:

- artifact download can stream into a staged path instead of returning full in-memory bytes
- the staged file reaches full byte count and still emits progress
- the install path can commit from a staged file source
- a failed download attempt does not leave a partial staged payload behind

Use fixture or local test doubles rather than real network calls.

**Step 2: Run test to verify it fails**

Run: `cargo test --package aim-core --test download_pipeline`
Expected: FAIL because the current add flow still downloads into `Vec<u8>`.

**Step 3: Write minimal implementation**

Refactor the add/install boundary so downloads are streamed into a staged file and the install integration path commits from disk.

Keep existing operation stages and user-facing progress events intact.

Make cleanup deterministic: if streaming, payload validation, or post-download verification fails, the staged file must be removed before returning the error.

**Step 4: Run test to verify it passes**

Run: `cargo test --package aim-core --test download_pipeline`
Expected: PASS.

**Step 5: Commit**

```bash
git add crates/aim-core/src/app/add.rs crates/aim-core/src/integration/install.rs crates/aim-core/tests/install_integration.rs crates/aim-core/tests/download_pipeline.rs
git commit -m "refactor: stream artifacts into staged payloads"
```

### Task 5: Add retry and timeout policy to the download client

**Files:**
- Modify: `crates/aim-core/src/app/add.rs`
- Modify: `crates/aim-core/src/source/github.rs`
- Modify: `crates/aim-core/tests/download_pipeline.rs`
- Modify: `crates/aim-core/tests/github_source_discovery.rs`

**Step 1: Write the failing tests**

Add focused tests that assert:

- the shared HTTP client is constructed with explicit timeout behavior
- download retries transient failures according to policy
- exhausted retries surface a clear failure
- retry exhaustion does not leave a staged payload behind

Prefer test doubles around client-building or download helpers over brittle timing assertions.

**Step 2: Run test to verify it fails**

Run: `cargo test --package aim-core --test download_pipeline`
Expected: FAIL because timeout and retry policy are not represented yet.

**Step 3: Write minimal implementation**

Introduce a small shared download client or helper configuration that both GitHub discovery and artifact download can use. Add explicit timeout configuration and retry loops for transient failures.

Do not add resume support in this slice unless it falls out naturally from the refactor; timeout and retry are the required behaviors.

Make the timeout contract explicit in code and tests. The implementation does not need user-facing configurability in v0.9, but it must use fixed non-infinite defaults.

**Step 4: Run test to verify it passes**

Run: `cargo test --package aim-core --test download_pipeline`
Expected: PASS.

**Step 5: Commit**

```bash
git add crates/aim-core/src/app/add.rs crates/aim-core/src/source/github.rs crates/aim-core/tests/download_pipeline.rs crates/aim-core/tests/github_source_discovery.rs
git commit -m "feat: add timeout and retry policy to downloads"
```

### Task 6: Enforce checksum verification on install

**Files:**
- Modify: `crates/aim-core/src/app/add.rs`
- Modify: `crates/aim-core/src/integration/install.rs`
- Modify: `crates/aim-core/src/domain/update.rs`
- Modify: `crates/aim-core/tests/install_integration.rs`
- Create: `crates/aim-core/tests/checksum_verification.rs`

**Step 1: Write the failing tests**

Add tests that assert:

- installs with a valid checksum succeed
- installs with a checksum mismatch fail before final payload commit
- installs without a checksum still succeed
- malformed trusted checksums fail before final payload commit
- checksum failure does not leave a staged payload behind

Use fixture metadata with deterministic payload bytes.

**Step 2: Run test to verify it fails**

Run: `cargo test --package aim-core --test checksum_verification`
Expected: FAIL because checksum hints are parsed but never enforced.

**Step 3: Write minimal implementation**

Thread checksum hints through artifact selection into install execution and verify the staged payload before the final rename. Surface a typed install failure for mismatch.

For v0.9, implement only the existing electron-builder checksum contract: compare the base64-encoded SHA-512 digest of the raw staged payload bytes against the trimmed `sha512` metadata value. Treat malformed trusted checksum input as an install failure.

**Step 4: Run test to verify it passes**

Run: `cargo test --package aim-core --test checksum_verification`
Expected: PASS.

**Step 5: Commit**

```bash
git add crates/aim-core/src/app/add.rs crates/aim-core/src/integration/install.rs crates/aim-core/src/domain/update.rs crates/aim-core/tests/install_integration.rs crates/aim-core/tests/checksum_verification.rs
git commit -m "feat: verify artifact checksum before install"
```

### Task 7: Make registry mutation atomic and locked

**Files:**
- Modify: `crates/aim-core/src/registry/store.rs`
- Modify: `crates/aim-cli/src/lib.rs`
- Create: `crates/aim-core/tests/registry_store.rs`
- Modify: `Cargo.toml`
- Modify: `crates/aim-core/Cargo.toml`

**Step 1: Write the failing tests**

Add registry store tests that assert:

- saves write through a temp file and leave the final registry valid
- concurrent mutating access cannot silently race
- lock acquisition failure surfaces a clear error
- the locked mutation path reloads the latest registry before applying the final mutation
- read-only flows do not require the mutation lock

**Step 2: Run test to verify it fails**

Run: `cargo test --package aim-core --test registry_store`
Expected: FAIL because save is a direct `fs::write` with no lock semantics.

**Step 3: Write minimal implementation**

Implement atomic save and advisory locking in the registry store. Thread any needed lock lifecycle changes through the CLI mutating commands.

Keep read-only flows simple and avoid unnecessary lock retention.

The lock scope must be deterministic:

- do not hold the registry lock during network discovery, downloads, or desktop integration
- acquire the exclusive lock immediately before the final registry transaction
- reload the latest registry while the lock is held
- apply the mutation by `stable_id`
- save atomically, then release the lock

If a remove target disappears between pre-lock planning and the locked reload, fail cleanly instead of silently removing the wrong record.

**Step 4: Run test to verify it passes**

Run: `cargo test --package aim-core --test registry_store`
Expected: PASS.

**Step 5: Commit**

```bash
git add crates/aim-core/src/registry/store.rs crates/aim-cli/src/lib.rs crates/aim-core/tests/registry_store.rs Cargo.toml crates/aim-core/Cargo.toml
git commit -m "feat: add atomic and locked registry mutation"
```

### Task 8: Run provider regression coverage and final verification

**Files:**
- Modify: `README.md`
- Modify: `.plans/008-v0-9-finalisation/2026-03-21-v0-9-finalisation-implementation-plan.md`

**Step 1: Add any missing regression expectations**

If provider or CLI regressions appear during execution, add the smallest missing focused tests in the existing suites:

- `crates/aim-core/tests/query_resolution.rs`
- `crates/aim-core/tests/adapter_contract.rs`
- `crates/aim-core/tests/install_integration.rs`
- `crates/aim-core/tests/update_planning.rs`
- `crates/aim-cli/tests/end_to_end_cli.rs`

**Step 2: Run focused verification**

Run:

```bash
cargo test --package aim-core --test search_github --test download_pipeline --test checksum_verification --test registry_store --test install_integration
cargo test --package aim-cli --test cli_commands --test search_cli --test end_to_end_cli
```

Expected: PASS.

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
git add README.md .plans/008-v0-9-finalisation/2026-03-21-v0-9-finalisation-implementation-plan.md crates/
git commit -m "feat: finalize v0.9 reliability and search"
```
