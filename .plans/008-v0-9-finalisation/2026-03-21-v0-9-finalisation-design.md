# v0.9 Finalisation Design

## Goal

Ship a credible v0.9 release that improves operational trust and product completeness without widening provider scope prematurely. This slice hardens downloads, enforces integrity checks, makes registry mutation safer, finalises a provider-extensible search contract with GitHub search as the first implementation, and aligns docs with the actual supported provider surface.

## Release Positioning

v0.9 is a trust-and-discoverability release.

It is not the true v1.0 provider-completion release. `custom-json` is explicitly deferred to v1.0. Broader provider discovery remains phase 2 work.

## In Scope

1. Stream artifact downloads to disk instead of buffering whole payloads in memory.
2. Add timeout and retry behavior to the download path.
3. Enforce checksum verification when trusted metadata provides a checksum.
4. Make registry mutation atomic and add advisory locking for mutating flows.
5. Finalise `aim search <query>` with a provider-extensible search abstraction in `aim-core` and GitHub as the first remote provider.
6. Update user-facing docs so supported providers and current search scope are described honestly.
7. Add regression coverage for the above behaviors.

## Explicitly Out Of Scope

1. Implementing `custom-json`.
2. Broad multi-provider remote search parity.
3. Adding `info`, `show`, `dry-run`, rollback, or version pinning.
4. Expanding GitLab or SourceForge install resolution beyond the currently defended contract.
5. Reworking the CLI into an interactive picker or install-from-search workflow.

## Architecture

### Download Hardening

The current add flow downloads into memory and only then stages the payload. v0.9 changes that boundary so the network layer streams into a staged file on disk. Payload validation and final commit remain owned by the install integration path, but the source of truth becomes a staged file rather than a `Vec<u8>` buffer.

This keeps memory usage effectively flat for large AppImages and makes retry or timeout policy attach naturally to the download operation.

### Integrity Enforcement

Checksum hints already exist in parsed metadata. v0.9 carries those hints through artifact selection and install execution so the staged payload can be verified before the final install commit. If a trusted checksum exists and validation fails, install must fail closed. If no checksum exists, install continues with no false claim of verification.

For v0.9, the only enforced trusted checksum contract is the existing electron-builder `sha512` field. That checksum must be treated as a base64-encoded SHA-512 digest of the raw payload bytes. Verification compares the base64 digest of the staged payload against the trimmed metadata value. A malformed trusted checksum is an install failure, not a warning.

### Registry Safety

Registry writes move to an atomic temp-file-and-rename pattern. Mutating commands also acquire an advisory registry lock so add, update, and remove cannot clobber each other through concurrent read-modify-write cycles.

The registry layer remains in `aim-core`; `aim-cli` should continue to orchestrate, not own persistence rules.

To avoid long lock retention, v0.9 does not hold the registry lock for network discovery, downloads, or desktop integration work. Instead, mutating flows acquire the exclusive advisory lock immediately before the registry transaction, reload the latest registry under lock, apply the final mutation by `stable_id`, save atomically, then release the lock. Read-only flows such as `list`, bare review, and `search` do not take the mutation lock.

### Search Architecture

Search becomes a first-class app flow in `aim-core`, not a CLI-only helper. The abstraction should be provider-extensible from the beginning, but only GitHub remote search is implemented in v0.9.

The stable shape is:

- `SearchQuery` for raw user intent and optional limits
- `SearchProvider` trait for provider-specific search backends
- `SearchResult` / `SearchResults` domain types for normalized output
- `build_search_results(...)` app entry point that aggregates remote provider hits and local installed matches

GitHub search should provide install-ready queries that feed the existing add flow. Non-GitHub providers can implement the same contract later without changing the CLI surface.

For v0.9, GitHub search is repository search only. The normalized install-ready query should be the canonical `owner/repo` form so search results feed the existing add flow without introducing a parallel install path.

### CLI Surface

`aim search <query>` is added as a read-only command.

Output should distinguish:

- remote provider results
- installed/local matches
- warnings such as partial provider failure or rate limit degradation

Search does not install anything in v0.9. It is a discovery surface only.

The CLI contract should also be deterministic:

- default remote result limit is 10
- GitHub remote hits preserve provider ranking order, with canonical locator as the stable tie-breaker when fixtures or adapters need explicit ordering
- local installed matches use case-insensitive substring matching against `stable_id` and `display_name`
- local matches render in a separate section and are sorted by exact match first, then prefix match, then substring match, with `stable_id` as the final tie-breaker

## Data Flow

### Add / Install

1. Resolve query into source semantics.
2. Discover release candidates and metadata as today.
3. Select artifact and attach optional checksum hint.
4. Stream artifact bytes into a staged file.
5. Verify checksum if available.
6. Validate payload shape.
7. Commit staged payload into final location.
8. Persist registry through the locked, atomic registry store.

If download, checksum verification, or payload validation fails, the staged file must be removed before returning the error.

### Search

1. Parse `aim search <query>`.
2. Build `SearchQuery`.
3. Run enabled providers, currently GitHub.
4. Normalize remote hits into provider-neutral results.
5. Derive local installed matches from the registry.
6. Render a stable CLI summary.

Search warnings must preserve partial-failure explainability. For v0.9, a GitHub rate limit or transport failure should become a warning if any local results still exist, and a command failure only if the overall command would otherwise produce no meaningful result.

## Error Handling

### Download Failures

- Connection and HTTP failures should be retried according to policy.
- Exhausted retries should surface as a clear install failure.
- Timeout should be explicit rather than hanging indefinitely.
- Partial staged payloads must be removed on failure.

### Checksum Failures

- A checksum mismatch is a hard error.
- A malformed trusted checksum is a hard error.
- Absence of checksum is not an error.
- Search results must never imply verified integrity.

### Registry Lock Failures

- A second mutating process should fail cleanly with an explicit lock message or short wait policy.
- Non-mutating flows like list and bare review should remain read-only and avoid unnecessary lock contention.
- The registry mutation transaction must reload the latest registry state while holding the lock before applying the final mutation.

### Search Failures

- Provider failure should degrade to warnings when at least one provider succeeds.
- Search should fail only when the overall operation cannot produce a meaningful result.
- Search ordering, limit behavior, and installed-match rules must be stable across runs.

## Testing Strategy

1. Unit tests for streaming or staged download helpers, retry policy, and checksum verification.
2. Registry store tests for atomic write behavior and lock semantics.
3. CLI command tests for `aim search --help` and search rendering.
4. Fixture-backed GitHub search tests in `aim-core`.
5. Install integration tests for checksum pass and checksum mismatch behavior.
6. Focused regression tests to ensure current GitLab, SourceForge, and direct URL install semantics do not regress.
7. Cleanup tests to ensure failed downloads or failed checksum validation do not leave staged payloads behind.

## Acceptance Criteria

1. Large artifact downloads no longer require full in-memory buffering.
2. Download timeout and retry policy exist and are covered by tests.
3. Trusted checksums are enforced before final install commit.
4. Registry writes are atomic and mutating commands do not race each other silently.
5. `aim search <query>` works end to end against GitHub fixtures.
6. Search architecture allows additional providers to be added in phase 2 without changing the public CLI contract.
7. README and related plan docs describe current provider and search scope honestly.
8. Failed download or checksum paths do not leave orphaned staged files behind.

## v1.0 Follow-On

The true v1.0 track can build on this by:

1. implementing `custom-json`
2. widening provider discovery beyond GitHub search
3. expanding provider install semantics where justified by defended tests
