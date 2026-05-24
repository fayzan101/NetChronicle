-- NetChronicle initial schema (PostgreSQL)

CREATE EXTENSION IF NOT EXISTS "pgcrypto";

CREATE TYPE activity_category AS ENUM (
    'work',
    'learning',
    'entertainment',
    'distraction',
    'neutral',
    'unknown'
);

CREATE TYPE network_stability AS ENUM (
    'stable',
    'degraded',
    'unstable',
    'offline'
);

CREATE TABLE users (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email           TEXT UNIQUE,
    display_name    TEXT NOT NULL,
    settings        JSONB NOT NULL DEFAULT '{}',
    tracking_enabled BOOLEAN NOT NULL DEFAULT true,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE category_rules (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id         UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    pattern         TEXT NOT NULL,
    pattern_type    TEXT NOT NULL DEFAULT 'domain',
    category        activity_category NOT NULL,
    priority        INT NOT NULL DEFAULT 0,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE sessions (
    session_id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id             UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    start_time          TIMESTAMPTZ NOT NULL,
    end_time            TIMESTAMPTZ,
    category            activity_category NOT NULL DEFAULT 'unknown',
    productivity_score  REAL,
    network_stability   network_stability,
    primary_apps        TEXT[] NOT NULL DEFAULT '{}',
    metadata            JSONB NOT NULL DEFAULT '{}',
    created_at          TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_sessions_user_start ON sessions (user_id, start_time DESC);

CREATE TABLE website_logs (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id         UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    session_id      UUID REFERENCES sessions(session_id) ON DELETE SET NULL,
    url             TEXT NOT NULL,
    domain          TEXT NOT NULL,
    time_spent_sec  INT NOT NULL DEFAULT 0,
    category        activity_category NOT NULL DEFAULT 'unknown',
    visited_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_website_logs_user_visited ON website_logs (user_id, visited_at DESC);
CREATE INDEX idx_website_logs_domain ON website_logs (user_id, domain);

CREATE TABLE app_activity_logs (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id         UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    session_id      UUID REFERENCES sessions(session_id) ON DELETE SET NULL,
    app_name        TEXT NOT NULL,
    window_title    TEXT,
    duration_sec    INT NOT NULL DEFAULT 0,
    category        activity_category NOT NULL DEFAULT 'unknown',
    recorded_at     TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_app_logs_user_recorded ON app_activity_logs (user_id, recorded_at DESC);

CREATE TABLE network_logs (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id         UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    latency_ms      REAL,
    packet_loss_pct REAL,
    bandwidth_mbps  REAL,
    stability       network_stability,
    disconnect      BOOLEAN NOT NULL DEFAULT false,
    recorded_at     TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_network_logs_user_recorded ON network_logs (user_id, recorded_at DESC);

CREATE TABLE reports (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id         UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    report_type     TEXT NOT NULL,
    period_start    DATE NOT NULL,
    period_end      DATE NOT NULL,
    summary         JSONB NOT NULL DEFAULT '{}',
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (user_id, report_type, period_start, period_end)
);

CREATE INDEX idx_reports_user_type ON reports (user_id, report_type, period_start DESC);

CREATE TABLE raw_events (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id         UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    event_type      TEXT NOT NULL,
    payload         JSONB NOT NULL,
    recorded_at     TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_raw_events_user_recorded ON raw_events (user_id, recorded_at DESC);

CREATE OR REPLACE FUNCTION set_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = now();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER users_updated_at
    BEFORE UPDATE ON users
    FOR EACH ROW
    EXECUTE FUNCTION set_updated_at();
