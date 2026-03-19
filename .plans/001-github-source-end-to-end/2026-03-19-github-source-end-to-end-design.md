# GitHub Source End-to-End Design

## Goal

Extend `aim-core` so GitHub-backed installs and updates work end to end across repo shorthand, GitHub URLs, and direct GitHub release asset URLs, while keeping the architecture reusable for future sources and a later GUI client.

The design should support broad discovery, metadata-aware artifact selection, durable update-channel fallback, and a clean separation between source discovery, metadata parsing, and update decision-making.

## Agreed Product Shape

### Supported GitHub entry forms

- `owner/repo`
- GitHub repo URL
- GitHub release URL
- direct GitHub release asset URL

### Canonical identity and source behavior

- Canonical app identity should resolve to the best available app identity, usually the GitHub repository when that can be determined confidently
- The system should preserve the original user input and normalized source references for auditability and later recovery
- If the user points at an older release or a prerelease directly, the system should respect that context rather than silently switching to an unrelated channel

### Discovery and artifact selection behavior

- Discovery should be broad enough to inspect GitHub releases, candidate AppImage assets, and adjacent metadata files
- Metadata should influence both install-time artifact selection and future update behavior
- Artifact selection should prefer metadata-guided choices first and filename heuristics second
- Stable releases should be preferred by default, unless the user explicitly started from a prerelease

### Update-channel behavior

- The preferred update channel should, by default, match the install origin as closely as possible
- The registry should retain as many validated alternate channels as practical so future updates can fall back automatically if the preferred path fails
- `aim` should be able to explain why a particular channel or artifact was chosen

## Recommended Architecture

Use a three-part core model inside `aim-core`:

- `source` discovers and fetches candidate release information, assets, and metadata documents
- `metadata` parses source-agnostic metadata formats such as `electron-builder` YAML or zsync-related metadata
- `update` ranks channels and artifacts, applies product rules, and persists a durable update strategy

This architecture was selected over keeping all upstream logic under `adapters`, because formats like `latest-linux.yml` and zsync metadata are not fundamentally tied to a single provider. They are metadata formats that can be discovered from different source types and should remain reusable.

## Core Boundary Decision

The critical rule is:

- `source` discovers
- `metadata` interprets
- `update` decides
- `app` orchestrates

That boundary keeps source-specific fetching logic separate from metadata parsing and prevents update rules from leaking into transport or parser code.

## Proposed Module Layout

Within `crates/aim-core/src/`, the design should evolve toward:

- `app/`
- `domain/`
- `source/`
- `metadata/`
- `update/`
- `registry/`
- `integration/`
- `platform/`

Responsibilities:

### `app`

- orchestration layer for add, update, list, and remove flows
- interactive decision boundaries
- conversion of lower-level results into user-facing plans and prompts

### `source`

- normalize user input into typed source references
- resolve GitHub shorthand, repo URLs, release URLs, and asset URLs
- fetch releases, assets, and candidate metadata documents
- detect candidate update channels without interpreting metadata semantics deeply

### `metadata`

- source-agnostic parsers for metadata document formats
- initial parsers should cover:
  - `electron-builder` Linux metadata such as `latest-linux.yml`
  - zsync-related metadata or update hints
- return structured hints, warnings, and confidence rather than UI-facing decisions

### `update`

- combine source discovery and parsed metadata
- rank channels and candidate artifacts
- apply install-origin-first default prioritization
- retain alternate channels for fallback
- build persisted update strategies and reviewable plans

### `domain` and `registry`

- `domain` holds the stable source-agnostic types used across the core
- `registry` stores only the durable information needed to explain and recover update behavior later

## Discovery And Update Flow

### 1. Input resolution

`source::resolve_input` should accept:

- `owner/repo`
- GitHub repo URL
- GitHub release URL
- direct release asset URL
- generic URL fallback

It should classify the input, retain the raw source input, and derive a normalized source reference. When possible, it should also derive a canonical app identity anchored to the GitHub repository.

### 2. Broad source discovery

The source layer should:

- fetch repository and release context
- enumerate candidate AppImage assets
- detect adjacent metadata files such as `latest-linux.yml`
- detect related update artifacts such as zsync files or embedded update hints when available

If the user supplied a link to an older release, discovery should still be broad enough to see newer releases, while preserving enough context for the application layer to ask whether the user wants to track that release lineage or switch to latest-supported updates.

### 3. Metadata parsing

Each metadata parser should consume raw document bytes plus lightweight fetch context and return structured hints such as:

- version
- artifact URL
- checksum or digest information
- channel or release label
- architecture or platform compatibility
- updater-family identity
- parser confidence
- warnings

Metadata parsing must remain source-agnostic. A GitHub discovery flow may hand documents to the parser, but the parser should not depend on GitHub-specific assumptions.

### 4. Channel construction

