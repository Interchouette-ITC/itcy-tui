# itcy-tui

Ratatui status UI for **ITCy** (Interchouette ITC AI experience).

| | |
| --- | --- |
| **Canonical** | [Interchouette-ITC/itcy-tui](https://github.com/Interchouette-ITC/itcy-tui) |
| **Worker fork** | [Interchouette/itcy-tui](https://github.com/Interchouette/itcy-tui) |
| **Local** | `/opt3/itcy-tui` |
| **Pairs with** | product [Interchouette-ITC/itcy](https://github.com/Interchouette-ITC/itcy) (`GET /health` on `:4700`) |

Public showcase of the early TUI stage. Develop on the **Interchouette** fork; open PRs into **Interchouette-ITC**.

## Run

```bash
# Product must already answer /health (do not start a second listener if :4700 is taken):
curl -s http://localhost:4700/health   # expect: ok
make test
make run
# Full-screen TUI. Quit: q / Esc / Ctrl-C. Refresh: r
# override URL: ITCY_HEALTH_URL=http://127.0.0.1:4700/health make run
```

If `make run` in `/opt3/itcy` prints `Address already in use`, the product is already up - that is fine. See skill `itcy-dev-stack`.

## What you see

A **full-screen** bordered UI (not a single shell line):

- Header: **ITCy** · Interchouette ITC status
- `health: ok` (green) when product answers, or `DOWN` (red) with the error
- `url:` of the probe
- Footer: `polls: N` incrementing ~1/s
- After quit, your normal shell prompt returns cleanly

## Status

S0t: ratatui pane + live `/health` wiring.
