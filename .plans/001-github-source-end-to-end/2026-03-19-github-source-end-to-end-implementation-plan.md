# GitHub Source End-to-End Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add end-to-end GitHub source support in `aim-core` for shorthand, repo URLs, release URLs, and direct asset URLs, with source-agnostic metadata parsing, install-origin-first channel selection, and registry-backed fallback channels.

**Architecture:** Reshape the current GitHub skeleton from an adapter-centric model into explicit `source`, `metadata`, and `update` boundaries. Keep `aim-cli` thin by moving normalization, metadata interpretation, channel ranking, and recovery behavior into `aim-core`, then extend the registry so future updates can survive upstream changes.

**Tech Stack:** Rust, Cargo workspace, serde, toml, reqwest-compatible fetch abstractions, clap, dialoguer, assert_cmd, predicates, fixture-driven tests in `crates/aim-core/tests` and `crates/aim-cli/tests`.

---

### Task 1: Introduce the new core boundary modules and types

**Files:**
- Create: `crates/aim-core/src/source/mod.rs`
- Create: `crates/aim-core/src/source/input.rs`
- Create: `crates/aim-core/src/source/github.rs`
- Create: `crates/aim-core/src/metadata/mod.rs`
- Create: `crates/aim-core/src/metadata/document.rs`
- Create: `crates/aim-core/src/update/mod.rs`
- Modify: `crates/aim-core/src/lib.rs`
- Modify: `crates/aim-core/src/domain/source.rs`
- Modify: `crates/aim-core/src/domain/update.rs`
- Test: `crates/aim-core/tests/query_resolution.rs`

**Step 1: Write the failing test**

```rust
use aim_core::source::input::{classify_input, SourceInputKind};

#[test]
fn classifies_github_release_asset_url() {
    let input = classify_input(
        "https://github.com/pingdotgg/t3code/releases/download/v0.0.11/T3-Code-0.0.11-x86_64.AppImage",
    )
    .unwrap();

    assert_eq!(input.kind, SourceInputKind::GitHubReleaseAssetUrl);
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test classifies_github_release_asset_url --package aim-core --test query_resolution`
Expected: FAIL because the `source` module and new input classification types do not exist yet

**Step 3: Write minimal implementation**

Add the new top-level modules and export them from `aim_core`. Introduce the minimum source and update domain types needed to classify GitHub inputs without rewriting existing workflows yet.

**Step 4: Run test to verify it passes**

Run: `cargo test classifies_github_release_asset_url --package aim-core --test query_resolution`
Expected: PASS

**Step 5: Commit**

```bash
git add crates/aim-core/src/lib.rs crates/aim-core/src/source crates/aim-core/src/metadata crates/aim-core/src/update crates/aim-core/src/domain/source.rs crates/aim-core/src/domain/update.rs crates/aim-core/tests/query_resolution.rs
git commit -m "feat: add source metadata and update module boundaries"
```

### Task 2: Implement GitHub input normalization across all supported entry forms

**Files:**
- Modify: `crates/aim-core/src/source/input.rs`
- Modify: `crates/aim-core/src/source/github.rs`
- Modify: `crates/aim-core/src/app/query.rs`
- Modify: `crates/aim-core/src/app/identity.rs`
- Test: `crates/aim-core/tests/query_resolution.rs`
- Test: `crates/aim-core/tests/identity_resolution.rs`

**Step 1: Write the failing test**

```rust
use aim_core::app::query::resolve_query;
use aim_core::domain::source::{NormalizedSourceKind, SourceInputKind};

#[test]
fn resolves_owner_repo_to_github_repo_source() {
    let source = resolve_query("sharkdp/bat").unwrap();
    assert_eq!(source.input_kind, SourceInputKind::RepoShorthand);
    assert_eq!(source.normalized_kind, NormalizedSourceKind::GitHubRepository);
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test resolves_owner_repo_to_github_repo_source --package aim-core --test query_resolution`
Expected: FAIL because normalized GitHub source kinds are not represented yet

