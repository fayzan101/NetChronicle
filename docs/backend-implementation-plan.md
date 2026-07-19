# NetChronicle — Backend Implementation Plan (archived)

> **Superseded.** Use [`docs/implementation-plan.md`](./implementation-plan.md) for the current phase-wise plan.
>
> This file is kept for historical Phase 0–3 task detail. Baseline tables below are **stale** (written mid-sync); Phases 1–3 are largely complete on `main` as of July 2026.

**Last reviewed:** July 2026 (post-sync) — status outdated; see new plan.

---

## Current baseline (Phase 0 — historical snapshot)

> Snapshot from early sync — **do not treat as current status.**

| Component | Status | Notes |
|-----------|--------|-------|
| `crates/common` | Done | Shared types: events, sessions, categories |
| `migrations/001_initial_schema.sql` | Done | Full PostgreSQL schema |
| `crates/db` | ~80% | Pool, migrations, repositories (user, activity, network, analytics) |
| `crates/agent` | ~65% | Foreground window tracking, browser domain heuristics, DB writes, live snapshots |
| `crates/network-monitor` | ~40% | TCP connect probe only (`TcpProbe`) |
| `crates/categorization` | ~50% | In-memory default rules; domain + app classification |
| `crates/api` | ~75% | All planned GET routes wired to PostgreSQL |
| `crates/session-builder` | ~5% | Stub — returns empty |
| `crates/analytics` | ~10% | Stub — logic lives in API/DB queries instead |
| Tests / CI | 0% | None |
| Auth | 0% | Single local user auto-created |
| Frontend | 0% | Intentionally deferred |

**Overall backend score at snapshot: 5.5 / 10** — superseded by shipped Phase 1–3 work.

### Working data flow today

```
Foreground window (x-win)
  → ActivityTracker (agent)
  → app_activity_logs / website_logs / raw_events
  → PostgreSQL (Neon or local)
  → Axum API (/sessions, /daily-report, …)
```

---

## Phase 1 — Stabilize the core pipeline

**Goal:** Make agent + API reliable for daily use on one machine.

**Estimated effort:** 1 week

### Tasks

1. **Fix & harden agent**
   - Normalize formatting / error handling in `collector.rs`
   - Skip empty or system windows (e.g. `Program Manager`, lock screen)
   - Configurable ignore list via env or user settings JSON
   - Graceful reconnect if Neon/Postgres drops mid-run

2. **Repository completeness**
   - Add `SessionRepository` (read/write `sessions` table — even if empty for now)
   - Add `CategoryRuleRepository` (CRUD on `category_rules`)
   - Add `ReportRepository` (read/write `reports` table)
   - Pagination on list endpoints (`limit`, `offset`, `from`, `to` query params)

3. **API improvements**
   - Consistent JSON field naming (`camelCase` everywhere)
   - Query params: `?date=`, `?from=`, `?to=` on report endpoints
   - Structured error responses (`{ "error": "..." }`)
   - OpenAPI spec or static `docs/api.md`

4. **Documentation**
   - Restore `README.md` (empty after sync)
   - Document env vars, run order, Neon setup

### Exit criteria

- Agent runs 8+ hours without crash
- All API routes return real DB data with date filters
- README documents local + Neon setup

---

## Phase 2 — Session builder & analytics engine

**Goal:** Turn raw logs into meaningful sessions and move scoring logic out of the API.

**Estimated effort:** 1–2 weeks

### Tasks

1. **Implement `session-builder`**
   - Idle-gap grouping (default 5 min) from `app_activity_logs`
   - Detect primary apps per session
   - Overlay network stability from `network_logs` in session window
   - Persist to `sessions` table with `productivity_score` placeholder

2. **Background session job**
   - Option A: Agent runs session builder every N minutes after flush
   - Option B: Separate `netchronicle-worker` binary on a timer
   - Mark processed logs to avoid double-counting (metadata flag or `session_id` FK)

3. **Implement `analytics` crate**
   - `daily_summary(user_id, date)` from sessions + logs
   - `weekly_summary`, `monthly_summary`
   - `generate_insights()` — time-of-day patterns, distraction impact, network correlation
   - Refactor API routes to call analytics crate instead of inline SQL

4. **Update API**
   - `/sessions` reads from `sessions` table (not raw app logs)
   - Add `GET /timeline?date=` — merged app + website timeline
   - Store computed daily/weekly summaries in `reports` table (cache)

### Exit criteria

- `sessions` table populated automatically
- `/daily-report` and `/insights` use `analytics` crate
- Weekly report cached in `reports`

---

## Phase 3 — Smarter tracking & categorization

**Goal:** Improve accuracy of what is being tracked and how it is labeled.

**Estimated effort:** 2 weeks

### Tasks

1. **Category rules from database**
   - Agent loads `category_rules` for user on startup (cache + refresh interval)
   - API: `GET/POST/PUT/DELETE /category-rules`
   - Priority-based matching (longest pattern wins)

2. **Browser URL accuracy**
   - Phase 3a: Improve title parsing (Chrome, Edge, Firefox title formats)
   - Phase 3b: Lightweight browser extension or native messaging bridge
     - Extension sends `{ url, tab_id, active }` to local agent via HTTP/WebSocket
   - Store exact URL in `website_logs.url`