The update layer should turn source and metadata results into reusable channel records, for example:

- GitHub releases API channel
- `electron-builder` metadata channel
- zsync-derived channel
- direct-asset-lineage channel

Each channel should record:

- how it was discovered
- what artifacts it can produce
- its confidence and compatibility scope
- whether it matches the original install origin

### 5. Ranking and persistence

The update layer should select:

- one preferred channel
- an ordered list of validated alternate channels

Default rule:

- prefer the channel that best matches how the app was originally installed
- retain all other validated channels in fallback order

Artifact selection rule:

- metadata-guided first
- filename heuristics second

Prerelease rule:

- only prefer prerelease channels when the user explicitly started from one
- otherwise prefer stable releases first

## Domain Model

The design should introduce or evolve the following stable concepts.

### `AppIdentity`

- canonical tracked-app identity
- usually GitHub repository identity when resolvable
- includes normalized key and display name data

### `SourceInput`

- the exact user-provided input
- shorthand, repo URL, release URL, asset URL, or generic URL

### `SourceRef`

- normalized source locator used for discovery
- may represent a GitHub repo, a release lineage, a direct asset lineage, or another supported source pattern

### `MetadataDocument`

- raw fetched metadata plus provenance
- includes source URL, content type, fetch time, digest, document type guess, and raw content or a blob reference

### `MetadataHint`

- parsed interpretation of a metadata document
- includes install and update hints such as version, artifact URL, checksum, architecture, warnings, and confidence

### `UpdateChannel`

- a durable description of one update path
- examples include `github-releases`, `electron-builder`, `zsync`, and `direct-asset-lineage`

### `ArtifactCandidate`

- one installable AppImage candidate
- includes URL or path, version, architecture, provenance, checksum data, and score explanation

### `UpdateStrategy`

- preferred channel
- alternate channels in order
- preference reason such as install-origin match or stronger metadata confidence

## Registry Model

The registry should persist only the information that matters for future updates, explainability, and recovery.

Persist:

- canonical `AppIdentity`
- original `SourceInput`
- normalized `SourceRef`
- installed version and artifact state when known
- `UpdateStrategy`
- retained `UpdateChannel` records
- useful parsed metadata hints
- selected raw metadata snapshots or references

Do not persist every transient discovery result. The registry is not a general fetch cache. It should keep enough information to:

- explain why a channel was chosen
- retry alternates later
- survive upstream layout changes
- support debugging of bad matches

## Failure Handling

### Source failures

- source failures should be local, not globally fatal by default
- if GitHub API discovery fails but a direct asset URL remains usable, installation should continue through that path
- if one candidate discovery branch fails, the system should keep evaluating other validated branches where possible

### Metadata failures

- metadata parse failures should be typed and non-blocking
- failure to parse `latest-linux.yml` must not disable plain GitHub release discovery
- low-confidence metadata may still contribute hints, but should not outrank stronger direct evidence

### Update selection explanations

The update layer should be able to explain why a channel or artifact was preferred or rejected, using signals such as:

- install-origin match
- metadata confidence
- architecture compatibility
- prerelease mismatch
- stale, missing, or incompatible artifacts

## User Prompting Rules

Prompt only when the ambiguity materially changes behavior.

Expected prompt cases:

- the user linked an older specific release and the system can also see newer supported releases
- multiple AppImage artifacts remain equally plausible after metadata and heuristic ranking
- identity resolution remains low confidence after source normalization

Everything else should be automatic and explainable in the generated plan.

## Testing Strategy

### Source tests

- normalize shorthand, repo URLs, release URLs, and asset URLs
- verify broad discovery from mocked GitHub responses
- preserve prompt context for older-version inputs

### Metadata tests

- fixture-driven parsing for `electron-builder` YAML, zsync, and malformed inputs
- confidence and warning behavior
- source-agnostic parser contract

### Update tests

- install-origin-first ranking
- alternate-channel fallback ordering
- stable-versus-prerelease preference
- metadata-guided artifact selection outranking filename heuristics

### Registry tests

- round-trip persistence of preferred channel, alternates, metadata hints, and metadata snapshots
- backward-compatible loading when existing registry entries lack new fields

### End-to-end CLI tests

- add from GitHub shorthand
- add from direct GitHub release asset URL
- update behavior when preferred channel fails and an alternate succeeds

## Recommended Implementation Direction

Implementation should incrementally reshape the current adapter-oriented GitHub skeleton toward the new boundaries rather than attempt a single large rewrite.

Recommended sequence:

1. introduce `source`, `metadata`, and `update` domain boundaries and shared types
2. migrate GitHub discovery logic into the new source boundary
3. add metadata parsers and fixture coverage
4. add channel ranking and registry persistence extensions
5. update CLI orchestration and end-to-end tests

This keeps the change reviewable and protects the existing thin-client architecture where `aim-core` owns all reusable logic.