**Step 3: Write minimal implementation**

Teach query resolution and identity normalization to recognize:
- `owner/repo`
- GitHub repo URLs
- GitHub release URLs
- direct GitHub release asset URLs

Preserve the original input while returning a normalized source reference that can later drive discovery.

**Step 4: Run test to verify it passes**

Run: `cargo test resolves_owner_repo_to_github_repo_source --package aim-core --test query_resolution`
Expected: PASS

**Step 5: Commit**

```bash
git add crates/aim-core/src/source/input.rs crates/aim-core/src/source/github.rs crates/aim-core/src/app/query.rs crates/aim-core/src/app/identity.rs crates/aim-core/tests/query_resolution.rs crates/aim-core/tests/identity_resolution.rs
git commit -m "feat: normalize github input forms"
```

### Task 3: Add GitHub discovery records for releases, assets, and linked metadata

**Files:**
- Modify: `crates/aim-core/src/source/github.rs`
- Modify: `crates/aim-core/src/domain/source.rs`
- Create: `crates/aim-core/tests/github_source_discovery.rs`
- Modify: `crates/aim-core/src/adapters/test_support.rs`

**Step 1: Write the failing test**

```rust
use aim_core::source::github::discover_github_candidates;

#[test]
fn discovery_reports_appimage_assets_and_latest_linux_yml() {
    let discovery = discover_github_candidates(/* mocked github response */).unwrap();

    assert!(discovery
        .assets
        .iter()
        .any(|asset| asset.name.ends_with(".AppImage")));
    assert!(discovery
        .metadata_documents
        .iter()
        .any(|doc| doc.url.ends_with("latest-linux.yml")));
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test discovery_reports_appimage_assets_and_latest_linux_yml --package aim-core --test github_source_discovery`
Expected: FAIL because source discovery does not yet return structured assets and metadata document records

**Step 3: Write minimal implementation**

Add GitHub discovery result types that expose:
- releases
- AppImage assets
- discovered metadata document URLs
- enough provenance to support later prompt and ranking logic

Use existing test-support scaffolding rather than real network calls.

**Step 4: Run test to verify it passes**

Run: `cargo test discovery_reports_appimage_assets_and_latest_linux_yml --package aim-core --test github_source_discovery`
Expected: PASS

**Step 5: Commit**

```bash
git add crates/aim-core/src/source/github.rs crates/aim-core/src/domain/source.rs crates/aim-core/src/adapters/test_support.rs crates/aim-core/tests/github_source_discovery.rs
git commit -m "feat: add github source discovery records"
```

### Task 4: Add source-agnostic metadata document and parser contracts

**Files:**
- Modify: `crates/aim-core/src/metadata/mod.rs`
- Modify: `crates/aim-core/src/metadata/document.rs`
- Create: `crates/aim-core/src/metadata/parser.rs`
- Modify: `crates/aim-core/src/domain/update.rs`
- Create: `crates/aim-core/tests/metadata_contract.rs`

**Step 1: Write the failing test**

```rust
use aim_core::metadata::{parse_document, MetadataDocument, ParsedMetadataKind};

#[test]
fn unknown_document_returns_typed_warning_not_panic() {
    let doc = MetadataDocument::plain_text("https://example.test/notes.txt", b"not metadata");
    let result = parse_document(&doc).unwrap();

    assert_eq!(result.kind, ParsedMetadataKind::Unknown);
    assert!(!result.warnings.is_empty());
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test unknown_document_returns_typed_warning_not_panic --package aim-core --test metadata_contract`
Expected: FAIL because the metadata parsing contract does not exist yet

**Step 3: Write minimal implementation**

Define:
- metadata document input type
- metadata parse result type
- source-agnostic parser entry point
- typed warnings for unsupported or malformed documents