3. **Idle / AFK detection**
   - Pause tracking when no input for X minutes (Windows API or `GetLastInputInfo`)
   - Do not count idle time toward online minutes

4. **App metadata**
   - Map process path → friendly name
   - Optional icon hash / process category in `raw_events`

### Exit criteria

- User can add custom rules via API; agent applies them within 1 refresh cycle
- Browser domains accurate for top 3 browsers without manual title hacks
- Idle time excluded from productivity metrics

---

## Phase 4 — Network monitoring (real metrics)

**Goal:** Deliver the “network + productivity” differentiator from the product plan.

**Estimated effort:** 1–2 weeks

### Tasks

1. **Replace TCP probe with real measurements**
   - ICMP ping (platform-specific; admin may be required on Windows)
   - Packet loss over N pings
   - Optional: lightweight HTTP download for bandwidth estimate

2. **Disconnect detection**
   - Detect adapter down / no route
   - Write `disconnect = true` events with timestamps

3. **Network–session correlation**
   - Tag sessions as `stable | degraded | unstable | offline` during their time window
   - Insights: “Network instability during your 2–3 PM coding session”

4. **API**
   - `GET /network-stats?from=&to=` with aggregation (avg, p95 latency)
   - `GET /network-events` for disconnect spikes

### Exit criteria

- Latency and packet loss stored every sample interval
- Sessions have `network_stability` populated
- Insights reference network quality

---

## Phase 5 — Auth, multi-user & settings

**Goal:** Support more than one local user; prepare for remote deployment.

**Estimated effort:** 2 weeks

### Tasks

1. **User model**
   - Registration / login (email + password or magic link — keep simple)
   - JWT or session tokens for API
   - Agent authenticates with API key per user/device

2. **Settings API**
   - `GET/PATCH /settings` — tracking on/off, poll intervals, privacy flags
   - Persist in `users.settings` JSONB

3. **Device / agent registration**
   - `devices` table (optional migration): agent_id, user_id, last_seen
   - `GET /live-status` scoped to active device

4. **Privacy**
   - `POST /export` — JSON/CSV dump for user
   - `DELETE /data` — wipe user activity with confirmation

### Exit criteria

- Multiple users on one Neon instance without data leakage
- Agent requires valid `AGENT_API_KEY` or user token
- Settings control tracking behavior without restart where possible

---

## Phase 6 — Reports, workers & production hardening

**Goal:** Backend ready for always-on use and future frontend hookup.

**Estimated effort:** 2–3 weeks

### Tasks

1. **`netchronicle-worker` binary**
   - Nightly: rebuild sessions, compute daily + weekly reports
   - Prune old `raw_events` (retention policy)
   - Refresh materialized summaries

2. **Reports API**
   - `GET /reports/daily|weekly|monthly`
   - `GET /reports/export?format=csv|json`
   - PDF generation deferred unless needed

3. **Testing**
   - Unit tests: categorization, session-builder, analytics scoring
   - Integration tests: API + test Postgres (Docker in CI)
   - Agent smoke test with mocked window provider

4. **Observability**
   - Structured logging (already via `tracing`)
   - Health: `GET /health` + DB connectivity check
   - Metrics endpoint (optional Prometheus)

5. **CI/CD**
   - GitHub Actions: `cargo test`, `cargo clippy`, fmt
   - Release builds for Windows agent + API

### Exit criteria

- CI green on every PR
- Reports generated without manual intervention
- Documented deployment for API on Fly/Railway + Neon

---

## Phase 7 — Optional backend extensions (post-MVP)

Not required for first usable product; listed for roadmap clarity.

| Feature | Description |
|---------|-------------|
| DNS-based tracking | Passive domain capture without browser extension |
| ML categorization | Learn categories from user corrections |
| WebSocket live stream | Push live status instead of poll/snapshot |
| Tauri/system tray | Rust desktop shell wrapping agent (still no web UI) |
| Team / org accounts | Aggregate stats across users |

---

## Suggested build order (summary)

```
Phase 0  ✅  Baseline (current)
Phase 1  →   Stabilize agent + API + docs
Phase 2  →   Sessions + analytics engine
Phase 3  →   Tracking accuracy + DB rules
Phase 4  →   Real network metrics
Phase 5  →   Auth + settings + privacy
Phase 6  →   Workers + tests + CI + reports
Phase 7  ○   Optional advanced features
```

**Recommended next step:** See [`implementation-plan.md`](./implementation-plan.md) — start at **Phase 4 (real network metrics)**.

---

## Crate ownership map (target end state)

| Crate | Phases |
|-------|--------|
| `agent` | 1, 3, 4, 5 |
| `db` | 1, 2, 5, 6 |
| `session-builder` | 2 |
| `analytics` | 2, 4, 6 |
| `categorization` | 3 |
| `network-monitor` | 4 |
| `api` | 1, 2, 3, 4, 5, 6 |
| `common` | All (shared types evolve as needed) |
| `worker` (new) | 6 |

---

## Out of scope (explicitly deferred)

- Next.js / any frontend dashboard
- PDF report UI
- Mobile apps
- Billing / subscriptions

The API is the contract for frontend work later; no UI work until Phase 6 backend is stable.
