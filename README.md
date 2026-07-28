# itcy-tui

Ratatui status UI for **ITCy** (Interchouette ITC AI experience).

| | |
| --- | --- |
| **Canonical** | [Interchouette-ITC/itcy-tui](https://github.com/Interchouette-ITC/itcy-tui) |
| **Worker fork** | [Interchouette/itcy-tui](https://github.com/Interchouette/itcy-tui) |
| **Local** | `/opt3/itcy/tui` |
| **Pairs with** | product [Interchouette-ITC/itcy](https://github.com/Interchouette-ITC/itcy) (`GET /health` on `:4700`) |

Early status pane for the product. Develop on the **Interchouette** fork; open PRs into **Interchouette-ITC**.

## Run

```bash
# Product must already answer /health (do not start a second listener if :4700 is taken):
curl -s http://localhost:4700/health   # expect: ok
make test
make run
# Full-screen TUI. Quit: q / Esc / Ctrl-C. Refresh: r. Commands: c
# override URL: ITCY_HEALTH_URL=http://127.0.0.1:4700/health make run
```

If `make run` in `/opt3/itcy` prints `Address already in use`, the product is already up - that is fine. See skill `itcy-dev-stack`.

## What you see

A **full-screen** bordered UI (not a single shell line):

- Header: **ITCy** · Interchouette ITC status
- `health: ok` (green) when product answers, or `DOWN` (red) with the error
- `providers` + `freeform` / `load` / `draft` routes from `GET /status`
- `webhook: ok` with `url:` (`/github/webhook_ITCy`) + `detail:` (secret ready / last wake)
- `delivery:` / `last:` / `warn:` from S4w (`last_github_delivery`, `github_delivery_warn`)
- Key `c`: toggle slash-command reference (`/ingest <external url>`, …)
- Footer: `polls: N` incrementing ~1/s
- Live process logs are **not** in the TUI: attach the product window (`screen -dRR itcy`)
- After quit, your normal shell prompt returns cleanly

If the pane looks like shell/docker/cargo junk mixed into the UI: the TUI hard-clears on start and exit. Under GNU screen it skips the alternate buffer (screen often ignores it) and wipes the main buffer instead. Prefer a dedicated screen window (`itcy-tui`). Optional in `~/.screenrc`: `altscreen on`.

## Status

S0t + S4h + S4w: ratatui pane + live `/health` + `/status` (providers / routes / webhook wake + delivery health) + slash-command pane (`c`). Hard clear on enter (screen-safe).