Keep the implementation minimal and independent from GitHub-specific code.

**Step 4: Run test to verify it passes**

Run: `cargo test unknown_document_returns_typed_warning_not_panic --package aim-core --test metadata_contract`
Expected: PASS

**Step 5: Commit**

```bash
git add crates/aim-core/src/metadata crates/aim-core/src/domain/update.rs crates/aim-core/tests/metadata_contract.rs
git commit -m "feat: add metadata parser contract"
```

### Task 5: Implement `electron-builder` Linux metadata parsing

**Files:**
- Create: `crates/aim-core/src/metadata/electron_builder.rs`
- Modify: `crates/aim-core/src/metadata/mod.rs`
- Test: `crates/aim-core/tests/metadata_electron_builder.rs`
- Create: `crates/aim-core/tests/fixtures/latest-linux.yml`

**Step 1: Write the failing test**

```rust
use aim_core::metadata::{parse_document, MetadataDocument, ParsedMetadataKind};

#[test]
fn parses_latest_linux_yml_into_download_hints() {
    let raw = include_bytes!("fixtures/latest-linux.yml");
    let doc = MetadataDocument::yaml("https://example.test/latest-linux.yml", raw);
    let result = parse_document(&doc).unwrap();

    assert_eq!(result.kind, ParsedMetadataKind::ElectronBuilder);
    assert_eq!(result.hints.primary_download.as_deref(), Some("T3-Code-0.0.11-x86_64.AppImage"));
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test parses_latest_linux_yml_into_download_hints --package aim-core --test metadata_electron_builder`
Expected: FAIL because `electron-builder` metadata is not parsed yet

**Step 3: Write minimal implementation**

Add an `electron_builder` parser that extracts:
- version
- primary download artifact
- checksum or digest when present
- architecture hints when available
- parser confidence and warnings

**Step 4: Run test to verify it passes**

Run: `cargo test parses_latest_linux_yml_into_download_hints --package aim-core --test metadata_electron_builder`
Expected: PASS

**Step 5: Commit**

```bash
git add crates/aim-core/src/metadata/electron_builder.rs crates/aim-core/src/metadata/mod.rs crates/aim-core/tests/metadata_electron_builder.rs crates/aim-core/tests/fixtures/latest-linux.yml
git commit -m "feat: parse electron builder linux metadata"
```

### Task 6: Implement zsync metadata parsing and channel hints

**Files:**
- Create: `crates/aim-core/src/metadata/zsync.rs`
- Modify: `crates/aim-core/src/metadata/mod.rs`
- Test: `crates/aim-core/tests/metadata_zsync.rs`
- Create: `crates/aim-core/tests/fixtures/example.zsync`

**Step 1: Write the failing test**

```rust
use aim_core::metadata::{parse_document, MetadataDocument, ParsedMetadataKind};

#[test]
fn parses_zsync_document_into_channel_hints() {
    let raw = include_bytes!("fixtures/example.zsync");
    let doc = MetadataDocument::plain_text("https://example.test/app.AppImage.zsync", raw);
    let result = parse_document(&doc).unwrap();

    assert_eq!(result.kind, ParsedMetadataKind::Zsync);
    assert!(result.hints.primary_download.is_some());
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test parses_zsync_document_into_channel_hints --package aim-core --test metadata_zsync`
Expected: FAIL because zsync parsing does not exist yet

**Step 3: Write minimal implementation**

Add a zsync parser that extracts download URL, filename, version-like hints where possible, and channel confidence without coupling it to one upstream source.

**Step 4: Run test to verify it passes**

Run: `cargo test parses_zsync_document_into_channel_hints --package aim-core --test metadata_zsync`
Expected: PASS

**Step 5: Commit**

```bash
git add crates/aim-core/src/metadata/zsync.rs crates/aim-core/src/metadata/mod.rs crates/aim-core/tests/metadata_zsync.rs crates/aim-core/tests/fixtures/example.zsync
git commit -m "feat: parse zsync metadata documents"
```

