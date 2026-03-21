# AppImageHub Provider And Query Fallback Design

## Summary

This change adds AppImageHub as a first-class source and search provider, removes the unused `custom-json` stub, and changes positional `aim <query>` so it falls back to search when no direct installable match is available across providers. The direct-input contract stays deterministic: explicit provider URLs and shorthands still install immediately, while plain-name queries become a search-first discovery experience only after strict direct resolution fails.

## Goals

- Add AppImageHub as a supported provider for direct resolution and search.
- Accept direct AppImageHub inputs as either item URLs or `appimagehub/<id>` shorthand.
- Preserve strict direct resolution semantics for provider-specific inputs.
- Make positional `aim <query>` fall back to cross-provider search when direct resolution is unsupported or yields no installable artifact.
- Remove the `custom-json` adapter stub and its registration/tests.
- Keep provider identity stable in the registry by using AppImageHub numeric item IDs as canonical locators.

## Non-Goals

- No AppImageHub slug-based direct input format.
- No new interactive chooser or auto-install of fuzzy search winners in this slice.
- No redesign of `aim search` output beyond adding AppImageHub-backed results.
- No machine-readable provider metadata output beyond what current domain models already expose.
- No provider-agnostic alias layer for AppImageHub names.

## Approaches

### Option 1: Canonical IDs for direct AppImageHub input, names through search

This is the approved design. Direct AppImageHub inputs are limited to the observable stable forms `https://www.appimagehub.com/p/<id>` and `appimagehub/<id>`. Plain names are handled by the cross-provider search path, which can return install-ready `appimagehub/<id>` install queries. This keeps direct installs deterministic and avoids inventing a slug model the service does not clearly expose.

### Option 2: Title-based direct shorthand

This would accept human-readable AppImageHub names as if they were stable direct locators. It looks cleaner, but it collapses search and direct resolution into one fuzzy layer and makes title normalization, collisions, and title drift part of the persistent provider contract.

### Option 3: Provider alias registry

This would map friendly names to canonical AppImageHub IDs inside `aim`. It could improve CLI ergonomics later, but it adds storage, collision handling, update drift, and debugging overhead that are not justified for the initial provider integration.

## Approved Design

### Public Command Behavior

Positional `aim <query>` becomes a two-phase command:

1. attempt strict direct resolution using existing provider/source rules
2. if direct resolution yields an installable artifact, continue through the current add/install flow
3. if direct resolution is unsupported or resolves to a provider item with no installable artifact, fall back to cross-provider search

This preserves deterministic direct installs for URLs and provider shorthands while turning plain-name queries such as `aim firefox` into a discovery flow instead of a dead-end error.

The fallback rule is intentionally command-level policy, not a change to `resolve_query(...)`. Strict source classification should remain strict. The new behavior belongs in a higher-level orchestration layer that decides whether the positional command is an install or a search.

### AppImageHub Input Model

AppImageHub direct inputs are:

- `https://www.appimagehub.com/p/<id>`
- `http://www.appimagehub.com/p/<id>`
- `appimagehub/<id>`

The canonical provider identity is the numeric AppImageHub item ID. That ID should be stored as the canonical locator in the resolved source so the registry remains stable even if the user-facing title changes.

The source model gains:

- `SourceKind::AppImageHub`
- a `SourceInputKind` for AppImageHub URLs
- a `SourceInputKind` for AppImageHub shorthand
- `NormalizedSourceKind::AppImageHub`

The visible locator can remain the item page URL for readability, but provider matching and search/install identity should rely on the canonical ID.

### Provider Architecture

AppImageHub should be implemented as a real source adapter plus a real search provider.

The source adapter is responsible for:

- normalizing URL and shorthand inputs into a `SourceRef`
- resolving an AppImageHub item into the latest installable AppImage artifact
- returning a provider-specific no-installable-artifact outcome when the item exists but does not expose a usable AppImage asset

The search provider is responsible for:

- searching AppImageHub content by title or provider-supported query text
- mapping results into the existing `SearchResult` model
- returning install-ready queries as `appimagehub/<id>`

This keeps AppImageHub aligned with the existing GitHub/Search provider split and avoids tunneling it through generic direct URLs.

### Positional Query Fallback Flow

Add a new orchestration layer for positional `aim <query>` that makes the install-versus-search decision explicit. Conceptually:

- attempt build-add-plan
- if a direct install plan is produced, continue with the current install path
- if the add-plan path fails with unsupported query or no installable artifact, build search results instead
- if search finds hits, render search results instead of returning the add error
- if search finds nothing, render the normal empty search state instead of `unsupported source query`

This should be represented as a dedicated dispatch/core decision rather than scattered error remapping in CLI display code.

### Error Handling

The important distinction is between provider resolution and user-facing command behavior:

- strict direct resolution can still return `Unsupported`
- AppImageHub resolution can still return `NoInstallableArtifact`
- positional `aim <query>` treats those outcomes as search-fallback triggers
- explicit `aim search <query>` skips direct resolution entirely and always searches

Provider failures during search should continue to use the existing warning model. If all providers fail and no results are available, search should still surface provider failure warnings as today.

Malformed direct AppImageHub inputs should remain malformed direct inputs. They should not silently become provider-specific direct matches. If the text is not a valid direct AppImageHub source, it should either be an unsupported direct query or a plain search term depending on the command path.

### Search Result Semantics

AppImageHub search hits should populate:

- `provider_id = "appimagehub"`
- `display_name = <item title>`
- `source_locator = <item page url>`
- `install_query = appimagehub/<id>`
- `canonical_locator = <id>`

Installed-status annotation should treat AppImageHub IDs the same way GitHub currently treats canonical locators, so installed AppImageHub apps can show as installed or update-available inside search results.

### Removal Of `custom-json`

`custom-json` is a stub with no supported query or resolution behavior. It should be removed outright:

- delete the adapter module
- remove it from adapter registration
- remove it from smoke/contract coverage where it appears as an expected adapter kind

This slice should not replace it with another placeholder. AppImageHub is the real provider addition.

## Testing Strategy

### Query Classification Coverage

Add direct-resolution tests for:

- AppImageHub URL classification
- `appimagehub/<id>` classification
- malformed shorthand rejection

### Adapter Coverage

Add AppImageHub adapter tests for:

- successful normalization from URL and shorthand
- successful resolution to an installable AppImage artifact
- no-installable-artifact outcomes for valid provider items without usable assets

### Search Coverage

Add search tests for:

- AppImageHub provider hit mapping into `SearchResult`
- installed-status annotation based on AppImageHub canonical IDs
- cross-provider search continuing when one provider fails

### Positional Query Coverage

Add app/CLI tests for:

- positional query directly installing when a provider resolves installably
- positional query falling back to search on unsupported query
- positional query falling back to search on no-installable-artifact
- positional query rendering empty search output when fallback search returns no matches

### Registry/Smoke Coverage

Update adapter smoke and contract coverage to:

- remove `custom-json`
- include `appimagehub`

## Delivery Notes

- Keep `resolve_query(...)` strict; the fallback behavior belongs in a higher-level add/search decision.
- Prefer small extensions to existing domain types over introducing a second search result model.
- AppImageHub provider identity should be numeric-ID based from day one to avoid future migration churn.
- Do not add title-derived aliases in this slice.