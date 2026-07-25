# NetChronicle (Internet Diary)

Rust backend that tracks digital activity, measures network health, and exposes productivity data through a REST API.

## Architecture

```
Foreground apps/sites (agent)
  → PostgreSQL (Neon or local)
  → Session builder (background)
  → Analytics engine
  → Axum REST API
```

## Prerequisites

- Rust (stable)
- PostgreSQL — [Neon](https://neon.tech) or local Docker

## Setup

```bash
cp .env.example .env
# Set DATABASE_URL (Neon requires ?sslmode=require)
```

### Neon example

```
DATABASE_URL=postgresql://USER:PASSWORD@HOST/neondb?sslmode=require
```

### Local Postgres

```bash
docker compose up -d
```

## Run

```bash
# Terminal 1 — API (applies migrations on startup)
cargo run -p netchronicle-api

# Terminal 2 — tracking agent
cargo run -p netchronicle-agent

# Terminal 3 — worker (session rebuild, report cache, retention)
cargo run -p netchronicle-worker
```

API: `http://127.0.0.1:8080`

### Dashboard

```bash
cd apps/dashboard
npm start
```

Dashboard: `http://localhost:4200` (expects API at `http://localhost:8080` with `AUTH_REQUIRED=false` for local mode).

## Environment variables

| Variable | Default | Description |
|----------|---------|-------------|
| `DATABASE_URL` | local postgres | PostgreSQL connection string |
| `API_HOST` | `127.0.0.1` | API bind host |
| `API_PORT` | `8080` | API bind port |
| `AGENT_POLL_INTERVAL_SECS` | `2` | Foreground window poll interval |
| `AGENT_MIN_SEGMENT_SECS` | `3` | Min activity segment to persist |
| `AGENT_IGNORE_APPS` | — | Comma-separated apps/titles to skip |
| `NETWORK_SAMPLE_INTERVAL_SECS` | `30` | Network probe interval |
| `NETWORK_PROBE_HOST` | `8.8.8.8` | ICMP / connectivity probe host |
| `NETWORK_PROBE_TCP_PORT` | `53` | TCP fallback port when ping fails |
| `NETWORK_PING_COUNT` | `4` | Echo requests per sample |
| `NETWORK_BANDWIDTH_ENABLED` | `false` | Enable HTTP bandwidth estimate |
| `NETWORK_BANDWIDTH_URL` | Cloudflare speed URL | Download URL for bandwidth probe |
| `NETWORK_BANDWIDTH_BYTES` | `100000` | Max bytes to download per bandwidth sample |
| `SESSION_REBUILD_INTERVAL_SECS` | `300` | Session builder interval |
| `SESSION_REBUILD_LOOKBACK_DAYS` | `2` | Days to rebuild (today + yesterday) |
| `WORKER_REPORT_LOOKBACK_DAYS` | `30` | Days of reports the worker recomputes |
| `RAW_EVENTS_RETENTION_DAYS` | `30` | Prune raw_events older than this |
| `SESSION_IDLE_GAP_SECS` | `300` | Idle gap between sessions |
| `SESSION_MIN_DURATION_SECS` | `60` | Minimum session length |
| `AGENT_IDLE_THRESHOLD_SECS` | `300` | Pause tracking after this many idle seconds |
| `AGENT_BROWSER_FEED_PORT` | `9477` | Local HTTP port for browser extension tab feed |
| `RULES_REFRESH_INTERVAL_SECS` | `60` | How often the agent reloads category rules from DB |
| `SETTINGS_REFRESH_INTERVAL_SECS` | `30` | How often the agent reloads user settings |
| `AUTH_REQUIRED` | `false` | Require Bearer / API key on API (and agent key) |
| `AGENT_API_KEY` | — | Agent API key (`nck_…`) binds writes to a user |
| `AGENT_DEVICE_ID` | hostname | Stable device id for this agent |
| `AGENT_DEVICE_NAME` | `Local Agent` | Friendly device name |
| `DEFAULT_USER_ID` | — | API default user UUID (local mode only) |

## API overview

See [docs/api.md](docs/api.md) for full endpoint documentation.

| Endpoint | Description |
|----------|-------------|
| `GET /health` | Health + DB check |
| `POST /auth/register` | Create user + session token |
| `POST /auth/login` | Login + session token |
| `GET/POST /auth/api-keys` | List / create agent API keys |
| `DELETE /auth/api-keys/{id}` | Revoke API key |
| `GET/PATCH /settings` | Tracking + privacy settings |
| `GET/POST /devices` | List / register devices |
| `POST /devices/heartbeat` | Touch device last_seen |
| `POST /export` | Export user activity (JSON/CSV) |
| `POST /data/delete-token` | Confirmation token for wipe |
| `DELETE /data` | Wipe user activity |
| `GET /sessions` | Built sessions |
| `GET /timeline` | Merged app + website timeline |
| `GET /daily-report` | Daily productivity summary |
| `GET /weekly-report` | Weekly summary (cached) |
| `GET /live-status` | Current activity snapshot (`?deviceId=`) |
| `GET /network-stats` | Network samples + avg/p95 aggregations |
| `GET /network-events` | Disconnects and latency/loss spikes |
| `GET /reports/daily\|weekly\|monthly` | Cached period reports |
| `GET /reports/export` | Export report as JSON or CSV |
| `GET /metrics` | Prometheus-style gauges |
| `GET /insights` | Rule-based insights |
| `GET/POST/PUT/DELETE /category-rules` | Category rule CRUD |
| `POST /browser-tab` | Report active browser tab URL (extension fallback) |

Auth: send `Authorization: Bearer <token>` or `X-Api-Key: nck_…`. With `AUTH_REQUIRED=false` (default), local single-user mode still works without a token.

## Browser extension

Install the unpacked Chrome/Edge extension from [`extension/`](extension/) so the agent receives exact tab URLs (see [extension/README.md](extension/README.md)).

**Linux idle detection:** install `xprintidle` (`sudo apt install xprintidle`) so AFK pauses tracking. macOS uses `ioreg`; Windows uses `GetLastInputInfo`.

## Crates

| Crate | Role |
|-------|------|
| `agent` | Foreground tracking + network sampling |
| `api` | REST API |
| `worker` | Session rebuild, report cache, retention |
| `db` | PostgreSQL repositories |
| `session-builder` | Groups logs into sessions |
| `analytics` | Scores and insights |
| `categorization` | Activity labeling |
| `network-monitor` | Network probes |
| `common` | Shared types |

## Docs

| Doc | Description |
|-----|-------------|
| [Implementation plan](docs/implementation-plan.md) | Phase-wise remaining work |
| [API](docs/api.md) | REST endpoint reference |
| [Architecture](docs/architecture.md) | System overview |
| [Deployment](docs/deployment.md) | API, agent, worker, Neon / Fly / Railway |

## Tests

```bash
cargo test --workspace
```
