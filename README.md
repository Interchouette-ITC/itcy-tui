# itcy-tui

Status terminal for **ITCy**, Interchouette ITC's AI LinkedIn operator.

Built with [ratatui](https://ratatui.rs/). Pairs with the [ITCy](https://github.com/Interchouette-ITC/itcy) binary (`GET /health`, `GET /status`).

## Run

```bash
# ITCy must already be up:
curl -s http://127.0.0.1:4700/health   # expect: ok

make test
make run
```

Optional: `ITCY_HEALTH_URL=http://127.0.0.1:4700/health make run`

| Key | Action |
| --- | --- |
| `q` / Esc / Ctrl-C | Quit |
| `r` | Refresh |
| `c` | Command reference |

## What you see

Health, model routes, GitHub webhook delivery, enrich-queue progress, and a short Slack command reference. Product logs stay with the ITCy process.

## License

BUSL-1.1 (Interchouette-ITC). See [LICENSE](LICENSE).
