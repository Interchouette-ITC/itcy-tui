# itcy-tui

Terminal for **ITCy**, Interchouette ITC's AI LinkedIn operator.

Built with [ratatui](https://ratatui.rs/). One chrome, four views:

- **Publications (`p`):** public GitHub trees for [itcy-publications](https://github.com/Interchouette-ITC/itcy-publications) (org and fork; `drafts`, `posts`, `drafts_tweet`, `tweets`). Works with no ITCy process.
- **Live (`d`):** when the [ITCy](https://github.com/Interchouette-ITC/itcy) binary is up, probes `GET /health` and `GET /status`, plus org ingress [itc-hooks](https://github.com/Interchouette-ITC/itc-hooks).
- **Commands (`c`):** slash catalog. Enter on `/list` injects a read-only list when the product is up.
- **Saved list (`s`):** parsed `/list` table. Needs the live product.

## Run

```bash
make test
make run
```

ITCy does not need to be running. If product `/health` is down, the browser opens on publications. If it is up, you land on live; press `p` for publications.

Optional: `ITCY_HEALTH_URL=http://127.0.0.1:4700/health make run`

Quit with `q` or Ctrl-C. `Esc` never quits: it closes `/` filter, `:` command, or help.

| Key | Action |
| --- | --- |
| `q` / Ctrl-C | Quit |
| `Esc` | Close overlay or help |
| `d` `c` `p` `s` | Live / commands / pubs / saved list |
| `?` | Help |
| `:` | Command palette (Tab complete, Up/Down history) |
| `/` | Filter the current table (`n/N`) |
| `y` | Copy selected id (OSC 52; mouse capture disables native select) |
| `r` | Refresh probes; refetch GitHub tree on pubs; re-run `/list` |
| `o` / `f` | Org / fork (publications) |
| `1`-`4` | drafts / posts / drafts_tweet / tweets |
| `Tab` | Focus list or preview |
| `j` `k` / arrows | Move selection (or scroll preview when focused) |
| `g` / `G` | First / last row |
| Enter | Load body now (pubs) or run `/list` (commands catalog) |
| click | Tabs, chips, table rows, list vs preview |
| wheel | Scroll the pane under the pointer |

Colon commands: `live` `commands` `pubs` `list` `help` `org` `fork` `drafts` `posts` `drafts_tweet` `tweets` `reload` `open <id-prefix>`.

## What you see

**Live:** ITCy health, ingress open/down, model routes, GitHub hooks delivery, enrich-queue progress, Tor. Product logs stay with the ITCy process. Public org webhook path is on itc-hooks (`POST /github/webhook_ITC`); ITCy receives `POST /hooks/github` only.

**Publications:** id, `YYYY/MM` shard, subject (filled after a body load this session), `body.md` preview. Selection loads after 400ms; Enter loads now. The selected id stays large in the footer for `y`.

**Commands:** slash table. Click selects; Enter on `/list` injects when ITCy is up. Other rows are documentation.

**Saved list:** id, status, subject from `/list`. Grey / empty when the product is down.

## License

BUSL-1.1 (Interchouette-ITC). See [LICENSE](LICENSE).