### Task 7: Add update-channel ranking and artifact scoring

**Files:**
- Modify: `crates/aim-core/src/update/mod.rs`
- Create: `crates/aim-core/src/update/channels.rs`
- Create: `crates/aim-core/src/update/ranking.rs`
- Modify: `crates/aim-core/src/app/update.rs`
- Modify: `crates/aim-core/src/domain/update.rs`
- Modify: `crates/aim-core/tests/update_planning.rs`

**Step 1: Write the failing test**

```rust
use aim_core::update::ranking::rank_channels;

#[test]
fn install_origin_match_beats_higher_level_fallback() {
    let ranked = rank_channels(/* preferred direct asset lineage */, /* github releases */, /* electron-builder */);
    assert_eq!(ranked[0].reason.as_str(), "install-origin-match");
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test install_origin_match_beats_higher_level_fallback --package aim-core --test update_planning`
Expected: FAIL because channels and ranking reasons are not modeled yet

**Step 3: Write minimal implementation**

Implement channel and artifact ranking rules for:
- install-origin-first preference
- stable-over-prerelease by default
- metadata-guided artifact selection ahead of filename heuristics
- ordered alternates retained for fallback

**Step 4: Run test to verify it passes**

Run: `cargo test install_origin_match_beats_higher_level_fallback --package aim-core --test update_planning`
Expected: PASS

**Step 5: Commit**

```bash
git add crates/aim-core/src/update crates/aim-core/src/app/update.rs crates/aim-core/src/domain/update.rs crates/aim-core/tests/update_planning.rs
git commit -m "feat: add update channel ranking"
```

### Task 8: Extend the registry model for source input, strategy, and fallback channels

**Files:**
- Modify: `crates/aim-core/src/registry/model.rs`
- Modify: `crates/aim-core/src/registry/store.rs`
- Modify: `crates/aim-core/src/domain/app.rs`
- Modify: `crates/aim-core/tests/registry_roundtrip.rs`

**Step 1: Write the failing test**

```rust
use aim_core::registry::store::RegistryStore;

#[test]
fn registry_round_trips_update_strategy_and_alternates() {
    let store = RegistryStore::new(/* temp path */);
    let original = sample_record_with_strategy();

    store.save(&[original.clone()]).unwrap();
    let loaded = store.load().unwrap();

    assert_eq!(loaded[0].update_strategy.preferred.reason, "install-origin-match");
    assert_eq!(loaded[0].update_strategy.alternates.len(), 2);
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test registry_round_trips_update_strategy_and_alternates --package aim-core --test registry_roundtrip`
Expected: FAIL because the registry cannot persist the new strategy fields yet

**Step 3: Write minimal implementation**

Extend the registry model to persist:
- original source input
- normalized source reference
- preferred channel
- ordered alternates
- selected metadata hints or snapshot references

Keep loading backward-compatible for existing records that lack these fields.

**Step 4: Run test to verify it passes**

Run: `cargo test registry_round_trips_update_strategy_and_alternates --package aim-core --test registry_roundtrip`
Expected: PASS

**Step 5: Commit**

```bash
git add crates/aim-core/src/registry/model.rs crates/aim-core/src/registry/store.rs crates/aim-core/src/domain/app.rs crates/aim-core/tests/registry_roundtrip.rs
git commit -m "feat: persist update strategy and fallback channels"
```

### Task 9: Wire the add flow through source discovery, metadata parsing, and channel selection

**Files:**
- Modify: `crates/aim-core/src/app/add.rs`
- Modify: `crates/aim-core/src/app/identity.rs`
- Modify: `crates/aim-core/src/source/github.rs`
- Modify: `crates/aim-core/tests/github_add_flow.rs`

**Step 1: Write the failing test**

