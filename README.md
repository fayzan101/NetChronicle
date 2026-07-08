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
```

API: `http://127.0.0.1:8080`

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
| `SESSION_REBUILD_INTERVAL_SECS` | `300` | Session builder interval |
| `SESSION_IDLE_GAP_SECS` | `300` | Idle gap between sessions |
| `SESSION_MIN_DURATION_SECS` | `60` | Minimum session length |
| `DEFAULT_USER_ID` | — | API default user UUID |

## API overview

See [docs/api.md](docs/api.md) for full endpoint documentation.

| Endpoint | Description |
|----------|-------------|
| `GET /health` | Health + DB check |
| `GET /sessions` | Built sessions |
| `GET /timeline` | Merged app + website timeline |
| `GET /daily-report` | Daily productivity summary |
| `GET /weekly-report` | Weekly summary (cached) |
| `GET /live-status` | Current activity snapshot |
| `GET /network-stats` | Network samples |
| `GET /insights` | Rule-based insights |
| `GET/POST/DELETE /category-rules` | Category rule CRUD |

Common query params: `?date=YYYY-MM-DD`, `?from=`, `?to=`, `?limit=`, `?offset=`, `?user_id=`

## Crates

| Crate | Role |
|-------|------|
| `agent` | Foreground tracking + network sampling |
| `api` | REST API |
| `db` | PostgreSQL repositories |
| `session-builder` | Groups logs into sessions |
| `analytics` | Scores and insights |
| `categorization` | Activity labeling |
| `network-monitor` | Network probes |
| `common` | Shared types |

## Tests

```bash
cargo test --workspace
```
