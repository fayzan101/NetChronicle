use netchronicle_common::ActivityCategory;
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::{category_to_db, CategoryRuleRow};

pub struct CategoryRuleRepository<'a> {
    pool: &'a PgPool,
}

impl<'a> CategoryRuleRepository<'a> {
    pub fn new(pool: &'a PgPool) -> Self {
        Self { pool }
    }

    pub async fn list(&self, user_id: Uuid) -> anyhow::Result<Vec<CategoryRuleRow>> {
        let rows = sqlx::query_as::<_, CategoryRuleRow>(
            r#"
            SELECT id, user_id, pattern, pattern_type, category::text AS category, priority, created_at
            FROM category_rules
            WHERE user_id = $1
            ORDER BY priority DESC, created_at ASC
            "#,
        )
        .bind(user_id)
        .fetch_all(self.pool)
        .await?;

        Ok(rows)
    }

    pub async fn create(
        &self,
        user_id: Uuid,
        pattern: &str,
        pattern_type: &str,
        category: ActivityCategory,
        priority: i32,
    ) -> anyhow::Result<CategoryRuleRow> {
        let row = sqlx::query_as::<_, CategoryRuleRow>(
            r#"
            INSERT INTO category_rules (user_id, pattern, pattern_type, category, priority)
            VALUES ($1, $2, $3, $4::activity_category, $5)
            RETURNING id, user_id, pattern, pattern_type, category::text AS category, priority, created_at
            "#,
        )
        .bind(user_id)
        .bind(pattern)
        .bind(pattern_type)
        .bind(category_to_db(category))
        .bind(priority)
        .fetch_one(self.pool)
        .await?;

        Ok(row)
    }

    pub async fn update(
        &self,
        user_id: Uuid,
        rule_id: Uuid,
        pattern: &str,
        pattern_type: &str,
        category: ActivityCategory,
        priority: i32,
    ) -> anyhow::Result<Option<CategoryRuleRow>> {
        let row = sqlx::query_as::<_, CategoryRuleRow>(
            r#"
            UPDATE category_rules
            SET pattern = $3, pattern_type = $4, category = $5::activity_category, priority = $6
            WHERE id = $1 AND user_id = $2
            RETURNING id, user_id, pattern, pattern_type, category::text AS category, priority, created_at
            "#,
        )
        .bind(rule_id)
        .bind(user_id)
        .bind(pattern)
        .bind(pattern_type)
        .bind(category_to_db(category))
        .bind(priority)
        .fetch_optional(self.pool)
        .await?;

        Ok(row)
    }

    pub async fn delete(&self, user_id: Uuid, rule_id: Uuid) -> anyhow::Result<bool> {
        let result = sqlx::query(
            r#"
            DELETE FROM category_rules
            WHERE id = $1 AND user_id = $2
            "#,
        )
        .bind(rule_id)
        .bind(user_id)
        .execute(self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }
}
