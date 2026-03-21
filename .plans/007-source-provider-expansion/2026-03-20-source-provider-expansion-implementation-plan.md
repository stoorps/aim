# Source And Provider Expansion Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Make GitLab and SourceForge real repository-backed install sources, preserve direct URL as a first-class exact-resolution source, and keep zsync as update metadata rather than an install provider.

**Architecture:** Normalize the source taxonomy first, then add a capability-shaped resolver layer that distinguishes repository-backed sources from exact artifact sources. Preserve truthful install origin data in the registry and let update planning attach richer metadata without rewriting source identity.

**Tech Stack:** Rust, Cargo workspace, `aim-core` source and adapter modules, existing fixture-backed integration tests in `crates/aim-core/tests`, CLI end-to-end tests in `crates/aim-cli/tests`, existing registry and update planning code.

## Follow-up Status

Task 1 hit a classification ambiguity blocker after the initial rollout. The follow-up design and execution live in `.plans/007-source-provider-expansion/2026-03-20-task-1-ambiguity-handoff-addendum.md`.

Current state on this branch:

- ambiguous GitLab deep paths and one SourceForge nested download path are now preserved as provider-owned candidate kinds during classification
- the first GitLab candidate slice now resolves as a concrete repository-backed install source at the adapter layer
- the SourceForge `files/releases/stable/download` candidate slice now resolves as a concrete latest-download install source at the adapter layer
- the SourceForge `files/releases/v*/download` slice is now preserved as a provider-owned candidate and reports `NoInstallableArtifact`
- unsupported queries remain distinct from provider-owned no-artifact outcomes

Classifier policy for follow-up work:

- accept explicit concrete shapes
- accept explicit provider-candidate shapes
- reject everything else

Future changes should expand the allowlist deliberately rather than adding broad negative-rule coverage for every unsupported provider page family.

---

### Task 1: Lock down source taxonomy with failing classification tests

**Files:**
- Modify: `crates/aim-core/tests/query_resolution.rs`
- Modify: `crates/aim-core/src/source/input.rs`
- Modify: `crates/aim-core/src/domain/source.rs`

**Step 1: Write the failing tests**

Add classification tests that cover:
- GitLab repository and release-like URL forms that should classify as `GitLab`
- supported SourceForge URL forms that should classify as `SourceForge`
- direct URLs that must remain `DirectUrl`
- malformed provider URLs that must fail as unsupported

**Step 2: Run test to verify it fails**

Run: `cargo test --package aim-core --test query_resolution`
Expected: FAIL because SourceForge is not yet part of the public source taxonomy and current classification rules are too narrow.

**Step 3: Write minimal classification changes**

Update the source domain and classifier so the public source taxonomy includes the approved source kinds and supported input forms without introducing zsync as an install source.

**Step 4: Run test to verify it passes**

Run: `cargo test --package aim-core --test query_resolution`
Expected: PASS.

**Step 5: Commit**

```bash
git add crates/aim-core/tests/query_resolution.rs crates/aim-core/src/source/input.rs crates/aim-core/src/domain/source.rs
git commit -m "test: cover expanded source taxonomy"
```

### Task 2: Add a shared resolver contract for source capabilities

**Files:**
- Modify: `crates/aim-core/src/adapters/traits.rs`
- Modify: `crates/aim-core/src/adapters/mod.rs`
- Modify: `crates/aim-core/tests/adapter_contract.rs`
- Modify: `crates/aim-core/src/app/query.rs`

**Step 1: Write the failing tests**

Add contract tests that assert:
- repository-backed resolvers accept only their own source kinds
- exact-resolution resolvers accept only exact artifact kinds
- resolvers can return structured “no installable artifact” outcomes rather than collapsing to unsupported

**Step 2: Run test to verify it fails**

Run: `cargo test --package aim-core --test adapter_contract`
Expected: FAIL because the current adapter trait does not distinguish source capability outcomes cleanly enough.

**Step 3: Write minimal resolver contract changes**

Refine the shared adapter or resolver contract to represent:
- unsupported source kind
- supported source with successful artifact resolution
- supported source with no installable artifact found

Keep the API small and do not add terminal concerns.

**Step 4: Run test to verify it passes**

Run: `cargo test --package aim-core --test adapter_contract`
Expected: PASS.

**Step 5: Commit**

```bash
git add crates/aim-core/src/adapters/traits.rs crates/aim-core/src/adapters/mod.rs crates/aim-core/tests/adapter_contract.rs crates/aim-core/src/app/query.rs
git commit -m "feat: add capability-shaped resolver contract"
```