```rust
use aim_core::app::add::build_add_plan;

#[test]
fn add_plan_prefers_metadata_guided_appimage_when_available() {
    let plan = build_add_plan(/* github shorthand with latest-linux.yml */).unwrap();

    assert_eq!(plan.selected_artifact.selection_reason, "metadata-guided");
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test add_plan_prefers_metadata_guided_appimage_when_available --package aim-core --test github_add_flow`
Expected: FAIL because add planning does not yet route through metadata-aware ranking

**Step 3: Write minimal implementation**

Update add planning to:
- resolve normalized GitHub source input
- perform discovery
- parse any fetched metadata documents
- rank channels and artifacts
- emit a plan that records why the winning artifact was chosen

**Step 4: Run test to verify it passes**

Run: `cargo test add_plan_prefers_metadata_guided_appimage_when_available --package aim-core --test github_add_flow`
Expected: PASS

**Step 5: Commit**

```bash
git add crates/aim-core/src/app/add.rs crates/aim-core/src/app/identity.rs crates/aim-core/src/source/github.rs crates/aim-core/tests/github_add_flow.rs
git commit -m "feat: wire add flow through source and metadata pipeline"
```

### Task 10: Add prompt context for older releases and ambiguous artifacts

**Files:**
- Modify: `crates/aim-core/src/app/interaction.rs`
- Modify: `crates/aim-core/src/app/add.rs`
- Modify: `crates/aim-cli/src/ui/prompt.rs`
- Modify: `crates/aim-cli/tests/ui_summary.rs`
- Modify: `crates/aim-cli/tests/end_to_end_cli.rs`

**Step 1: Write the failing test**

```rust
use aim_core::app::add::build_add_plan;

#[test]
fn direct_old_release_url_requests_tracking_choice_prompt() {
    let plan = build_add_plan(/* direct old github asset url with newer releases available */).unwrap();

    assert!(plan
        .interactions
        .iter()
        .any(|item| item.key == "tracking-preference"));
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test direct_old_release_url_requests_tracking_choice_prompt --package aim-core --test github_add_flow`
Expected: FAIL because the flow does not surface the new prompt context yet

**Step 3: Write minimal implementation**

Add typed prompt requests for:
- older explicit release versus latest-supported tracking
- ambiguous artifact ties after metadata and heuristics

Keep prompt rendering in `aim-cli`, but define the request shape in `aim-core`.

**Step 4: Run test to verify it passes**

Run: `cargo test direct_old_release_url_requests_tracking_choice_prompt --package aim-core --test github_add_flow`
Expected: PASS

**Step 5: Commit**

```bash
git add crates/aim-core/src/app/interaction.rs crates/aim-core/src/app/add.rs crates/aim-cli/src/ui/prompt.rs crates/aim-cli/tests/ui_summary.rs crates/aim-cli/tests/end_to_end_cli.rs
git commit -m "feat: add prompt support for github tracking choices"
```

### Task 11: Teach update planning to fall back when the preferred channel fails

**Files:**
- Modify: `crates/aim-core/src/app/update.rs`
- Modify: `crates/aim-core/src/update/ranking.rs`
- Modify: `crates/aim-core/tests/update_planning.rs`
- Modify: `crates/aim-cli/tests/end_to_end_cli.rs`

**Step 1: Write the failing test**

```rust
use aim_core::app::update::build_update_plan;

#[test]
fn update_plan_uses_alternate_channel_after_preferred_failure() {
    let plan = build_update_plan(/* registry entry with failing preferred channel */).unwrap();

    assert_eq!(plan.updates[0].selected_channel.kind.as_str(), "electron-builder");
    assert_eq!(plan.updates[0].selection_reason.as_str(), "preferred-channel-failed");
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test update_plan_uses_alternate_channel_after_preferred_failure --package aim-core --test update_planning`
Expected: FAIL because update planning does not yet retry alternates

**Step 3: Write minimal implementation**

