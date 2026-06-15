# NetChronicle (Internet Diary)

Background system that tracks digital activity, measures network health, and surfaces productivity intelligence through a real-time dashboard.

## Architecture

```
[Browser / Apps / OS Network]
            ↓
   Rust Agent (crates/agent)
            ↓
   Processing (session-builder, categorization, analytics, network-monitor)
            ↓
   PostgreSQL (migrations/)
            ↓
   REST API (crates/api — Axum)
            ↓
   Next.js Dashboard (apps/dashboard)
```

## Repository layout

| Path | Purpose |
|------|---------|
| `crates/agent` | Data collection (apps, sites, sessions) |
| `crates/network-monitor` | Latency, packet loss, bandwidth |
| `crates/session-builder` | Raw events → sessions |
| `crates/categorization` | Work / learning / distraction labels |
| `crates/analytics` | Scores, focus time, insights |
| `crates/db` | PostgreSQL access (SQLx) |
| `crates/common` | Shared types and utilities |
| `crates/api` | HTTP API for the dashboard |
| `apps/dashboard` | Next.js web UI |
| `migrations/` | PostgreSQL schema (SQLx) |

## Prerequisites

- [Rust](https://rustup.rs/) (stable)
- [Node.js](https://nodejs.org/) 20+
- [Docker](https://www.docker.com/) (for PostgreSQL)

## Quick start

```bash
cp .env.example .env
# Edit .env — for Neon, set DATABASE_URL with ?sslmode=require

# API (runs migrations on startup)
cargo run -p netchronicle-api

# Agent (tracks foreground app/window + network; writes to PostgreSQL)
cargo run -p netchronicle-agent
```

### Neon PostgreSQL

Neon requires SSL. Use a connection string like:

```
DATABASE_URL=postgresql://USER:PASSWORD@HOST/neondb?sslmode=require
```

Do **not** commit `.env` — it is gitignored.

### Local PostgreSQL (optional)

```bash
docker compose up -d
```

Open [http://localhost:3000](http://localhost:3000) for the dashboard (when added). API defaults to `http://localhost:8080`.

## API endpoints (planned)

| Endpoint | Description |
|----------|-------------|
| `GET /sessions` | List sessions |
| `GET /daily-report` | Daily summary |
| `GET /weekly-report` | Weekly analytics |
| `GET /live-status` | Real-time tracking |
| `GET /network-stats` | Network health |
| `GET /insights` | Productivity suggestions |

## License

Proprietary — all rights reserved unless otherwise noted.
