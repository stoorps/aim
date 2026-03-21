# Repository Audit: aim Post-AppImageHub

**Timestamp:** 2026-03-21T20:08:04Z
**Audited commit:** `f8ffb953763ceab41bb97a26afae52df7e31d539`
**Branch at audit time:** `main`
**Scope:** repository-level functionality gaps, engineering holes, and security concerns after the AppImageHub provider merge

## Executive Summary

The codebase is in materially better shape than the first generated audit suggested. Two previously reported items were false positives and have been removed:

- registry writes are already atomic via temp-file-plus-rename in [crates/aim-core/src/registry/store.rs](crates/aim-core/src/registry/store.rs)
- CLI search coverage already exists in [crates/aim-cli/tests/search_cli.rs](crates/aim-cli/tests/search_cli.rs)

That said, there are still real pre-release security gaps.

### Verified top findings

1. Critical: insecure `http://` downloads are accepted by the direct URL path and some provider parsing paths
2. Critical: AppImageHub download URLs from the upstream API are trusted without a transport or trust-policy check
3. High: desktop entry generation accepts unsanitized display names, allowing newline-based desktop-file field injection
4. Medium: AppImageHub exposes MD5 metadata, but installs do not validate any AppImageHub checksum at all
5. Low: stable ID normalization permits `..`, so explicit path hardening is still missing as defense in depth

### Overall verdict

Not production-ready for hostile or semi-hostile networks. The dominant work left is security hardening, not core feature breadth.

## Detailed Findings

### SEC-001: Insecure HTTP Downloads Accepted

**Severity:** Critical
**Category:** Security
**Why it matters:** Any accepted `http://` download enables trivial man-in-the-middle payload replacement.

**Evidence**

