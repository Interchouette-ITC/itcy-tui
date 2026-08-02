# itcy-tui

Status terminal for **ITCy**, Interchouette ITC's AI LinkedIn operator.

Built with [ratatui](https://ratatui.rs/). Pairs with the [ITCy](https://github.com/Interchouette-ITC/itcy) binary on `:4700` (`GET /health`, `GET /status`) and probes org ingress [itc-hooks](https://github.com/Interchouette-ITC/itc-hooks) on `:7007`.

## Run

```bash
# ITCy must already be up:
curl -s http://127.0.0.1:4700/health   # expect: ok
# Optional: ingress live for webhook hydration:
curl -s http://127.0.0.1:7007/health   # expect: ok

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

ITCy health, ingress open/down, model routes, GitHub hooks delivery, enrich-queue progress, and a short Slack command reference. Product logs stay with the ITCy process. Public org webhook path is on itc-hooks (`POST /github/webhook_ITC`); ITCy receives `POST /hooks/github` only.

## License

BUSL-1.1 (Interchouette-ITC). See [LICENSE](LICENSE).