### Task 3: Make GitLab a real repository-backed install source

**Files:**
- Modify: `crates/aim-core/src/adapters/gitlab.rs`
- Modify: `crates/aim-core/src/app/add.rs`
- Modify: `crates/aim-core/tests/adapter_contract.rs`
- Modify: `crates/aim-core/tests/install_integration.rs`
- Modify: `crates/aim-cli/tests/end_to_end_cli.rs`

**Step 1: Write the failing tests**

Add tests that assert:
- a GitLab source resolves to a concrete install candidate
- install flow persists a truthful GitLab install origin
- CLI integration can install a fixture-backed GitLab source end to end

**Step 2: Run test to verify it fails**

Run: `cargo test --package aim-core --test install_integration`
Expected: FAIL because GitLab resolution is currently placeholder-level and not wired into the add flow meaningfully.

**Step 3: Write minimal implementation**

Implement GitLab-specific repository-backed resolution using the new resolver contract and thread the result through the add flow without changing direct URL or zsync semantics.

**Step 4: Run test to verify it passes**

Run: `cargo test --package aim-core --test install_integration`
Expected: PASS.

**Step 5: Commit**

```bash
git add crates/aim-core/src/adapters/gitlab.rs crates/aim-core/src/app/add.rs crates/aim-core/tests/adapter_contract.rs crates/aim-core/tests/install_integration.rs crates/aim-cli/tests/end_to_end_cli.rs
git commit -m "feat: add gitlab install source resolution"
```

### Task 4: Preserve direct URL as an exact-resolution source

**Files:**
- Modify: `crates/aim-core/src/adapters/direct_url.rs`
- Modify: `crates/aim-core/src/app/add.rs`
- Modify: `crates/aim-core/tests/install_integration.rs`
- Modify: `crates/aim-cli/tests/end_to_end_cli.rs`

**Step 1: Write the failing tests**

Add tests that assert:
- direct URL installs continue to resolve exactly to the provided artifact
- registry persistence keeps the original direct URL source kind and locator
- no provider-like reclassification occurs after install

**Step 2: Run test to verify it fails**

Run: `cargo test --package aim-core --test install_integration`
Expected: FAIL if the new resolver contract or registry changes accidentally regress exact-resolution behavior.

**Step 3: Write minimal implementation**

Adjust the direct URL path to use the new resolver interfaces while preserving exact-resolution semantics and best-effort metadata only.

**Step 4: Run test to verify it passes**

Run: `cargo test --package aim-core --test install_integration`
Expected: PASS.

**Step 5: Commit**

```bash
git add crates/aim-core/src/adapters/direct_url.rs crates/aim-core/src/app/add.rs crates/aim-core/tests/install_integration.rs crates/aim-cli/tests/end_to_end_cli.rs
git commit -m "feat: preserve direct url exact resolution semantics"
```

### Task 5: Add SourceForge as a repository-backed source for supported project forms

**Files:**
- Modify: `crates/aim-core/src/adapters/sourceforge.rs`
- Modify: `crates/aim-core/src/source/input.rs`
- Modify: `crates/aim-core/src/app/add.rs`
- Modify: `crates/aim-core/tests/adapter_contract.rs`
- Modify: `crates/aim-core/tests/install_integration.rs`
- Modify: `crates/aim-cli/tests/end_to_end_cli.rs`

**Step 1: Write the failing tests**

Add tests that assert:
- supported SourceForge URL or project forms classify correctly
- SourceForge resolution can produce a concrete install candidate
- SourceForge installs persist truthful origin data

**Step 2: Run test to verify it fails**

Run: `cargo test --package aim-core --test adapter_contract`
Expected: FAIL because SourceForge currently returns unsupported from its adapter.

**Step 3: Write minimal implementation**

Implement only the supported SourceForge project or download forms needed for exact current-product scope. Return structured no-artifact failures for valid-but-non-installable projects.

**Step 4: Run test to verify it passes**

Run: `cargo test --package aim-core --test adapter_contract`
Expected: PASS.

**Step 5: Commit**

```bash
git add crates/aim-core/src/adapters/sourceforge.rs crates/aim-core/src/source/input.rs crates/aim-core/src/app/add.rs crates/aim-core/tests/adapter_contract.rs crates/aim-core/tests/install_integration.rs crates/aim-cli/tests/end_to_end_cli.rs
git commit -m "feat: add sourceforge install source resolution"
```

