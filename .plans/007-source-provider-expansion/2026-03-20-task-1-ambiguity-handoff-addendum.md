# Task 1 Ambiguity Handoff Addendum

## Goal

Resolve the Task 1 blocker by moving ambiguous GitLab and SourceForge URL handling out of pure taxonomy heuristics and into provider-aware resolution.

## Problem Restatement

The blocker is not that the classifier is missing a few more path rules.

The blocker is that some provider-hosted URL shapes do not carry enough information to determine final install semantics from path shape alone.

Two cases are responsible for the review churn:

- GitLab deep paths where a segment may be either a subgroup slug or a resource-like segment
- SourceForge `files/.../download` paths where the same suffix can represent either a concrete file download or a folder-style endpoint

Trying to settle those cases in `resolve_query(...)` forces the code into a false choice:

- accept ambiguous inputs too early and misclassify them
- reject provider-owned inputs too early and lose useful context

## Design Decision

Adopt an ambiguity handoff model.

That means:

- the classifier remains authoritative only for cases it can determine with high confidence
- ambiguous provider-hosted inputs are preserved as provider-owned candidates rather than flattened into `Unsupported`
- provider adapters become the layer that decides whether an ambiguous input is:
  - a supported repository or project source
  - a supported exact download form
  - a supported source with no installable artifact
  - truly unsupported for that provider

## Contract Boundary

### Classification policy

The classifier should use a strict positive-matching contract.

Each input shape must land in exactly one of three buckets:

- accept as a definite supported source
- accept as an explicit provider-owned candidate
- reject as unsupported

This means the classifier should prefer a small allowlist of accepted shapes over an expanding catalog of bespoke rejection rules.

Negative rules are still allowed when needed to protect a known false-positive family, but they are defensive exceptions, not the main design strategy.

### Classification must do

- identify definite GitHub, GitLab, SourceForge, direct URL, and file inputs
- accept only explicitly enumerated concrete shapes or explicitly enumerated candidate shapes
- preserve canonical locator hints when they are certain
- preserve enough raw path context for later provider-specific disambiguation
- continue classifying concrete artifact URLs as `DirectUrl` when the classifier can say so confidently

### Classification must not do

- grow by accumulating one-off rejection rules for every unsupported provider page family
- guess whether a GitLab deep path is a subgroup path or a resource page when the path shape is ambiguous
- guess whether a SourceForge nested `files/.../download` path is a file or folder endpoint when the path shape is ambiguous
- perform provider-specific network discovery

### Resolver layer must do

- own final interpretation of ambiguous provider-hosted inputs
- return structured outcomes through the adapter contract
- keep `UnsupportedSource` reserved for sources the adapter genuinely does not own
- use `NoInstallableArtifact` for provider-owned inputs that are valid but not installable under current scope

## Proposed Source Model Adjustment

Introduce an explicit handoff shape for ambiguous provider-owned inputs.

The minimal acceptable form is:

- preserve the original locator
- preserve provider ownership
- preserve any canonical parts that are certain
- add a signal that provider resolution is still required before install semantics are known

This can be modeled either as:

1. a dedicated ambiguity marker on `SourceRef`
2. additional normalized kinds representing provider-owned unresolved candidates

The preferred direction is additional normalized kinds, because they keep the ambiguity visible in tests and logs without adding a free-form boolean that can drift.

Illustrative shapes:

- `NormalizedSourceKind::GitLabCandidate`
- `NormalizedSourceKind::SourceForgeCandidate`

The exact enum names are secondary. The important part is making unresolved provider ownership explicit.

## Provider Responsibilities

### GitLab

GitLab adapter logic should decide whether a GitLab-owned ambiguous input is:

- a valid repository locator
- a release-like source with concrete version semantics
- a provider-owned but non-installable resource page
- unsupported because it does not fit the adapter's supported contract

Initial scope should stay narrow:

- keep current definite repository and release-like support
- add only one or two ambiguous deep-path cases as a first expansion slice
- do not try to solve every GitLab resource URL family at once

### SourceForge

SourceForge adapter logic should decide whether a SourceForge-owned ambiguous input is:

- a concrete latest-download install source
- a concrete direct artifact URL
- a provider-owned project or folder view with no installable artifact
- unsupported for current source scope

Initial scope should stay narrow:

- keep bare project URLs as provider-owned and non-installable
- keep `files/latest/download` as the first concrete repository-backed install source
- add exactly one nested `files/.../download` ambiguity case to the adapter decision path

## Testing Strategy

