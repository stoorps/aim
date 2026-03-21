# Security Hardening Config And Integrity Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add secure-by-default HTTP policy controls, enforce AppImageHub HTTPS and checksum handling, sanitize desktop entries, harden stable-ID path usage, and document the remaining AppImageHub trust issue.

**Architecture:** Extend the existing runtime `CliConfig` with `allow_http`, thread that config into dispatch and add/install planning, keep provider-returned AppImageHub URLs on a stricter HTTPS-only path, add a provider-specific MD5 integrity check distinct from the existing trusted checksum mechanism, and tighten install-time path and desktop-entry generation at the boundary where files are written.

**Tech Stack:** Rust workspace, Cargo tests, TOML config loading, existing install pipeline, fixture-backed provider tests.

---

### Task 1: Record the approved security shape in repo docs

**Files:**
- Create: `.architecture/security-issues.md`
- Modify: `README.md`
- Reference: `.audits/2026-03-21T20-08-04Z-post-appimagehub-security-audit.md`

**Step 1: Write the security issues note**

Create `.architecture/security-issues.md` with:

- a short description of the AppImageHub host-trust gap
- current mitigation: AppImageHub downloads must be HTTPS
- deferred work: domain allowlist / provider trust policy
- status label such as `open`

**Step 2: Update the README security/config section**

Document:

- `allow_http = false` default
- `allow_http = true` only affects user-supplied HTTP sources
- provider-returned AppImageHub URLs remain HTTPS-only

**Step 3: Verify docs exist and read clearly**

Run: `rg -n "allow_http|AppImageHub|security" README.md .architecture/security-issues.md`
Expected: matching lines in both files

**Step 4: Commit**

```bash
git add .architecture/security-issues.md README.md
git commit -m "docs: record download security policy"
```

### Task 2: Add `allow_http` to runtime config and thread it into dispatch

**Files:**
- Modify: `crates/aim-cli/src/config.rs`
- Modify: `crates/aim-cli/src/main.rs`
- Modify: `crates/aim-cli/src/lib.rs`
- Test: `crates/aim-cli/tests/config_loading.rs`

**Step 1: Write the failing config tests**

Add tests covering:

- default config has `allow_http == false`
- config file with `allow_http = true` parses and loads correctly

**Step 2: Run the focused tests to verify failure**

Run: `cargo test --package aim-cli --test config_loading`
Expected: FAIL because `allow_http` does not exist yet

**Step 3: Add the config field**

Update `CliConfig` with:

- `allow_http: bool`
- `#[serde(default)]`
- default value `false`

**Step 4: Thread config into dispatch**

Refactor the dispatch entrypoints so the already-loaded runtime config is available during query resolution and install planning.

Preferred shape:

- add `dispatch_with_reporter_and_config(...)`
- keep existing `dispatch_with_reporter(...)` delegating to default config if needed for compatibility
- update `main.rs` to call the config-aware path

**Step 5: Run the focused tests to verify pass**

Run: `cargo test --package aim-cli --test config_loading`
Expected: PASS

**Step 6: Commit**

```bash
git add crates/aim-cli/src/config.rs crates/aim-cli/src/main.rs crates/aim-cli/src/lib.rs crates/aim-cli/tests/config_loading.rs
git commit -m "feat: add allow_http runtime config"
```

### Task 3: Enforce HTTP policy for user-supplied sources only

**Files:**
- Modify: `crates/aim-core/src/source/input.rs`
- Modify: `crates/aim-core/src/app/add.rs`
- Modify: `crates/aim-cli/src/lib.rs`
- Test: `crates/aim-core/tests/query_resolution.rs`
- Test: `crates/aim-cli/tests/end_to_end_cli.rs`

**Step 1: Write the failing behavior tests**

Add tests covering:

- direct `http://example.com/app.AppImage` fails by default
- the same input succeeds when `allow_http = true`
- explicit SourceForge `http://...` inputs follow the same rule

**Step 2: Run the focused tests to verify failure**

Run: `cargo test --package aim-cli --test end_to_end_cli`
Expected: FAIL because HTTP is currently accepted unconditionally

**Step 3: Add an explicit HTTP policy check**

Implement a narrow policy helper that is evaluated only for user-supplied source inputs before add/install proceeds.

Requirements:

- reject insecure HTTP when config disallows it
- preserve HTTPS behavior unchanged
- do not let this config affect provider-returned URLs

**Step 4: Surface a clear security error**

Ensure the user sees a message equivalent to:

- `insecure HTTP sources are disabled; set allow_http = true to permit them`

**Step 5: Run the focused tests to verify pass**

Run: `cargo test --package aim-cli --test end_to_end_cli`
Expected: PASS with both rejection and opt-in cases covered

**Step 6: Commit**

```bash
git add crates/aim-core/src/source/input.rs crates/aim-core/src/app/add.rs crates/aim-cli/src/lib.rs crates/aim-core/tests/query_resolution.rs crates/aim-cli/tests/end_to_end_cli.rs
git commit -m "feat: enforce user http policy"
```

### Task 4: Enforce HTTPS for AppImageHub provider-returned downloads

**Files:**
- Modify: `crates/aim-core/src/source/appimagehub.rs`
- Modify: `crates/aim-core/src/adapters/appimagehub.rs`
- Modify: `crates/aim-core/src/app/add.rs`
- Test: `crates/aim-core/tests/adapter_contract.rs`
- Test: `crates/aim-cli/tests/end_to_end_cli.rs`

**Step 1: Write the failing AppImageHub tests**

Add a fixture-backed case where AppImageHub returns an `http://` download URL.

Expected result:

- install planning or resolution fails with a provider-specific security error
- this remains true even when `allow_http = true`

**Step 2: Run the focused tests to verify failure**

Run: `cargo test --package aim-core --test adapter_contract`
Expected: FAIL because AppImageHub URLs are currently accepted verbatim

**Step 3: Add AppImageHub URL validation**

Validate provider-returned AppImageHub download URLs for:

- HTTPS scheme required
- clear provider-specific error path

Do not add the broader host allowlist in this task.

**Step 4: Run the focused tests to verify pass**

Run: `cargo test --package aim-core --test adapter_contract && cargo test --package aim-cli --test end_to_end_cli`
Expected: PASS

**Step 5: Commit**

```bash
git add crates/aim-core/src/source/appimagehub.rs crates/aim-core/src/adapters/appimagehub.rs crates/aim-core/src/app/add.rs crates/aim-core/tests/adapter_contract.rs crates/aim-cli/tests/end_to_end_cli.rs
git commit -m "fix: require https for appimagehub downloads"
```

### Task 5: Sanitize desktop entry display names

**Files:**
- Modify: `crates/aim-core/src/app/add.rs`
- Test: `crates/aim-core/tests/install_integration.rs`
- Test: `crates/aim-cli/tests/end_to_end_cli.rs`

**Step 1: Write the failing desktop-entry tests**

Add tests covering:

- display name containing `\nExec=evil` does not inject a second field
- display name containing control characters renders safely
- normal display names still render as expected

**Step 2: Run the focused tests to verify failure**

Run: `cargo test --package aim-core --test install_integration`
Expected: FAIL because desktop entry output currently interpolates raw display names

**Step 3: Implement minimal sanitation**

Add a helper near desktop entry rendering that:

- strips `\r` and `\n`
- replaces other control characters with spaces or removes them
- preserves ordinary printable text

Use the sanitized value only for desktop-entry rendering, not for mutating the stored app record.

**Step 4: Run the focused tests to verify pass**

Run: `cargo test --package aim-core --test install_integration`
Expected: PASS

**Step 5: Commit**

```bash
git add crates/aim-core/src/app/add.rs crates/aim-core/tests/install_integration.rs crates/aim-cli/tests/end_to_end_cli.rs
git commit -m "fix: sanitize desktop entry names"
```

### Task 6: Enforce AppImageHub MD5 integrity checks

**Files:**
- Modify: `Cargo.toml`
- Modify: `crates/aim-core/Cargo.toml`
- Modify: `crates/aim-core/src/domain/artifact.rs` or the existing artifact type definition file
- Modify: `crates/aim-core/src/app/add.rs`
- Modify: `crates/aim-core/src/integration/install.rs`
- Test: `crates/aim-core/tests/checksum_verification.rs`
- Test: `crates/aim-cli/tests/end_to_end_cli.rs`

**Step 1: Identify the artifact checksum type location**

Before editing, confirm where `ArtifactCandidate` is defined and where a provider-specific MD5 field should live.

**Step 2: Write the failing integrity tests**

Add tests covering:

- AppImageHub install succeeds with matching MD5 fixture data
- AppImageHub install fails before commit on MD5 mismatch
- AppImageHub install still succeeds when no MD5 exists

**Step 3: Run the focused tests to verify failure**

Run: `cargo test --package aim-core --test checksum_verification`
Expected: FAIL because AppImageHub MD5 is currently ignored