### Task 6: Keep registry origin truthful and update metadata additive

**Files:**
- Modify: `crates/aim-core/src/registry/model.rs`
- Modify: `crates/aim-core/src/app/add.rs`
- Modify: `crates/aim-core/src/app/update.rs`
- Modify: `crates/aim-core/src/update/channels.rs`
- Modify: `crates/aim-core/tests/update_planning.rs`
- Modify: `crates/aim-core/tests/registry_roundtrip.rs`

**Step 1: Write the failing tests**

Add tests that assert:
- GitLab and SourceForge installs preserve original source kind and locator after roundtrip persistence
- direct URL installs remain direct URL installs after metadata inspection
- discovered update channels augment stored state without rewriting source identity
- zsync remains update metadata only

**Step 2: Run test to verify it fails**

Run: `cargo test --package aim-core --test update_planning`
Expected: FAIL because update planning and registry expectations do not yet fully encode the approved source-versus-update split.

**Step 3: Write minimal implementation**

Adjust registry and update planning logic so install origin remains canonical and update channels remain additive metadata.

**Step 4: Run test to verify it passes**

Run: `cargo test --package aim-core --test update_planning`
Expected: PASS.

**Step 5: Commit**

```bash
git add crates/aim-core/src/registry/model.rs crates/aim-core/src/app/add.rs crates/aim-core/src/app/update.rs crates/aim-core/src/update/channels.rs crates/aim-core/tests/update_planning.rs crates/aim-core/tests/registry_roundtrip.rs
git commit -m "feat: preserve source identity through update planning"
```

### Task 7: Improve provider-aware error reporting without changing CLI shape

**Files:**
- Modify: `crates/aim-core/src/adapters/traits.rs`
- Modify: `crates/aim-core/src/app/add.rs`
- Modify: `crates/aim-cli/src/lib.rs`
- Modify: `crates/aim-core/tests/install_failures.rs`
- Modify: `crates/aim-cli/tests/end_to_end_cli.rs`

**Step 1: Write the failing tests**

Add tests that distinguish:
- unsupported source semantics
- supported source with no installable artifact
- transport or integration failure

**Step 2: Run test to verify it fails**

Run: `cargo test --package aim-core --test install_failures`
Expected: FAIL because failure reasons are not yet structured enough to preserve those distinctions.

**Step 3: Write minimal implementation**

Introduce explicit failure categories and thread them through the add flow so the CLI can render clearer provider-aware messages without changing the progress UI architecture.

**Step 4: Run test to verify it passes**

Run: `cargo test --package aim-core --test install_failures`
Expected: PASS.

**Step 5: Commit**

```bash
git add crates/aim-core/src/adapters/traits.rs crates/aim-core/src/app/add.rs crates/aim-cli/src/lib.rs crates/aim-core/tests/install_failures.rs crates/aim-cli/tests/end_to_end_cli.rs
git commit -m "feat: clarify provider-aware source resolution failures"
```

### Task 8: Full verification

**Files:**
- Modify: `README.md`
- Modify: `crates/aim-core/tests/github_source_discovery.rs`
- Modify: `crates/aim-core/tests/query_resolution.rs`
- Modify: `crates/aim-core/tests/install_integration.rs`
- Modify: `crates/aim-core/tests/update_planning.rs`
- Modify: `crates/aim-cli/tests/end_to_end_cli.rs`

**Step 1: Tighten any stale expectations**

Update docs and tests so the product contract matches the approved design:
- GitLab and SourceForge are install sources
- direct URL remains exact-resolution
- zsync remains update metadata

**Step 2: Run focused workspace verification**

Run: `cargo test --package aim-core --test query_resolution --test adapter_contract --test install_integration --test update_planning --test install_failures`
Expected: PASS.

**Step 3: Run CLI verification**

Run: `cargo test --package aim-cli --test end_to_end_cli`
Expected: PASS.

**Step 4: Run full workspace verification**

Run: `cargo fmt --all && cargo test --workspace`
Expected: PASS.

**Step 5: Commit**

```bash
git add README.md crates/aim-core/tests/github_source_discovery.rs crates/aim-core/tests/query_resolution.rs crates/aim-core/tests/install_integration.rs crates/aim-core/tests/update_planning.rs crates/aim-core/tests/install_failures.rs crates/aim-cli/tests/end_to_end_cli.rs
git commit -m "docs: align source provider contract and tests"
```