Teach update planning to:
- evaluate the preferred channel first
- fall back through ordered alternates when the preferred path is stale, broken, or incompatible
- preserve an explanation for the fallback decision

**Step 4: Run test to verify it passes**

Run: `cargo test update_plan_uses_alternate_channel_after_preferred_failure --package aim-core --test update_planning`
Expected: PASS

**Step 5: Commit**

```bash
git add crates/aim-core/src/app/update.rs crates/aim-core/src/update/ranking.rs crates/aim-core/tests/update_planning.rs crates/aim-cli/tests/end_to_end_cli.rs
git commit -m "feat: add update fallback channel behavior"
```

### Task 12: Remove or slim the legacy GitHub adapter entry points

**Files:**
- Modify: `crates/aim-core/src/adapters/mod.rs`
- Modify: `crates/aim-core/src/adapters/github.rs`
- Modify: `crates/aim-core/src/adapters/traits.rs`
- Modify: `crates/aim-core/tests/adapter_contract.rs`
- Modify: `crates/aim-core/tests/adapter_smoke.rs`

**Step 1: Write the failing test**

```rust
use aim_core::adapters::github::GitHubAdapter;

#[test]
fn legacy_github_adapter_delegates_to_source_pipeline() {
    let adapter = GitHubAdapter::default();
    let result = adapter.normalize("sharkdp/bat").unwrap();
    assert_eq!(result.normalized_kind.as_str(), "github-repository");
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test legacy_github_adapter_delegates_to_source_pipeline --package aim-core --test adapter_contract`
Expected: FAIL because the legacy adapter layer has not been reconciled with the new boundaries

**Step 3: Write minimal implementation**

Either:
- slim the GitHub adapter into a compatibility wrapper over `source::github`, or
- reduce the adapter layer so it no longer owns metadata or ranking responsibilities

Do not leave duplicated GitHub logic in both places.

**Step 4: Run test to verify it passes**

Run: `cargo test legacy_github_adapter_delegates_to_source_pipeline --package aim-core --test adapter_contract`
Expected: PASS

**Step 5: Commit**

```bash
git add crates/aim-core/src/adapters/mod.rs crates/aim-core/src/adapters/github.rs crates/aim-core/src/adapters/traits.rs crates/aim-core/tests/adapter_contract.rs crates/aim-core/tests/adapter_smoke.rs
git commit -m "refactor: reconcile legacy adapter layer with source pipeline"
```

### Task 13: Run full verification and update top-level docs if needed

**Files:**
- Modify: `README.md`
- Modify: `.plans/001-github-source-end-to-end/2026-03-19-github-source-end-to-end-design.md`
- Modify: `.plans/001-github-source-end-to-end/2026-03-19-github-source-end-to-end-implementation-plan.md`

**Step 1: Run focused test suites**

Run: `cargo test --package aim-core --test query_resolution --test identity_resolution --test github_source_discovery --test metadata_contract --test metadata_electron_builder --test metadata_zsync --test github_add_flow --test update_planning --test registry_roundtrip`
Expected: PASS

**Step 2: Run full workspace verification**

Run: `cargo test --workspace`
Expected: PASS

Run: `cargo fmt --check`
Expected: PASS

Run: `cargo clippy --workspace --all-targets --all-features -- -D warnings`
Expected: PASS

**Step 3: Update docs minimally**

Document any visible changes to supported GitHub input forms or update behavior in `README.md`. Only update the design or plan docs if implementation forced a justified divergence.

**Step 4: Re-run doc-relevant tests if docs changed code examples**

Run: `cargo test --workspace`
Expected: PASS

**Step 5: Commit**

```bash
git add README.md .plans/001-github-source-end-to-end/2026-03-19-github-source-end-to-end-design.md .plans/001-github-source-end-to-end/2026-03-19-github-source-end-to-end-implementation-plan.md
git commit -m "docs: finalize github source end-to-end support"
```