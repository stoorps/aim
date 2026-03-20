# Update Execution And Provider Contract Design

## Goal

Add real update execution to `aim` and tighten the provider abstraction so additional providers can implement a consistent normalize-and-resolve contract instead of each adapter inventing its own shape.

## Agreed Product Shape

### Update behavior

- `aim` with no args remains a review surface for installed updates
- `aim update` executes updates for all registered apps
- Update execution reuses the existing install engine so payload staging, desktop integration, icon extraction, and rollback semantics stay in one place
- Successful updates replace the stored app record with the newly installed record
- Failed updates are reported per app and do not prevent other apps from updating

### Provider contract

- `SourceAdapter` should define the core behaviors providers are expected to implement:
  - `normalize(query)`
  - `resolve(source)`
- Adapters should return a shared adapter error type rather than bespoke per-adapter error enums for basic contract failures
- Existing GitHub and GitLab adapters should be brought under that shared contract first

## Recommended Approach

Use a reuse-first update executor.

Instead of building a second install path for updates, the update flow should turn each installed app back into an install request and run it through `build_add_plan(...)` and `install_app(...)` using the stored source and install scope metadata. This keeps update behavior aligned with add/install and avoids duplicating staging, desktop integration, and rollback logic.

For adapters, tighten the trait with the minimum common operations now, without prematurely forcing full discovery logic for every provider. That gives the next provider a clear implementation target while keeping the current GitHub path intact.

## Update Execution Model

### 1. Select update targets

- Build the existing update review plan from installed app records
- Treat every planned item as an executable candidate when `aim update` runs

### 2. Reconstruct install intent

For each app:

- use `source_input` when available
- otherwise fall back to stored source locator
- use persisted install scope from install metadata when available
- default legacy records to user scope if no metadata exists

### 3. Apply update

- build a fresh add plan for the app query
- install using the existing install engine
- persist the newly returned `AppRecord`

### 4. Report results

For each app capture:

- updated app id and display name
- success or failure
- version before and after when available
- warnings from install/update execution

### 5. Persist registry last

- save the registry after processing all apps
- successful app records replace old ones
- failed apps retain their previous records

## Failure Model

- per-app failures do not abort the entire update command
- install failures reuse existing install cleanup behavior
- the command returns a summary including updated count and failed count
- failed updates leave the previous registry entry intact

## Provider Contract Model

### Trait requirements

`SourceAdapter` should require:

- `id()`
- `capabilities()`
- `normalize(query)`
- `resolve(source)`

### Shared error type

Use a small shared adapter error enum for contract-level failures such as:

- unsupported query
- unsupported source
- resolution failure

Provider-specific rich errors can still exist behind the provider implementation if needed later, but the contract should expose a stable top-level error shape now.

## Verification Strategy

### Update execution tests

- CLI update applies updates for installed apps instead of only rendering a plan
- update failure keeps previous registry entry intact and reports failure
- update uses persisted install scope for reinstall path

### Provider contract tests

- contract tests verify GitHub and GitLab implement normalize/resolve through the trait
- adapter smoke tests continue to prove all expected adapters are registered

## Non-Goals

- interactive per-app update selection in this slice
- provider-specific live discovery for every non-GitHub adapter
- refactoring the entire source pipeline around adapters in this change
