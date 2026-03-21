# Source And Provider Expansion Design

## Goal

Expand install-source coverage beyond the current GitHub-centric path without collapsing providers, exact artifact sources, and update metadata into one abstraction.

## Problem Statement

The current codebase has a real mismatch between what the domain and adapter layers suggest is possible and what the end-to-end install pipeline actually treats as first-class:

- the domain source model already includes GitHub, GitLab, direct URL, and file
- the adapter layer also advertises SourceForge and zsync shapes
- the public source pipeline and most end-to-end behavior are still effectively GitHub-shaped

That creates two problems:

- adding new install origins risks becoming a copy of the GitHub path with provider-specific exceptions bolted on later
- `zsync` is at risk of being modeled as a provider even though the current code treats it primarily as update metadata

## Design Goals

- make `GitLab` a real repository-backed install source
- make `SourceForge` a real repository-backed install source
- preserve `direct-url` as a first-class exact-resolution source
- keep install origin semantics truthful in the registry
- allow provider-native update re-resolution where it fits naturally
- keep `zsync` as update metadata rather than forcing it into the install-source model

## Non-Goals

- generic search UX across all providers
- full behavioral parity across every provider in the first slice
- rewriting the CLI presentation layer
- promoting `zsync` into a first-class install source
- inventing a universal provider abstraction that erases real capability differences

## Architectural Decision

Separate three concerns explicitly:

- `source kind`: how the user identified the install origin
- `resolution strategy`: how the system turns that origin into a concrete installable artifact
- `update channel`: how an installed application later discovers newer payloads

Under this model:

- `GitHub`, `GitLab`, and `SourceForge` are repository-backed source kinds
- `direct-url` is an exact artifact source kind
- `file` remains a local artifact source kind
- `zsync` remains an update-channel and metadata mechanism

This keeps install and update semantics aligned without pretending every source type has provider-like behavior.

## Source Taxonomy

The input classification layer should classify user queries into a small, stable taxonomy:

- repository-backed sources
  - GitHub
  - GitLab
  - SourceForge
- exact artifact sources
  - direct URL
- local artifact sources
  - file

Classification should answer only:

- what kind of source the user provided
- what canonical locator can be derived
- whether release or asset hints are present
- whether the origin is inherently trackable

Classification should not try to encode provider-specific release discovery beyond those normalized hints.

## Resolution Model

After classification, a resolver layer should convert a `SourceRef` into an installable release candidate.

### Repository-backed sources

Repository-backed resolution should:

- accept a canonical repository or project locator
- discover release or download candidates using provider-specific logic
- select a concrete AppImage payload when confidence is sufficient
- return explicit structured failures when the repository exists but no installable AppImage is available

This applies to:

- GitHub
- GitLab
- SourceForge

### Exact artifact sources

Exact-resolution sources should:

- treat the user-provided locator as the concrete payload origin
- derive best-effort metadata without pretending release discovery exists
- remain installable even when rich update tracking is unavailable

This applies to:

- direct URL
- file

## Registry Semantics

Registry persistence should preserve the truth about where the install came from.

Each installed record should continue to store:

- original source kind
- original source locator
- canonical locator when one exists
- installed version
- installed file metadata

The key rule is:

- install origin remains the origin the user chose
- update mechanisms are additive metadata, not a replacement source identity

That means a direct URL install remains a direct URL install even if metadata later yields a richer update channel. A GitLab install remains a GitLab install even if update planning later uses provider-specific release rediscovery.

## Update Strategy

Update planning should use the install origin as the primary re-entry point.

For repository-backed sources:

- prefer provider-native re-resolution using the canonical locator
- attach update channels discovered during metadata inspection as additional evidence

For exact-resolution sources:

- keep update support weak by default unless post-install metadata offers something stronger

For `zsync`:

- keep it as discovered metadata and update-channel input
- do not rewrite the install source as `zsync`
- do not require `zsync` install-source tests in this phase

This preserves a clean distinction between install origin and update mechanism.

## Error Handling

The design should distinguish unsupported semantics from runtime failure.

### Unsupported source semantics

Examples:

- malformed provider URL shapes
- a project URL form we do not support yet
- a provider source kind sent to the wrong resolver

These should fail early during classification or resolver selection with provider-aware messages.

### Resolvable source, but no installable artifact

Examples:

- repository exists but has no AppImage asset
- release metadata is present but incomplete
- multiple assets exist but none match install heuristics confidently

These should be structured resolution failures rather than generic unsupported errors.

### Transport or integration failure

Examples:

- HTTP download failures
- metadata fetch failures
- local staging or desktop integration failures

These remain operational failures in the existing install and update flow.

## Testing Strategy

Testing should expand along capability lines rather than provider-specific copy-paste.

### Classification tests

Add coverage for:

- GitLab source forms
- SourceForge source forms
- direct URL edge cases
- unsupported or malformed provider inputs

### Resolver contract tests

Each resolver should satisfy a shared contract:

- accepts valid source refs for its own kind
- rejects source refs for other kinds
- returns a concrete install candidate or a structured no-artifact result

### End-to-end flow tests

Add focused flow coverage for:

- install from GitLab source
- install from direct URL
- install from SourceForge source
- truthful registry origin persistence
- update planning that uses install origin plus additive metadata

### Non-goal tests

Do not force `zsync` into install-source tests for this phase.

## Rollout

Recommended rollout order:

1. normalize the source taxonomy and resolver interfaces
2. wire GitLab and direct URL cleanly through install and registry persistence
3. add SourceForge using the same resolver contract, limited to supported URL and project forms
4. extend update planning only where the source kind supports provider-native re-resolution naturally
5. leave `zsync` unchanged except to ensure it remains additive update metadata

This keeps the product honest: each added source gets explicit semantics instead of being forced through a renamed GitHub pathway.