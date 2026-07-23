use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct UserRow {
    pub id: Uuid,
    pub email: Option<String>,
    pub display_name: String,
    pub settings: Value,
    pub tracking_enabled: bool,
    pub password_hash: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserSettings {
    #[serde(default = "default_true")]
    pub tracking_enabled: bool,
    #[serde(default)]
    pub poll_interval_secs: Option<u64>,
    #[serde(default)]
    pub idle_threshold_secs: Option<u64>,
    #[serde(default)]
    pub network_sample_interval_secs: Option<u64>,
    #[serde(default)]
    pub privacy_hide_titles: bool,
    #[serde(default)]
    pub privacy_hide_urls: bool,
}

fn default_true() -> bool {
    true
}

impl Default for UserSettings {
    fn default() -> Self {
        Self {
            tracking_enabled: true,
            poll_interval_secs: None,
            idle_threshold_secs: None,
            network_sample_interval_secs: None,
            privacy_hide_titles: false,
            privacy_hide_urls: false,
        }
    }
}

impl UserSettings {
    pub fn from_user(tracking_enabled: bool, settings: &Value) -> Self {
        let mut parsed: UserSettings = serde_json::from_value(settings.clone()).unwrap_or_default();
        parsed.tracking_enabled = tracking_enabled;
        parsed
    }

    pub fn to_json(&self) -> Value {
        serde_json::json!({
            "pollIntervalSecs": self.poll_interval_secs,
            "idleThresholdSecs": self.idle_threshold_secs,
            "networkSampleIntervalSecs": self.network_sample_interval_secs,
            "privacyHideTitles": self.privacy_hide_titles,
            "privacyHideUrls": self.privacy_hide_urls,
        })
    }
}

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

        let exists =
            sqlx::query_scalar::<_, bool>("SELECT EXISTS(SELECT 1 FROM users WHERE id = $1)")
                .bind(user_id)
                .fetch_one(self.pool)
                .await?;

        if exists {
            Ok(user_id)
        } else {
            self.ensure_local_user().await
        }
    }

    pub async fn list_ids(&self) -> anyhow::Result<Vec<Uuid>> {
        let ids = sqlx::query_scalar::<_, Uuid>("SELECT id FROM users ORDER BY created_at ASC")
            .fetch_all(self.pool)
            .await?;
        Ok(ids)
    }

    pub async fn get_by_id(&self, user_id: Uuid) -> anyhow::Result<Option<UserRow>> {
        let row = sqlx::query_as::<_, UserRow>(
            r#"
            SELECT id, email, display_name, settings, tracking_enabled, password_hash, created_at, updated_at
            FROM users
            WHERE id = $1
            "#,
        )
        .bind(user_id)
        .fetch_optional(self.pool)
        .await?;
        Ok(row)
    }

    pub async fn get_by_email(&self, email: &str) -> anyhow::Result<Option<UserRow>> {
        let row = sqlx::query_as::<_, UserRow>(
            r#"
            SELECT id, email, display_name, settings, tracking_enabled, password_hash, created_at, updated_at
            FROM users
            WHERE lower(email) = lower($1)
            "#,
        )
        .bind(email)
        .fetch_optional(self.pool)
        .await?;
        Ok(row)
    }

    pub async fn create_user(
        &self,
        email: &str,
        display_name: &str,
        password_hash: &str,
    ) -> anyhow::Result<UserRow> {
        let row = sqlx::query_as::<_, UserRow>(
            r#"
            INSERT INTO users (email, display_name, password_hash)
            VALUES ($1, $2, $3)
            RETURNING id, email, display_name, settings, tracking_enabled, password_hash, created_at, updated_at
            "#,
        )
        .bind(email)
        .bind(display_name)
        .bind(password_hash)
        .fetch_one(self.pool)
        .await?;
        Ok(row)
    }

    pub async fn get_settings(&self, user_id: Uuid) -> anyhow::Result<UserSettings> {
        let row = self
            .get_by_id(user_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("user not found"))?;
        Ok(UserSettings::from_user(row.tracking_enabled, &row.settings))
    }

    pub async fn update_settings(
        &self,
        user_id: Uuid,
        settings: &UserSettings,
    ) -> anyhow::Result<UserSettings> {
        sqlx::query(
            r#"
            UPDATE users
            SET tracking_enabled = $2,
                settings = $3,
                updated_at = now()
            WHERE id = $1
            "#,
        )
        .bind(user_id)
        .bind(settings.tracking_enabled)
        .bind(settings.to_json())
        .execute(self.pool)
        .await?;

        self.get_settings(user_id).await
    }

    pub async fn wipe_activity_data(&self, user_id: Uuid) -> anyhow::Result<u64> {
        let mut tx = self.pool.begin().await?;
        let mut total = 0u64;

        for table in [
            "raw_events",
            "network_logs",
            "website_logs",
            "app_activity_logs",
            "sessions",
            "reports",
        ] {
            let q = format!("DELETE FROM {table} WHERE user_id = $1");
            let result = sqlx::query(&q).bind(user_id).execute(&mut *tx).await?;
            total += result.rows_affected();
        }

        tx.commit().await?;
        Ok(total)
    }
}
