# Task 1 Blocker Note

## Summary

Task 1 is blocked on an unresolved contract decision, not on a failing implementation.

The focused taxonomy test suite currently passes:

- `cargo test --package aim-core --test query_resolution`

But code review keeps surfacing the same underlying issue: some GitLab and SourceForge URL shapes are ambiguous when classified from URL structure alone.

## Current Blocker

Two classes of URLs cannot be classified with high confidence using only path-shape heuristics:

1. GitLab deep paths
   - Example ambiguity: a path segment may be either a subgroup slug or a resource-like segment.
   - This makes URLs such as deeply nested subgroup paths indistinguishable from some non-repository resource paths without provider-aware resolution.

2. SourceForge nested `files/.../download` paths
   - Some are concrete file downloads.
   - Some are folder-style or version-folder download endpoints.
   - URL shape alone does not reliably distinguish the two in every case.

## Why This Blocks Task 1

Task 1 is supposed to harden source taxonomy. It is not supposed to solve provider semantics end to end.

At this point, the remaining disagreement is about where ambiguity should be resolved:

- in classification, using increasingly complex heuristics
- or later, in provider-aware resolution logic

Trying to solve this entirely in Task 1 has led to oscillation between:

- permissive rules that accept too many ambiguous URLs
- conservative rules that reject URLs a reviewer considers potentially valid

## Recommended Resolution

Freeze Task 1 on the current conservative contract and move ambiguity handling into Task 2.

That keeps Task 1 scoped to explicit supported forms and lets the resolver contract decide how to treat ambiguous provider URLs with richer semantics.

## Current Practical State

- taxonomy tests are green
- GitLab, SourceForge, and direct URL shapes are covered by focused tests
- ambiguous provider URL handling remains the only unresolved review topic for Task 1