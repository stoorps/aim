# Security Hardening Config And Integrity Design

## Summary

This change hardens the `aim` install pipeline around transport trust, desktop-entry generation, provider integrity checks, and path safety. The approved shape is narrow by design: add `allow_http` to `config.toml` with a default of `false`, apply it only to user-supplied HTTP sources, keep AppImageHub provider-returned downloads HTTPS-only for now, track the broader AppImageHub host-trust issue in `.architecture/security-issues.md`, sanitize launcher display names before writing `.desktop` files, enforce AppImageHub checksum verification semantics, and harden stable-ID-derived install paths.

## Goals

- Add `allow_http` to the user-facing runtime config with a secure default of `false`.
- Apply `allow_http` only to user-supplied source URLs and explicit HTTP provider inputs.
- Keep AppImageHub provider-returned download URLs on a stricter HTTPS-only policy.
- Record the unresolved AppImageHub host-trust issue in `.architecture/security-issues.md`.
- Prevent newline and control-character injection in generated desktop entries.
- Enforce AppImageHub checksum validation rather than silently dropping provider checksum metadata.
- Explicitly reject or contain dangerous stable IDs such as `..`.
- Add targeted adversarial regression coverage for the new hardening paths.
- Add lightweight audit logging around external helper execution.

## Non-Goals

- No global provider trust framework beyond the minimum AppImageHub note and HTTPS enforcement in this slice.
- No redesign of the theme config loader split unless it becomes necessary to thread runtime config.
- No cryptographic reclassification of AppImageHub MD5 into the existing trusted SHA-512 path.
- No broader provider security audit implementation outside the issues already approved here.
- No new CLI flags unless the implementation later proves config-only is insufficient.

## Approaches

### Option 1: Global `allow_http`

This would make a single config flag disable HTTPS enforcement everywhere, including provider-returned URLs. It is easy to wire, but it weakens the trust boundary too broadly and makes one local preference affect third-party provider behavior in ways that are hard to reason about.

### Option 2: User-Input-Only `allow_http`

This is the approved design. `allow_http` applies only to user-supplied direct URLs and explicit HTTP provider inputs such as legacy SourceForge HTTP forms. Provider-returned download URLs remain subject to provider-specific policy. This keeps the config narrow and predictable while still giving advanced users an escape hatch for manual HTTP sources.

### Option 3: Split Security Flags Per Source Class

This would introduce separate toggles for direct URLs, provider URLs, and possibly per-provider policy. It is the most explicit shape, but it creates unnecessary configuration surface for the current repo.

## Approved Design

### Config Model

Add `allow_http = false` to the existing runtime `config.toml` model used by `crates/aim-cli/src/config.rs`. This config is already loaded in `main.rs` for rendering behavior, but the current dispatch path does not receive it. The implementation should thread the loaded runtime config into dispatch rather than adding an unrelated second config lookup.

The existing theme-only loader under `crates/aim-cli/src/cli/config.rs` is not the place for this setting. This change should preserve that split unless unification becomes necessary later.

### Transport Policy

#### User-Supplied Sources

User-supplied HTTP sources are:

- raw `http://...` direct URLs
- explicit HTTP provider forms that the source parsing layer currently accepts, such as SourceForge HTTP URLs

These should be rejected by default when `allow_http = false` and allowed only when `allow_http = true`.

#### Provider-Returned URLs

Provider-returned URLs are not covered by `allow_http` in this slice. In particular, AppImageHub download URLs returned by the provider transport should be enforced as HTTPS-only regardless of user config.

This distinction preserves the user’s ability to opt into an insecure source they typed deliberately without silently expanding trust for provider-sourced URLs.

### AppImageHub Security Handling

AppImageHub gets two separate treatments:

1. **Immediate enforcement now**
   - reject non-HTTPS AppImageHub download URLs
   - fail resolution or add-plan construction with a provider-specific error

2. **Deferred broader issue**
   - document the missing host-trust / domain allowlist model in `.architecture/security-issues.md`
   - do not try to solve the broader provider trust framework in this slice

This satisfies the approved direction: the HTTPS rule halves the immediate risk, while the architectural gap remains visible and explicit.

### Desktop Entry Sanitization

The `.desktop` renderer should sanitize display names before interpolation. The key requirement is to prevent field injection, so the sanitation policy should at minimum:

- strip or reject `\n` and `\r`
- collapse other control characters
- preserve normal display names for legitimate apps

The sanitation should happen close to desktop-entry generation rather than mutating the stored display name globally.

### AppImageHub Checksum Enforcement

The current AppImageHub path stores MD5 metadata but drops it before installation. This slice should stop silently ignoring it.

Because the existing `trusted_checksum` path is a SHA-512 base64 trust mechanism, AppImageHub MD5 should not be forced into that same contract. Instead, add a provider-specific integrity verification path that:

- computes MD5 for the staged payload when AppImageHub provides one
- fails installation on mismatch
- treats MD5 as weaker integrity metadata, not as equivalent to the existing trusted checksum model

This preserves conceptual clarity: strong trusted checksums remain one mechanism, while provider-specific MD5 integrity checks are a separate, weaker guardrail.

### Stable ID Path Hardening

Stable IDs are interpolated into payload, desktop, and icon paths. The normalizer currently preserves `.` characters, so `..` survives.

The approved change is:

- reject normalized IDs containing `..`
- add a final containment assertion or validation on managed paths before installation proceeds

This is defense in depth even if current command paths do not expose an easy exploit.

### External Command Audit Logging

The desktop integration refresh path should log helper execution at debug level, including command name, args, and exit status when available. This is low-priority observability work, but it belongs in the same hardening tranche because it improves forensic clarity.

## Error Handling

- Rejected HTTP user inputs should fail with a clear security-oriented message explaining that HTTP is disabled unless `allow_http = true` is set.
- Rejected AppImageHub download URLs should fail with a provider-specific security message rather than a generic parse failure.
- Desktop-entry sanitation should be non-disruptive where possible; reject only if the sanitized output would become invalid or empty.
- AppImageHub checksum mismatch should fail install before commit, matching the spirit of existing checksum enforcement.
- Stable-ID hardening should fail deterministically with an explicit invalid-identity error rather than produce a malformed path.

## Testing Strategy

### Config And Transport Tests

Add tests for:

- `allow_http` defaulting to `false`
- config override with `allow_http = true`
- direct `http://` URL rejection by default
- direct `http://` URL success when config enables it
- SourceForge HTTP behavior matching the same policy
- AppImageHub provider-returned `http://` URL rejection even when `allow_http = true`

### Desktop Entry Tests

Add tests for:

- newline-bearing display names being sanitized before render
- control characters not appearing in generated desktop entries
- ordinary display names remaining unchanged

### Integrity Tests

Add tests for:

- valid AppImageHub MD5 succeeds
- invalid AppImageHub MD5 fails before commit
- missing AppImageHub MD5 continues with the current provider behavior

### Path Hardening Tests

Add tests for:

- normalized identifiers containing `..` being rejected
- installation path validation refusing escape outside managed roots

### Observability Tests

If practical, add tests around helper invocation logging or at least unit-test the formatting / branch behavior around helper execution outcomes.

## Delivery Notes

- Do not conflate AppImageHub MD5 with the existing trusted SHA-512 checksum contract.
- Keep `allow_http` policy narrow and explicit.
- Prefer plumbing the already-loaded runtime config into dispatch rather than inventing another config read path.
- Track deferred provider host trust in `.architecture/security-issues.md`, not hidden in TODOs.
- This slice is security-hardening-first; avoid mixing in unrelated product work.