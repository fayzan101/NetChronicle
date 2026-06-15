use sqlx::PgPool;
use uuid::Uuid;

pub struct UserRepository<'a> {
    pool: &'a PgPool,
}

impl<'a> UserRepository<'a> {
    pub fn new(pool: &'a PgPool) -> Self {
        Self { pool }
    }

    pub async fn ensure_local_user(&self) -> anyhow::Result<Uuid> {
        let existing = sqlx::query_scalar::<_, Uuid>(
            "SELECT id FROM users WHERE display_name = 'Local User' LIMIT 1",
        )
        .fetch_optional(self.pool)
        .await?;

        if let Some(id) = existing {
            return Ok(id);
        }

        let id = sqlx::query_scalar::<_, Uuid>(
            r#"
            INSERT INTO users (display_name, email)
            VALUES ('Local User', 'local@netchronicle.local')
            RETURNING id
            "#,
        )
        .fetch_one(self.pool)
        .await?;

        Ok(id)
    }

    pub async fn get_or_create(&self, user_id: Uuid) -> anyhow::Result<Uuid> {
        if user_id.is_nil() {
            return self.ensure_local_user().await;
        }

        let exists = sqlx::query_scalar::<_, bool>("SELECT EXISTS(SELECT 1 FROM users WHERE id = $1)")
            .bind(user_id)
            .fetch_one(self.pool)
            .await?;

        if exists {
            Ok(user_id)
        } else {
            self.ensure_local_user().await
        }
    }
}
