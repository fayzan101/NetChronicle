use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct AuthTokenRow {
    pub id: Uuid,
    pub user_id: Uuid,
    pub token_hash: String,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

pub struct AuthTokenRepository<'a> {
    pool: &'a PgPool,
}

impl<'a> AuthTokenRepository<'a> {
    pub fn new(pool: &'a PgPool) -> Self {
        Self { pool }
    }

    pub async fn insert(
        &self,
        user_id: Uuid,
        token_hash: &str,
        expires_at: DateTime<Utc>,
    ) -> anyhow::Result<AuthTokenRow> {
        let row = sqlx::query_as::<_, AuthTokenRow>(
            r#"
            INSERT INTO auth_tokens (user_id, token_hash, expires_at)
            VALUES ($1, $2, $3)
            RETURNING id, user_id, token_hash, expires_at, created_at
            "#,
        )
        .bind(user_id)
        .bind(token_hash)
        .bind(expires_at)
        .fetch_one(self.pool)
        .await?;
        Ok(row)
    }

    pub async fn find_valid(&self, token_hash: &str) -> anyhow::Result<Option<AuthTokenRow>> {
        let row = sqlx::query_as::<_, AuthTokenRow>(
            r#"
            SELECT id, user_id, token_hash, expires_at, created_at
            FROM auth_tokens
            WHERE token_hash = $1 AND expires_at > now()
            "#,
        )
        .bind(token_hash)
        .fetch_optional(self.pool)
        .await?;
        Ok(row)
    }

    pub async fn revoke_all_for_user(&self, user_id: Uuid) -> anyhow::Result<u64> {
        let result = sqlx::query("DELETE FROM auth_tokens WHERE user_id = $1")
            .bind(user_id)
            .execute(self.pool)
            .await?;
        Ok(result.rows_affected())
    }
}
