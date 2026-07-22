# Deployment

## Local (development)

```bash
# Postgres
docker compose up -d

# Terminal 1 — API
cargo run -p netchronicle-api

# Terminal 2 — Agent (foreground tracking)
cargo run -p netchronicle-agent

# Terminal 3 — Worker (sessions, reports, retention)
cargo run -p netchronicle-worker
```

One-shot worker (useful in cron / CI smoke):

```bash
WORKER_RUN_ONCE=true cargo run -p netchronicle-worker
```

## API + Neon (Fly.io / Railway)

1. Create a Neon database and set `DATABASE_URL=postgresql://...?sslmode=require`.
2. Deploy `netchronicle-api` as a long-running service:

```bash
# Example: Railway / Fly — set env
DATABASE_URL=...
API_HOST=0.0.0.0
API_PORT=8080
RUST_LOG=info
```

3. Health check: `GET /health` (checks DB).
4. Optional scrape: `GET /metrics` (Prometheus text).

Migrations run automatically on API (and worker) startup.

## Agent (Windows)

Run the agent on each tracked machine (it writes to the shared Postgres/Neon DB):

```powershell
$env:DATABASE_URL = "postgresql://USER:PASS@HOST/neondb?sslmode=require"
$env:AGENT_USER_ID = "<uuid>"   # optional after first run
cargo run -p netchronicle-agent --release
```

### Scheduled Task (always-on)

1. Build: `cargo build -p netchronicle-agent --release`
2. Create a Task Scheduler job that starts `target\release\netchronicle-agent.exe` at logon, with the same env vars (or a `.env` next to the binary).
3. Restart on failure: set the task to restart every 1 minute if it exits.

## Worker (reports + retention)

The worker rebuilds sessions, caches daily/weekly/monthly reports, and prunes old `raw_events`.

| Variable | Default | Description |
|----------|---------|-------------|
| `WORKER_SESSION_INTERVAL_SECS` | `300` | Session rebuild interval |
| `WORKER_REPORT_INTERVAL_SECS` | `900` | Report cache interval |
| `WORKER_RETENTION_INTERVAL_SECS` | `3600` | Prune interval |
| `SESSION_REBUILD_LOOKBACK_DAYS` | `2` | Days of sessions to rebuild |
| `WORKER_REPORT_LOOKBACK_DAYS` | `30` | Days of reports to recompute |
| `RAW_EVENTS_RETENTION_DAYS` | `30` | Delete raw events older than this |
| `WORKER_USER_ID` | all users | Optional single-user scope |
| `WORKER_RUN_ONCE` | `false` | Exit after one full job pass |

Run the worker beside the API (same Neon DB), or as a daily cron with `WORKER_RUN_ONCE=true`.

## Browser extension

See [extension/README.md](../extension/README.md) for Chrome/Edge install.
