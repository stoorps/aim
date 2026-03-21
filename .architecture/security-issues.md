# Security Issues

## AppImageHub Download Host Trust

**Status:** Open
**Severity:** High
**Area:** Provider trust / supply chain

### Summary

`aim` now enforces HTTPS for AppImageHub provider-returned download URLs, but it does not yet enforce a host trust policy or allowlist for those returned URLs.

### Current Mitigation

- AppImageHub download URLs must use `https://`
- insecure user-supplied HTTP policy is handled separately through `allow_http`

### Remaining Gap

A compromised or unexpected AppImageHub API response could still direct downloads to an arbitrary HTTPS host. Transport encryption alone does not establish publisher trust.

### Deferred Follow-Up

Future hardening should add one of:

- a fixed allowlist of expected AppImageHub download hosts
- a configurable host trust policy
- stronger publisher verification metadata if AppImageHub exposes it

### Notes

This issue is intentionally tracked separately from the immediate HTTPS enforcement work so the current hardening tranche can reduce risk without trying to solve the full provider trust model in one pass.