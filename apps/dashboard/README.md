# NetChronicle dashboard

Angular (v19) UI for NetChronicle — Phase 8 complete: Today, Timeline, Live, Network, Analytics, Insights, Reports, and Settings (auth + privacy).

## Run

```bash
# API must be up (AUTH_REQUIRED=false for local mode)
cargo run -p netchronicle-api

cd apps/dashboard
npm start
```

Open `http://localhost:4200`.

API base URL is set in `src/environments/environment.development.ts` (`apiUrl`).
