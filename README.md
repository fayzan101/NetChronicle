# NetChronicle 

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

