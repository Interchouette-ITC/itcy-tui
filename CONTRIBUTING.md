# Contributing to itcy-tui

This repository is **public**. README, commits, and PR text are for strangers: product language only, no private ops or sibling-project comparisons.

## Identity

- This UI is for **ITCy** (Interchouette ITC LinkedIn operator).
- Worker commits: author **Interchouette** `<contact@interchouette.net>`, SSH-signed so the PR head shows Verified on GitHub.

## Workflow

1. Branch from `dev` on the Interchouette fork (`feat/…`, `fix/…`, `docs/…`, `chore/…`).
2. Conventional commit subjects, for example `docs(tui): clarify public README product voice`.
3. Rebase onto `origin/dev` on the **fork** (keep commits signed), then push.
4. Open a PR into [Interchouette-ITC/itcy-tui](https://github.com/Interchouette-ITC/itcy-tui) `dev`.
5. Merge with squash (`gh pr merge --squash`). Do not rebase-merge onto the org remote.

## Do not

- Put Cursor plan or stage codes in README, comments, commits, or PR text
- Put local machine paths, agent handoff tone, or other internal ops in the README
- Append “Made with Cursor” to PR bodies
