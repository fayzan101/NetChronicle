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
      "networkStability": "stable"
    }
  ],
  "limit": 100,
  "offset": 0
}
```

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
      "durationSec": 120
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

### `GET /live-status`

Current activity from latest agent snapshot (within 60 seconds).

### `GET /network-stats`

Network samples for the requested time range.

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

### `DELETE /category-rules/{id}`

Delete a rule by ID.

---

## Notes

- Run `netchronicle-agent` to collect data and rebuild sessions every 5 minutes (configurable).
- Sessions appear after the session builder runs; raw app logs are always stored immediately.
- Weekly/daily reports are cached in the `reports` table after first computation.