**Step 4: Add a separate weak-integrity field/path**

Implement a provider-specific integrity path distinct from `trusted_checksum`.

Requirements:

- store the provider MD5 on the artifact candidate or equivalent install request
- verify it after staging and before commit
- do not overload the existing trusted SHA-512 semantics

**Step 5: Add any needed dependency explicitly**

If an MD5 crate is required, add it at the workspace and crate level.

**Step 6: Run the focused tests to verify pass**

Run: `cargo test --package aim-core --test checksum_verification && cargo test --package aim-cli --test end_to_end_cli`
Expected: PASS

**Step 7: Commit**

```bash
git add Cargo.toml crates/aim-core/Cargo.toml crates/aim-core/src/app/add.rs crates/aim-core/src/integration/install.rs crates/aim-core/tests/checksum_verification.rs crates/aim-cli/tests/end_to_end_cli.rs
git commit -m "feat: verify appimagehub md5 integrity"
```

### Task 7: Harden stable IDs and managed path containment

**Files:**
- Modify: `crates/aim-core/src/app/identity.rs`
- Modify: `crates/aim-core/src/app/add.rs`
- Test: `crates/aim-core/tests/identity_resolution.rs`
- Test: `crates/aim-core/tests/install_paths.rs`

**Step 1: Write the failing hardening tests**

Add tests covering:

- identifiers normalizing to `..` are rejected
- managed install paths do not escape managed roots

**Step 2: Run the focused tests to verify failure**

Run: `cargo test --package aim-core --test identity_resolution --test install_paths`
Expected: FAIL because `..` currently survives normalization and there is no explicit containment check

**Step 3: Implement identity and path validation**

Add:

- explicit normalized-ID rejection for `..`
- path containment validation before install proceeds

Keep the implementation minimal and deterministic.

**Step 4: Run the focused tests to verify pass**

Run: `cargo test --package aim-core --test identity_resolution --test install_paths`
Expected: PASS

**Step 5: Commit**

```bash
git add crates/aim-core/src/app/identity.rs crates/aim-core/src/app/add.rs crates/aim-core/tests/identity_resolution.rs crates/aim-core/tests/install_paths.rs
git commit -m "fix: harden stable id paths"
```

### Task 8: Add external helper audit logging and adversarial regression coverage

**Files:**
- Modify: `crates/aim-core/src/integration/refresh.rs`
- Modify: `crates/aim-core/src/source/appimagehub.rs`
- Test: `crates/aim-core/tests/adapter_contract.rs`
- Test: `crates/aim-core/tests/install_integration.rs`
- Test: `crates/aim-cli/tests/end_to_end_cli.rs`

**Step 1: Write the failing or missing regression tests**

Add adversarial cases for:

- malformed AppImageHub XML or missing fields handled cleanly
- malicious display names in fixture-backed install flows
- helper execution paths producing expected warnings/loggable branches

**Step 2: Implement minimal logging**

Add debug-level logging around helper execution in `refresh.rs`.

**Step 3: Run focused tests**

Run: `cargo test --package aim-core --test adapter_contract --test install_integration && cargo test --package aim-cli --test end_to_end_cli`
Expected: PASS

**Step 4: Commit**

```bash
git add crates/aim-core/src/integration/refresh.rs crates/aim-core/src/source/appimagehub.rs crates/aim-core/tests/adapter_contract.rs crates/aim-core/tests/install_integration.rs crates/aim-cli/tests/end_to_end_cli.rs
git commit -m "test: cover security edge cases"
```

### Task 9: Full verification and final docs pass

**Files:**
- Modify: `.plans/012-security-hardening-config-and-integrity/2026-03-21-security-hardening-config-and-integrity-design.md` if implementation drifted
- Modify: `.plans/012-security-hardening-config-and-integrity/2026-03-21-security-hardening-config-and-integrity-implementation-plan.md` if task wording drifted
- Modify: `.architecture/security-issues.md` if final wording needs adjustment

**Step 1: Run formatting and full verification**

Run:

```bash
cargo fmt --all
cargo test --workspace
cargo clippy --workspace --all-targets --all-features -- -D warnings
```

Expected: all commands succeed.

**Step 2: Re-read the security docs**

Confirm the final README and `.architecture/security-issues.md` text still matches the implementation.

**Step 3: Commit**

```bash
git add .plans/012-security-hardening-config-and-integrity .architecture/security-issues.md README.md
git commit -m "docs: record security hardening plan"
```