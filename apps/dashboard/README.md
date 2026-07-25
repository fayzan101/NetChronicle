# NetChronicle dashboard

Angular (v19) UI for NetChronicle — Phase 8: **Today**, **Timeline**, **Live**, **Network**, **Analytics**, **Insights**, and **Reports** (Settings next).

## Run

```bash
# API must be up (AUTH_REQUIRED=false for local mode)
cargo run -p netchronicle-api

cd apps/dashboard
npm start
```

Open `http://localhost:4200`.

API base URL is set in `src/environments/environment.development.ts` (`apiUrl`).
