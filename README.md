# itcy-tui

Ratatui status UI for **ITCy** (Interchouette ITC AI experience).

ITCy is the Interchouette ITC operator product. It is **not** ICA.

| | |
| --- | --- |
| **Canonical** | [Interchouette-ITC/itcy-tui](https://github.com/Interchouette-ITC/itcy-tui) |
| **Worker fork** | [Interchouette/itcy-tui](https://github.com/Interchouette/itcy-tui) |
| **Pairs with** | [Interchouette-ITC/itcy](https://github.com/Interchouette-ITC/itcy) (`GET /health` and `GET /status`, default `127.0.0.1:4700`) |

## Requirements

- Rust toolchain (edition 2021)
- A running ITCy binary that answers `GET /health` with body `ok`

## Run

```bash
curl -s http://127.0.0.1:4700/health   # expect: ok
make test
make run
```

Override the health URL if needed:

```bash
ITCY_HEALTH_URL=http://127.0.0.1:4700/health make run
```

Keys: `q` / Esc / Ctrl-C quit · `r` refresh · `c` slash-command reference.

## What you see

Full-screen bordered UI (not a single shell line):

- Header: **ITCy** · Interchouette ITC status
- `health:` product liveness
- `providers` and `freeform` / `load` / `draft` routes from `GET /status`
- `webhook:` path `POST /github/webhook_ITCy` plus last wake detail
- `delivery:` / `last:` / `warn:` GitHub delivery health from `/status`
- `enrich:` / `counts:` / `queue:` / `wall:` Tor enrich queue remaining effort, next due, wall streak, drip pid
- Key `c`: Slack slash-command reference (`/ingest`, `/enrich`, …)
- Footer: `polls: N` (~1/s)

Process logs are not in this TUI; use the product process or screen session that runs ITCy.

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md). Work on the Interchouette fork; open PRs into Interchouette-ITC `dev`.
