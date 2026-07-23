use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct DeviceRow {
    pub id: Uuid,
    pub user_id: Uuid,
    pub agent_id: String,
    pub name: String,
    pub last_seen: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

pub struct DeviceRepository<'a> {
    pool: &'a PgPool,
}

impl<'a> DeviceRepository<'a> {
    pub fn new(pool: &'a PgPool) -> Self {
        Self { pool }
    }

    pub async fn upsert(
        &self,
        user_id: Uuid,
        agent_id: &str,
        name: &str,
    ) -> anyhow::Result<DeviceRow> {
        let row = sqlx::query_as::<_, DeviceRow>(
            r#"
            INSERT INTO devices (user_id, agent_id, name, last_seen)
            VALUES ($1, $2, $3, now())
            ON CONFLICT (user_id, agent_id)
            DO UPDATE SET name = EXCLUDED.name, last_seen = now()
            RETURNING id, user_id, agent_id, name, last_seen, created_at
            "#,
        )
        .bind(user_id)
        .bind(agent_id)
        .bind(name)
        .fetch_one(self.pool)
        .await?;
        Ok(row)
    }

    pub async fn touch(&self, user_id: Uuid, agent_id: &str) -> anyhow::Result<()> {
        sqlx::query(
            r#"
            UPDATE devices
            SET last_seen = now()
            WHERE user_id = $1 AND agent_id = $2
            "#,
        )
        .bind(user_id)
        .bind(agent_id)
        .execute(self.pool)
        .await?;
        Ok(())
    }

    pub async fn list_for_user(&self, user_id: Uuid) -> anyhow::Result<Vec<DeviceRow>> {
        let rows = sqlx::query_as::<_, DeviceRow>(
            r#"
            SELECT id, user_id, agent_id, name, last_seen, created_at
            FROM devices
            WHERE user_id = $1
            ORDER BY last_seen DESC
            "#,
        )
        .bind(user_id)
        .fetch_all(self.pool)
        .await?;
        Ok(rows)
    }

    pub async fn get_by_id(
        &self,
        user_id: Uuid,
        device_id: Uuid,
    ) -> anyhow::Result<Option<DeviceRow>> {
        let row = sqlx::query_as::<_, DeviceRow>(
            r#"
            SELECT id, user_id, agent_id, name, last_seen, created_at
            FROM devices
            WHERE id = $1 AND user_id = $2
            "#,
        )
        .bind(device_id)
        .bind(user_id)
        .fetch_optional(self.pool)
        .await?;
        Ok(row)
    }

    pub async fn latest_for_user(&self, user_id: Uuid) -> anyhow::Result<Option<DeviceRow>> {
        let row = sqlx::query_as::<_, DeviceRow>(
            r#"
            SELECT id, user_id, agent_id, name, last_seen, created_at
            FROM devices
            WHERE user_id = $1
            ORDER BY last_seen DESC
            LIMIT 1
            "#,
        )
        .bind(user_id)
        .fetch_optional(self.pool)
        .await?;
        Ok(row)
    }
}
