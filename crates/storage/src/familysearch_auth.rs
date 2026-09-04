use crate::error::StorageError;
use crate::models::now_iso;

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct FamilySearchConnectionRow {
    pub id: i64,
    pub access_token: String,
    pub token_type: Option<String>,
    pub expires_at: Option<String>,
    pub scope: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

impl crate::repositories::Storage {
    pub async fn save_familysearch_token(
        &self,
        access_token: &str,
        expires_in: Option<u64>,
        token_type: Option<&str>,
        scope: Option<&str>,
    ) -> Result<(), StorageError> {
        let now = now_iso();
        let expires_at = expires_in.map(|secs| {
            let dt = chrono::Utc::now() + chrono::Duration::seconds(secs as i64);
            dt.to_rfc3339()
        });
        // Upsert id=1
        sqlx::query(
            "INSERT INTO familysearch_connections (id, access_token, token_type, expires_at, scope, created_at, updated_at)
             VALUES (1, ?1, ?2, ?3, ?4, ?5, ?6)
             ON CONFLICT(id) DO UPDATE SET access_token=excluded.access_token, token_type=excluded.token_type, expires_at=excluded.expires_at, scope=excluded.scope, updated_at=excluded.updated_at"
        )
        .bind(access_token)
        .bind(token_type)
        .bind(&expires_at)
        .bind(scope)
        .bind(&now)
        .bind(&now)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn get_familysearch_token(
        &self,
    ) -> Result<Option<FamilySearchConnectionRow>, StorageError> {
        let row = sqlx::query_as::<_, FamilySearchConnectionRow>(
            "SELECT * FROM familysearch_connections WHERE id=1",
        )
        .fetch_optional(&self.pool)
        .await?;
        Ok(row)
    }

    pub async fn delete_familysearch_token(&self) -> Result<(), StorageError> {
        sqlx::query("DELETE FROM familysearch_connections WHERE id=1")
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn is_familysearch_token_valid(&self) -> Result<bool, StorageError> {
        if let Some(row) = self.get_familysearch_token().await? {
            if let Some(exp) = row.expires_at {
                if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(&exp) {
                    let now = chrono::Utc::now();
                    if dt.with_timezone(&chrono::Utc) <= now {
                        return Ok(false);
                    }
                }
            }
            return Ok(true);
        }
        Ok(false)
    }

    pub async fn save_oauth_state(&self, state: &str) -> Result<(), StorageError> {
        let now = now_iso();
        sqlx::query("INSERT INTO familysearch_oauth_states (state, created_at) VALUES (?1, ?2)")
            .bind(state)
            .bind(&now)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn consume_oauth_state(&self, state: &str) -> Result<bool, StorageError> {
        // Check exists and not expired (>10 min)
        let row: Option<(String, String)> = sqlx::query_as(
            "SELECT state, created_at FROM familysearch_oauth_states WHERE state=?1",
        )
        .bind(state)
        .fetch_optional(&self.pool)
        .await?;
        if let Some((_, created_at)) = row {
            // Delete regardless
            sqlx::query("DELETE FROM familysearch_oauth_states WHERE state=?1")
                .bind(state)
                .execute(&self.pool)
                .await?;
            // Check expiry 10 minutes
            if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(&created_at) {
                let now = chrono::Utc::now();
                let diff = now.signed_duration_since(dt.with_timezone(&chrono::Utc));
                if diff.num_minutes() > 10 {
                    return Ok(false);
                }
                return Ok(true);
            }
            return Ok(true);
        }
        Ok(false)
    }

    pub async fn cleanup_expired_oauth_states(&self) -> Result<(), StorageError> {
        // Delete states older than 15 minutes
        let threshold = (chrono::Utc::now() - chrono::Duration::minutes(15)).to_rfc3339();
        sqlx::query("DELETE FROM familysearch_oauth_states WHERE created_at < ?1")
            .bind(threshold)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}
