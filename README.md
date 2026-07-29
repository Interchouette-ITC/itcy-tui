# itcy-tui

Terminal status UI for **[ITCy](https://github.com/Interchouette-ITC/itcy)**, built with [ratatui](https://ratatui.rs/).

ITCy is Interchouette ITC’s LinkedIn operator (Slack runtime, drafts, publications BAT, corpus tools). This repo is only the status pane that probes the always-on binary.

| | |
| --- | --- |
| **Canonical** | [Interchouette-ITC/itcy-tui](https://github.com/Interchouette-ITC/itcy-tui) |
| **Worker fork** | [Interchouette/itcy-tui](https://github.com/Interchouette/itcy-tui) |
| **Product API** | `GET /health` and `GET /status` on the ITCy binary (default `http://127.0.0.1:4700`) |

## Requirements

- Rust toolchain (edition 2021)
- A running ITCy process that answers `GET /health` with body `ok`

## Run

```bash
curl -s http://127.0.0.1:4700/health   # expect: ok
make test
make run
```

```bash
ITCY_HEALTH_URL=http://127.0.0.1:4700/health make run
```

| Key | Action |
| --- | --- |
| `q` / Esc / Ctrl-C | Quit |
| `r` | Refresh |
| `c` | Slash-command reference |

## What you see

- Product health and LLM provider / route snapshot
- GitHub webhook path and delivery health
- Enrich queue progress (remaining work, next due, wall streak, drip process)
- Slack slash-command cheat sheet (`c`)

Live product logs stay with the ITCy process, not in this pane.

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md).