- Direct URL classification accepts both HTTP and HTTPS in [crates/aim-core/src/source/input.rs](crates/aim-core/src/source/input.rs#L57)
- SourceForge parsing still explicitly accepts HTTP project URLs in [crates/aim-core/src/adapters/sourceforge.rs](crates/aim-core/src/adapters/sourceforge.rs#L219)

**Impact**

- A user can install a payload fetched over plaintext transport.
- An attacker on the network path can replace the AppImage before it is executed.

**Recommendation**

- Reject `http://` sources by default.
- If you want an escape hatch, require an explicit insecure override such as `--allow-insecure` and print a strong warning.
- Add regression tests for both rejection and override behavior.

### SEC-002: AppImageHub Download URLs Are Trusted Without Validation

**Severity:** Critical
**Category:** Security
**Why it matters:** The AppImageHub API is currently treated as authoritative for the final download URL, but the returned URL is not checked before being handed to the downloader.

**Evidence**

- AppImageHub download links are accepted verbatim in [crates/aim-core/src/source/appimagehub.rs](crates/aim-core/src/source/appimagehub.rs#L349)
- The selected AppImageHub download URL is passed directly into the install artifact candidate in [crates/aim-core/src/app/add.rs](crates/aim-core/src/app/add.rs#L181)

**Impact**

- If the upstream API is compromised or returns an unexpected domain, `aim` will fetch from that location.
- Combined with SEC-001, this becomes a clear supply-chain hole.

**Recommendation**

- Enforce HTTPS on provider-returned URLs.
- Add a host trust policy for AppImageHub downloads, either hard-coded to known domains or configurable.
- Add tests for invalid schemes and untrusted hosts.

### SEC-003: Desktop Entry Field Injection Through Unsanitized Display Name

**Severity:** High
**Category:** Security
**Why it matters:** The generated `.desktop` file interpolates `display_name` directly into the file body. The practical risk is newline injection, not shell metacharacters in the `Name` field.

**Evidence**

- Desktop entries are rendered by string interpolation in [crates/aim-core/src/app/add.rs](crates/aim-core/src/app/add.rs#L687)
- `display_name` can come from provider-controlled metadata through identity resolution in [crates/aim-core/src/app/identity.rs](crates/aim-core/src/app/identity.rs#L27)

**Impact**

- A malicious provider name containing `\nExec=...` or another desktop-entry key can inject extra fields into the generated launcher.

**Recommendation**

- Strip or reject newlines and control characters before writing desktop entries.
- Consider validating display names against a conservative allowlist for launcher generation.
- Add a regression test proving that a name containing `\nExec=evil` cannot alter the resulting desktop file.

### SEC-004: AppImageHub Checksum Metadata Is Parsed But Not Used

**Severity:** Medium
**Category:** Security / Integrity
**Why it matters:** AppImageHub metadata includes MD5 values, but the current flow intentionally drops them and performs no integrity check for AppImageHub downloads.

**Evidence**

- MD5 fields are stored on AppImageHub download records in [crates/aim-core/src/source/appimagehub.rs](crates/aim-core/src/source/appimagehub.rs#L14)
- AppImageHub installs set `trusted_checksum: None` in [crates/aim-core/src/app/add.rs](crates/aim-core/src/app/add.rs#L184)
- The checksum verifier only handles the existing trusted checksum path in [crates/aim-core/src/integration/install.rs](crates/aim-core/src/integration/install.rs#L169)

**Impact**

- AppImageHub downloads have no post-download integrity signal at all.
- This does not create remote code execution by itself, but it weakens the install pipeline substantially.

**Recommendation**

- Add explicit support for provider MD5 verification as a separate integrity path, or refuse to advertise checksum-backed trust when only MD5 is available.
- If you keep MD5 support, label it as weak integrity rather than strong trust.

### GAP-001: Stable ID Path Hardening Is Missing

**Severity:** Low
**Category:** Security / Hardening
**Why it matters:** Stable IDs are used in path construction. The normalizer preserves `.` characters, so `..` survives normalization.

**Evidence**

- Identifier normalization preserves `.` in [crates/aim-core/src/app/identity.rs](crates/aim-core/src/app/identity.rs#L62)
- Stable IDs are interpolated into installation paths in [crates/aim-core/src/app/add.rs](crates/aim-core/src/app/add.rs#L424)

**Impact**

- I did not verify a full exploit path from current command flows, so this is not an immediate blocker.
- It is still worth closing because the path join contract currently relies on upstream callers never producing `..`.

**Recommendation**

- Explicitly reject `..` in normalized stable IDs.
- Add a path containment assertion before final install paths are committed.

### OBS-001: No Audit Trail For External Refresh Commands

**Severity:** Low
**Category:** Observability
**Why it matters:** Desktop integration helpers are executed, but successful invocations are not logged anywhere useful for incident reconstruction.

**Evidence**

- Helper execution happens in [crates/aim-core/src/integration/refresh.rs](crates/aim-core/src/integration/refresh.rs#L17)

**Recommendation**

- Add debug logging for helper name, args, and exit status.

## False Positives Removed

These were checked and should not be treated as open findings:

- Registry writes are atomic: [crates/aim-core/src/registry/store.rs](crates/aim-core/src/registry/store.rs#L24)
- CLI search tests do exist, including AppImageHub coverage: [crates/aim-cli/tests/search_cli.rs](crates/aim-cli/tests/search_cli.rs#L1)

## Positive Findings

- Checksum verification for the existing trusted-checksum path is implemented and tested in [crates/aim-core/src/integration/install.rs](crates/aim-core/src/integration/install.rs#L169) and [crates/aim-core/tests/checksum_verification.rs](crates/aim-core/tests/checksum_verification.rs)
- Update rollback exists and is exercised in [crates/aim-core/src/app/update.rs](crates/aim-core/src/app/update.rs) and related tests
- Registry persistence is already atomic in [crates/aim-core/src/registry/store.rs](crates/aim-core/src/registry/store.rs)
- Search has both core-level and CLI-level coverage in [crates/aim-core/tests/appimagehub_search.rs](crates/aim-core/tests/appimagehub_search.rs) and [crates/aim-cli/tests/search_cli.rs](crates/aim-cli/tests/search_cli.rs)

## Missing Functionality / Product Holes

The most meaningful missing functionality from this pass is security functionality rather than user-facing commands:

- no secure/insecure transport policy split for downloads
- no provider trust policy for AppImageHub download hosts
- no provider-specific integrity verification for AppImageHub artifacts
- no adversarial-input test suite covering malicious provider metadata and malformed provider responses

I did not verify any major missing core CLI flow beyond those hardening gaps in this pass.

## Recommended Priority Order

### Immediate

1. Reject insecure HTTP downloads by default
2. Validate AppImageHub download URLs before download
3. Sanitize display names before desktop-entry generation

### Short-Term

4. Add AppImageHub integrity verification semantics
5. Add targeted security regression tests for malicious URLs and malicious display names

### Medium-Term

6. Harden stable ID path safety
7. Add structured logging for external helper execution

## Residual Test Gaps

- No regression test proving HTTP sources are rejected
- No regression test proving AppImageHub URLs are scheme/host validated
- No regression test proving newline-bearing display names cannot inject desktop-entry fields
- No adversarial fixture coverage for malformed or malicious AppImageHub XML payloads

## Audit Conclusion

This is a solid implementation base with real progress on provider breadth, rollback, and verification, but it still has several concrete security holes in the download trust boundary. Those should be treated as the next tranche before calling the installer production-safe.
