# itcy-tui

Terminal for **ITCy**, Interchouette ITC's AI LinkedIn operator.

Built with [ratatui](https://ratatui.rs/). Two jobs in one binary:

- **Publications browser:** reads public GitHub trees for [itcy-publications](https://github.com/Interchouette-ITC/itcy-publications) (org and fork, branches `drafts`, `posts`, `drafts_tweet`, `tweets`). Works with no ITCy process.
- **Live dashboard:** when the [ITCy](https://github.com/Interchouette-ITC/itcy) binary is up, probes `GET /health` and `GET /status`, plus org ingress [itc-hooks](https://github.com/Interchouette-ITC/itc-hooks). Saved drafts (`/list`) need that live product.

## Run

```bash
make test
make run
```

ITCy does not need to be running. If product `/health` is down, the browser opens on publications. If it is up, you land on the live dashboard; press `p` for publications.

Optional: `ITCY_HEALTH_URL=http://127.0.0.1:4700/health make run`

| Key | Action |
| --- | --- |
| `q` / Esc / Ctrl-C | Quit |
| `r` | Refresh (health/status; also refetch the GitHub tree on publications) |
| `d` | Live dashboard |
| `c` | Slash-command reference |
| `p` | Publications browser |
| `s` | Saved list (`/list`; needs ITCy) |
| `o` / `f` | Org / fork remote (publications) |
| `Tab` / `1`-`4` | Cycle or jump branch |
| `j` `k` / arrows | Select artefact |
| Enter | Load `body.md` |
| `/` | Filter by id |
| PgUp / PgDown | Scroll preview |

## What you see

**Live:** ITCy health, ingress open/down, model routes, GitHub hooks delivery, enrich-queue progress, Tor. Product logs stay with the ITCy process. Public org webhook path is on itc-hooks (`POST /github/webhook_ITC`); ITCy receives `POST /hooks/github` only.

**Publications:** artefact ids, optional `YYYY/MM` shard, `body.md` preview. The selected id is shown large for copy-paste.

**Commands:** one-screen slash cheat sheet. `s` runs `/list` when ITCy is up.

## License

BUSL-1.1 (Interchouette-ITC). See [LICENSE](LICENSE).
