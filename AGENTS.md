## Plans, designs, and specs locations.
IMPORTANT IF USING BRAINSTORMING SKILL!!!
Docuemnts of these types MUST live in `.plans/` within a sensibly named and indexed subfolder. This can be branch name or a feature name like `001-initial-implementation`, `002-adding-gitlab-provider`, etc.

## Audits
IMPORTANT!!
Audits are to live in `.audits` with a good name slug plus time & date.

## Architecture
IMPORTANT TO CHECK BEFORE ANY COMMIT!!
Architecture under `.architecture` must be maintained with each change.
 - Security issues stumbled upon or noticed during execution **already in code** must live in `security-issues.md`. Newly added issues during execution or planning should be raised to the user and/or dealt with, instead of growing the list. 
 - An overview of the workspace, should live in `overview.md`.
 - A roadmap of the planned direction & features in `roadmap.md`.

 ## UPM work
 We are currently using the `upm` branch as our "main" for all `upm` work, meaning feature branches/worktrees will merge back into here. 