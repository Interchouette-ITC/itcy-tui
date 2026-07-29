# Contributing to itcy-tui

This repository is **public**. Treat every README line, commit subject, and PR title as public.

## Identity

- Product name: **ITCy** (Interchouette ITC). Not ICA.
- Author on worker commits: **Interchouette** `<contact@interchouette.net>`
- Sign commits with the Interchouette SSH signing key so GitHub shows Verified on the PR head

## Workflow

1. Branch from `dev` on the Interchouette fork (`feat/…`, `fix/…`, `docs/…`, `chore/…`).
2. Conventional commit subjects, for example `docs(tui): rewrite public README for GitHub visitors`.
3. Rebase onto `origin/dev` on the **fork** (keep commits signed), then push.
4. Open a PR into [Interchouette-ITC/itcy-tui](https://github.com/Interchouette-ITC/itcy-tui) `dev`.
5. Merge with squash (`gh pr merge --squash`). Do not rebase-merge onto the org remote.

## Do not

- Put Cursor plan/stage IDs (`S0`, `S4h`, …) in README, code comments, commits, or PR text
- Use local machine paths or private agent handoff tone in the README
- Append “Made with Cursor” to PR bodies
