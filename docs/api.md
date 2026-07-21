# NetChronicle API

Base URL: `http://127.0.0.1:8080`

All JSON responses use **camelCase** field names. Errors return:

```json
{ "error": "message" }
```

## Common query parameters

| Param | Example | Description |
|-------|---------|-------------|
| `user_id` | UUID | Target user (defaults to local user) |
| `date` | `2026-06-15` | Single calendar day |
| `from` | RFC3339 timestamp | Range start |
| `to` | RFC3339 timestamp | Range end |
| `limit` | `100` | Max rows (1–1000) |
| `offset` | `0` | Pagination offset |

If `date` is set, it overrides `from`/`to` for that day.

---

## Health

### `GET /health`

```json
{
  "status": "ok",
  "database": "ok"
}
```

---

## Sessions

### `GET /sessions`

Returns built sessions from the `sessions` table (populated by the agent's session builder).

Query: `date`, `from`, `to`, `limit`, `offset`

```json
{
  "sessions": [
    {
      "sessionId": "uuid",
      "startTime": "2026-06-15T09:00:00Z",
      "endTime": "2026-06-15T10:30:00Z",
      "category": "work",
      "productivityScore": 85.0,
      "primaryApps": ["Code"],
      "networkStability": "stable",
      "websites": [
        {
          "domain": "github.com",
          "url": "https://github.com/user/repo",
          "timeSpentSec": 900,
          "category": "work"
        }
      ]
    }
  ],
  "limit": 100,
  "offset": 0
}
```

Timeline and session entries include `sessionId` when logs are linked after rebuild.

---

## Timeline

### `GET /timeline`

Merged app and website activity for a day.

```json
{
  "date": "2026-06-15",
  "entries": [
    {
      "time": "2026-06-15T09:00:00Z",
      "label": "github.com",
      "category": "work",
      "source": "website",
      "durationSec": 120,
      "sessionId": "uuid"
    }
  ]
}
```

---

## Reports

### `GET /daily-report`

```json
{
  "date": "2026-06-15",
  "productivityScore": 72.5,
  "totalOnlineMinutes": 240,
  "networkHealthScore": 95.0,
  "distractionRatio": 0.12,
  "focusMinutes": 180,
  "cached": false
}
```

### `GET /weekly-report`

```json
{
  "weekStart": "2026-06-09",
  "weekEnd": "2026-06-15",
  "summary": {
    "totalOnlineMinutes": 1200,
    "productiveMinutes": 900,
    "sessionCount": 42,
    "averageProductivityScore": 75.0,
    "categoryMinutes": [],
    "topApps": [],
    "topDomains": []
  },
  "cached": true
}
```

---

## Live & network

### `GET /network-stats`

Network samples and aggregations for the requested time range.

Query: `?date=YYYY-MM-DD` or `?from=&to=` plus optional `limit`.

```json
{
  "samples": [
    {
      "recordedAt": "2026-07-20T10:00:00Z",
      "latencyMs": 24.5,
      "packetLossPct": 0.0,
      "bandwidthMbps": null,
      "stability": "stable",
      "disconnect": false
    }
  ],
  "aggregation": {
    "sampleCount": 120,
    "avgLatencyMs": 28.4,
    "p95LatencyMs": 72.0,
    "avgPacketLossPct": 1.2,
    "avgBandwidthMbps": null,
    "disconnectCount": 2
  },
  "stabilityScore": 94.5
}
```

### `GET /network-events`

Disconnects and spike windows (latency ≥ 200ms or loss ≥ 15%).

```json
{
  "events": [
    {
      "recordedAt": "2026-07-20T10:15:00Z",
      "kind": "disconnect",
      "latencyMs": null,
      "packetLossPct": 100.0,
      "bandwidthMbps": null,
      "stability": "offline",
      "disconnect": true
    }
  ]
}
```

### `GET /live-status`

Current activity from latest agent snapshot (within 60 seconds).

---

## Insights

### `GET /insights`

Rule-based insights from sessions and usage patterns.

---

## Category rules

### `GET /category-rules`

List user-defined classification rules.

### `POST /category-rules`

```json
{
  "pattern": "notion.so",
  "patternType": "domain",
  "category": "work",
  "priority": 10
}
```

### `PUT /category-rules/{id}`

Update an existing rule (same body as POST).

### `DELETE /category-rules/{id}`

Delete a rule by ID.

---

## Browser extension bridge

### `POST /browser-tab` (API)

Accepts exact URL from a browser extension (alternative to agent feed on port 9477).

```json
{
  "url": "https://github.com/user/repo",
  "title": "GitHub"
}
```

### `POST http://127.0.0.1:9477/browser-tab` (agent)

Same payload, sent directly to the agent's local feed server for lowest latency.

---

## Notes

- Run `netchronicle-agent` to collect data and rebuild sessions every 5 minutes (configurable).
- Sessions appear after the session builder runs; raw app logs are always stored immediately.
- Weekly/daily reports are cached in the `reports` table after first computation.