The blocker should be resolved by shifting assertions to the right layer.

### Classification tests

Update `query_resolution` coverage so ambiguous cases assert provider ownership and handoff state instead of asserting final install semantics.

Coverage should be organized around accepted-shape allowlists:

- accepted concrete shapes
- accepted candidate shapes
- a small number of representative false-positive guards

Examples:

- a concrete SourceForge artifact download still classifies as `DirectUrl`
- a definite GitLab repository form still classifies as `GitLab`
- an ambiguous GitLab deep path becomes a GitLab-owned candidate, not `Unsupported`
- an ambiguous SourceForge nested download path becomes a SourceForge-owned candidate, not prematurely direct or unsupported

### Adapter contract tests

Add tests that assert adapters make the final decision for ambiguous handoff inputs.

Examples:

- GitLab candidate path resolves to supported repository semantics
- GitLab candidate path resolves to `NoInstallableArtifact`
- SourceForge candidate path resolves to `Resolved`
- SourceForge candidate path resolves to `NoInstallableArtifact`

### Install and failure tests

Keep install-flow tests focused on supported concrete outcomes.

Keep failure tests focused on the distinction between:

- unsupported query
- provider-owned source with no installable artifact
- runtime install or transport failure

## Incremental Execution Plan

### Phase 1: Lock the boundary

- update the design docs to state that classification only decides what it can know with certainty
- record that ambiguous provider-hosted inputs are a resolver concern

### Phase 2: Add handoff representation

- extend the source model with explicit provider-candidate semantics
- thread that representation through the query classifier

### Phase 3: Shift one GitLab ambiguity case

- add a failing classification test for an ambiguous GitLab deep path
- classify it as a GitLab-owned candidate
- add adapter contract coverage for the GitLab decision

### Phase 4: Shift one SourceForge ambiguity case

- add a failing classification test for a nested `files/.../download` ambiguity case
- classify it as a SourceForge-owned candidate
- add adapter contract coverage for the SourceForge decision

### Phase 5: Tighten error reporting

- make sure ambiguous provider-owned inputs that do not yield installable artifacts surface as `NoInstallableArtifact`
- avoid regressing them into unsupported-query failures

## Progress Update

Current implementation status in this branch:

- Phase 1 is complete. The classifier-versus-adapter boundary is now documented explicitly in this addendum.
- Phase 2 is complete. `GitLabCandidate` and `SourceForgeCandidate` now exist in the source model and are produced by classification for the narrow ambiguity cases under test.
- Phase 3 is complete for the first GitLab slice. `https://gitlab.com/<group>/<subgroup>/releases/<repo>` remains a classified candidate, but the GitLab adapter now resolves it as repository semantics with a derived canonical locator.
- Phase 4 is complete for the current SourceForge slices. `https://sourceforge.net/projects/<project>/files/releases/stable/download` remains a classified candidate and resolves as a provider-owned install source. The broader single-segment family `https://sourceforge.net/projects/<project>/files/releases/<release-folder>/download` is also preserved as a provider-owned candidate and resolves through installation and update. When the `<release-folder>` segment is clearly an artifact filename, provider resolution canonicalizes the stored SourceForge source to `https://sourceforge.net/projects/<project>/files/releases` while preserving the original typed download URL as the first-install artifact.
- Phase 5 is partially complete. Provider-owned ambiguous inputs now distinguish unsupported-query failures from no-artifact outcomes, and both GitLab and SourceForge have adapter-owned positive resolution paths for the accepted provider families.

The current intended classifier contract is:

- accept explicit supported shapes
- accept explicit candidate shapes
- reject everything else

That contract is intentionally stricter than heuristic best-effort classification and intentionally narrower than provider resolution.

What remains intentionally out of scope for this slice:

- additional GitLab candidate families beyond the first repository-style deep path
- broader SourceForge folder families beyond the current single-segment `releases/<release-folder>/download` support contract and the `files/releases` provider root
- any network-backed provider discovery in classification

## Success Criteria

This blocker is considered resolved when:

- `query_resolution` no longer oscillates over ambiguous provider-owned shapes
- ambiguous provider-hosted URLs are no longer forced into final install semantics during classification
- adapters are the only place where ambiguous provider paths are interpreted fully
- failure reporting distinguishes unsupported inputs from provider-owned non-installable inputs

## Non-Goals

- solving every ambiguous GitLab deep-path variant in one pass
- solving every SourceForge nested folder or version path in one pass
- introducing network discovery into the pure query classifier
- expanding current supported source scope beyond what the adapter tests can defend clearly