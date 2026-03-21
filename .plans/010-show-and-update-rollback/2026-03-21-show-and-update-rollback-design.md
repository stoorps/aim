# Show Command And Update Rollback Design

## Summary

This change adds a read-only `aim show <value>` command and hardens `aim update` so a failed reinstall restores the previously installed payload and generated integration files when possible. The public UX stays small: one `show` command for both installed-app inspection and install-query inspection, plus safer update execution without introducing a separate rollback command.

## Goals

- Add a single `aim show <value>` command for inspecting either an installed app or a resolvable install query.
- Resolve `show` inputs by checking installed apps first, then falling back to source/query resolution.
- Keep `show` read-only and reuse existing core resolution logic instead of creating a parallel inspection pipeline.
- Make update execution restore the previous installation files if the replacement install fails after touching the filesystem.
- Preserve the current registry-write ordering so only successful end-state records are written back.

## Non-Goals

- No standalone `aim rollback` command in this slice.
- No pinning or update-policy UX in this slice.
- No machine-readable `show --json` output yet.
- No registry backup or recovery feature beyond per-update install rollback.
- No redesign of existing `add`, `list`, `search`, or `remove` flows outside the minimum shared logic needed for `show` and rollback.

## Approaches

### Option 1: Separate `info` and `show` commands

This keeps installed-app inspection and remote-query inspection conceptually separate, but it forces the user to learn two entry points for what is effectively the same question: "what is this and what would aim do with it?" It also duplicates argument parsing, dispatch, help text, and render logic.

### Option 2: Make `show` a thin CLI-only wrapper around existing add and list code

This is faster to wire initially, but it would push installed lookup rules and query fallback rules into `aim-cli`, where they become harder to test and easier to drift from add/remove behavior. It also makes rollback hardening more likely to stay bolted onto `update.rs` without a clear boundary.

### Option 3: Add a unified core `show` service plus internal update rollback handling

This is the recommended approach. `aim-core` owns inspection and rollback behavior, while `aim-cli` remains responsible for command parsing and text rendering. The result is a small public surface with testable domain behavior and no extra persistent state.

## Approved Design

### Public Command Behavior

Add `aim show <value>` as a new subcommand. The command accepts the same broad input shapes already supported by install resolution: installed stable ids, display-name-like inputs, provider locators such as `owner/repo`, and supported URLs.

Resolution order is:

1. attempt installed-app lookup
2. if there is one clear installed match, show installed details
3. if there are no installed matches, fall back to query/source resolution
4. if installed lookup is ambiguous, fail without remote fallback

The ambiguity rule matters because an ambiguous installed match should not silently switch meaning and inspect a remote source instead. This should behave like the safe side of `remove`: when the input is not specific enough, the command tells the user to disambiguate.

### Installed-App Show Output

Installed inspection should render a concise but complete summary of the current record. The output should include:

- a compact summary line combining display name, stable id, installed version, and install scope
- a split title row with app name and stable id on the left, and installed version, inline update-status tag, and scope on the right when terminal width allows it
- a stacked fallback layout for narrower terminals rather than truncating the title row
- a secondary source line under the title row that shows provider and normalized source locator
- original source input only when it materially differs from the displayed source
- a compact `N past versions` history indicator below the source line, including the latest known version when the install is behind
- a small metadata block above files, with separate themed sibling lines for metadata kind plus architecture and for checksum
- installed payload, desktop entry, and icon paths rendered as the same style of secondary subinfo rather than a heavier bullet list

This is not intended to dump raw registry TOML. It should be a stable human-oriented summary that answers how the app was installed, whether it is behind, what file paths are currently tracked, and what update lineage exists without repeating a long metadata block.

### Remote Query Show Output

If installed lookup finds no match, `show` should resolve the input the same way `add` does, but stop before performing installation. The result should summarize:

- resolved source kind and locator
- selected artifact URL
- resolved version when available
- trusted checksum when available
- artifact selection reason
- interaction requirements if the add flow would require user choice or confirmation
- warnings produced during resolution

