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
# product must answer /health (from /opt3/itcy):
#   make run
make test
make run
# q quit · r refresh
# override URL: ITCY_HEALTH_URL=http://127.0.0.1:4700/health make run
```

## What you see

- Brand header **ITCy**
- Live `health: ok` / `DOWN` from the product probe
- Poll count footer

## Status

S0t: ratatui pane + live `/health` wiring.
