use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct ApiKeyRow {
    pub id: Uuid,
    pub user_id: Uuid,
    pub name: String,
    pub key_prefix: String,
    pub key_hash: String,
    pub created_at: DateTime<Utc>,
    pub last_used_at: Option<DateTime<Utc>>,
    pub revoked_at: Option<DateTime<Utc>>,
}

pub struct ApiKeyRepository<'a> {
    pool: &'a PgPool,
}

impl<'a> ApiKeyRepository<'a> {
    pub fn new(pool: &'a PgPool) -> Self {
        Self { pool }
    }

    pub async fn insert(
        &self,
        user_id: Uuid,
        name: &str,
        key_prefix: &str,
        key_hash: &str,
    ) -> anyhow::Result<ApiKeyRow> {
        let row = sqlx::query_as::<_, ApiKeyRow>(
            r#"
            INSERT INTO api_keys (user_id, name, key_prefix, key_hash)
            VALUES ($1, $2, $3, $4)
            RETURNING id, user_id, name, key_prefix, key_hash, created_at, last_used_at, revoked_at
            "#,
        )
        .bind(user_id)
        .bind(name)
        .bind(key_prefix)
        .bind(key_hash)
        .fetch_one(self.pool)
        .await?;
        Ok(row)
    }

    pub async fn find_by_hash(&self, key_hash: &str) -> anyhow::Result<Option<ApiKeyRow>> {
        let row = sqlx::query_as::<_, ApiKeyRow>(
            r#"
            SELECT id, user_id, name, key_prefix, key_hash, created_at, last_used_at, revoked_at
            FROM api_keys
            WHERE key_hash = $1 AND revoked_at IS NULL
            "#,
        )
        .bind(key_hash)
        .fetch_optional(self.pool)
        .await?;
        Ok(row)
    }

    pub async fn list_for_user(&self, user_id: Uuid) -> anyhow::Result<Vec<ApiKeyRow>> {
        let rows = sqlx::query_as::<_, ApiKeyRow>(
            r#"
            SELECT id, user_id, name, key_prefix, key_hash, created_at, last_used_at, revoked_at
            FROM api_keys
            WHERE user_id = $1
            ORDER BY created_at DESC
            "#,
        )
        .bind(user_id)
        .fetch_all(self.pool)
        .await?;
        Ok(rows)
    }

    pub async fn revoke(&self, user_id: Uuid, key_id: Uuid) -> anyhow::Result<bool> {
        let result = sqlx::query(
            r#"
            UPDATE api_keys
            SET revoked_at = now()
            WHERE id = $1 AND user_id = $2 AND revoked_at IS NULL
            "#,
        )
        .bind(key_id)
        .bind(user_id)
        .execute(self.pool)
        .await?;
        Ok(result.rows_affected() > 0)
    }

    pub async fn touch(&self, key_id: Uuid) -> anyhow::Result<()> {
        sqlx::query("UPDATE api_keys SET last_used_at = now() WHERE id = $1")
            .bind(key_id)
            .execute(self.pool)
            .await?;
        Ok(())
    }
}