The remote path should reuse existing add-plan building logic rather than creating a second source-resolution implementation. This keeps install behavior and inspection behavior aligned.

### Core Architecture

Add a new inspection module in `aim-core`, with a small domain type such as `ShowResult` that covers the two successful result shapes:

- installed app details
- resolved remote add-plan details

The core service should accept the user input plus the current installed app list and return either a `ShowResult` or a typed error describing:

- ambiguous installed match
- unsupported query
- no installable artifact
- provider resolution failure

This keeps the installed-first policy and error classification in one place. `aim-cli` then only needs to parse the new command, dispatch to the core service, and render the returned structure.

### CLI Architecture

`aim-cli` should add a `Show { value: String }` subcommand and a corresponding `DispatchResult::Show(...)` branch. Rendering belongs in the existing text renderer alongside add/list/search/update summaries.

There should not be separate public `show-installed` and `show-remote` result types in the CLI. The renderer can branch on the shared `ShowResult` model and produce headings such as `Installed App` or `Resolved Source`.

### Update Rollback Design

Rollback belongs inside update execution, not in CLI dispatch. `execute_update(...)` already has the install boundary where the old app record, install home, and reinstall attempt are all visible. That is the right point to stage a backup, perform the install, and restore on failure.

Before reinstalling an app with tracked installation paths, update execution should:

1. collect the currently tracked payload, desktop entry, and icon paths that still exist
2. move those files into a rollback staging directory under the install home
3. attempt the replacement install
4. on success, delete the rollback staging directory
5. on failure, restore the old files to their original locations and return the original app record as the retained registry state

The rollback staging directory should be private to update execution, deterministic enough to debug, and cleaned up best-effort after either success or restore.

### Rollback Result Semantics

The registry should continue to be mutated only after update execution finishes, using the returned app list. That means the current high-level safety property remains unchanged:

- successful update returns the new record
- failed update returns the old record

The new behavior is filesystem safety. If reinstall fails after replacing or partially generating files, update execution should attempt to restore the old payload and integration files before reporting failure.

Restore failure should remain visible. The failure reason should include whether the install failed, whether rollback restoration also failed, and which files were involved. This can be represented as a richer failure string in this slice; a new structured rollback-status enum is not required unless the implementation clearly benefits from it.

### Edge Cases

- If an app has no tracked install paths, rollback is a no-op and the update can fail exactly as it does today.
- If backup creation fails before the replacement install starts, the update should abort rather than risk destructive partial replacement.
- If some tracked files are already missing, backup should proceed with the files that still exist and record the rest as warnings.
- If installed lookup for `show` is ambiguous, return an ambiguity error and do not attempt remote resolution.
- Unsupported source input and "no installable artifact" should remain distinct outcomes in the remote `show` path.
- `show` remains read-only even if the resolved add plan contains interactions; it should describe them rather than prompt.

## Testing Strategy

### Show Coverage

Add core tests for:

- exact installed match returning installed details
- no installed match falling back to remote resolution
- ambiguous installed matches returning a safe error
- unsupported input remaining distinct from no-installable-artifact
- remote result carrying artifact URL, checksum, and warnings through the summary model

Add CLI tests for:

- `aim show <installed-id>` rendering installed details
- `aim show <query>` rendering resolved source details
- ambiguity and provider errors surfacing readable messages

### Rollback Coverage

Add update execution tests for:

- successful update removes rollback staging artifacts
- failed update restores the original payload path
- failed update restores desktop entry and icon files when present
- backup creation failure aborts before destructive replacement
- restore failure reports a compound reason and still keeps the original record in registry output

The tests should prefer temporary directories and fixture transports over shelling out or relying on real network calls.

## Delivery Notes

- Do not add persistent rollback metadata to the registry for this slice.
- Prefer new focused modules for `show` rather than making `lib.rs` or `update.rs` absorb more branching.
- Keep the text output stable and human-readable so later `--json` work can be added as a separate renderer decision instead of reworking the domain model.