use sqlx::SqlitePool;

use crate::{error::StorageError, models::*};

pub trait PersonRepository: Send + Sync {
    fn get_person(
        &self,
        id: i64,
    ) -> impl std::future::Future<Output = Result<Option<PersonRow>, StorageError>> + Send;
    fn list_persons(
        &self,
        tree_id: i64,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> impl std::future::Future<Output = Result<Vec<PersonRow>, StorageError>> + Send;
}

pub trait FamilyRepository: Send + Sync {
    fn get_family(
        &self,
        id: i64,
    ) -> impl std::future::Future<Output = Result<Option<FamilyRow>, StorageError>> + Send;
    fn list_families(
        &self,
        tree_id: i64,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> impl std::future::Future<Output = Result<Vec<FamilyRow>, StorageError>> + Send;
}

// Concrete implementation via SqlitePool
pub struct Storage {
    pub pool: SqlitePool,
}

impl Storage {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn get_tree(&self, id: i64) -> Result<Option<TreeRow>, StorageError> {
        let row = sqlx::query_as::<_, TreeRow>("SELECT * FROM trees WHERE id=?1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await?;
        Ok(row)
    }
    pub async fn list_trees(
        &self,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> Result<Vec<TreeRow>, StorageError> {
        let limit = limit.unwrap_or(100);
        let offset = offset.unwrap_or(0);
        let rows =
            sqlx::query_as::<_, TreeRow>("SELECT * FROM trees ORDER BY id LIMIT ?1 OFFSET ?2")
                .bind(limit)
                .bind(offset)
                .fetch_all(&self.pool)
                .await?;
        Ok(rows)
    }
    pub async fn get_person(&self, id: i64) -> Result<Option<PersonRow>, StorageError> {
        let r = sqlx::query_as::<_, PersonRow>("SELECT * FROM persons WHERE id=?1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await?;
        Ok(r)
    }
    pub async fn list_persons(
        &self,
        tree_id: i64,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> Result<Vec<PersonRow>, StorageError> {
        let limit = limit.unwrap_or(100);
        let offset = offset.unwrap_or(0);
        let rows = sqlx::query_as::<_, PersonRow>(
            "SELECT * FROM persons WHERE tree_id=?1 ORDER BY id LIMIT ?2 OFFSET ?3",
        )
        .bind(tree_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows)
    }
    pub async fn get_family(&self, id: i64) -> Result<Option<FamilyRow>, StorageError> {
        let r = sqlx::query_as::<_, FamilyRow>("SELECT * FROM families WHERE id=?1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await?;
        Ok(r)
    }
    pub async fn list_families(
        &self,
        tree_id: i64,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> Result<Vec<FamilyRow>, StorageError> {
        let limit = limit.unwrap_or(100);
        let offset = offset.unwrap_or(0);
        let rows = sqlx::query_as::<_, FamilyRow>(
            "SELECT * FROM families WHERE tree_id=?1 ORDER BY id LIMIT ?2 OFFSET ?3",
        )
        .bind(tree_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows)
    }
    pub async fn get_findings(
        &self,
        tree_id: i64,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> Result<Vec<FindingRow>, StorageError> {
        let limit = limit.unwrap_or(100);
        let offset = offset.unwrap_or(0);
        let rows = sqlx::query_as::<_, FindingRow>(
            "SELECT * FROM findings WHERE tree_id=?1 ORDER BY id LIMIT ?2 OFFSET ?3",
        )
        .bind(tree_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows)
    }
    pub async fn get_research_opportunities(
        &self,
        tree_id: i64,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> Result<Vec<ResearchOpportunityRow>, StorageError> {
        let limit = limit.unwrap_or(100);
        let offset = offset.unwrap_or(0);
        let rows = sqlx::query_as::<_, ResearchOpportunityRow>("SELECT * FROM research_opportunities WHERE tree_id=?1 ORDER BY score DESC LIMIT ?2 OFFSET ?3")
            .bind(tree_id).bind(limit).bind(offset).fetch_all(&self.pool).await?;
        Ok(rows)
    }
    pub async fn get_top_research_opportunities(
        &self,
        tree_id: i64,
        min_priority: Option<&str>,
        limit: i64,
    ) -> Result<Vec<ResearchOpportunityRow>, StorageError> {
        // priority filtering: compare rank if needed, but simple: if min_priority is high/critical filter by score thresholds
        // Instead filter by priority string rank in memory or SQL CASE?
        // For simplicity if min_priority provided, filter in Rust after fetch large set, or SQL WHERE priority in list
        let rows = if let Some(min) = min_priority {
            let allowed: Vec<&str> = match min.to_lowercase().as_str() {
                "critical" => vec!["critical"],
                "high" => vec!["critical", "high"],
                "medium" => vec!["critical", "high", "medium"],
                "warning" => vec!["critical", "high", "warning", "medium"],
                _ => vec!["critical", "high", "medium", "warning", "info", "low"],
            };
            let _placeholders = allowed.iter().map(|_| "?").collect::<Vec<_>>().join(",");
            // dynamic query not trivial with query_as, fetch all and filter
            let all = self
                .get_research_opportunities(tree_id, Some(1000), Some(0))
                .await?;
            all.into_iter()
                .filter(|r| {
                    if let Some(p) = &r.priority {
                        allowed.contains(&p.as_str())
                    } else {
                        false
                    }
                })
                .take(limit as usize)
                .collect()
        } else {
            self.get_research_opportunities(tree_id, Some(limit), Some(0))
                .await?
        };
        Ok(rows)
    }
    pub async fn get_branches(&self, tree_id: i64) -> Result<Vec<BranchAnalysisRow>, StorageError> {
        let rows = sqlx::query_as::<_, BranchAnalysisRow>(
            "SELECT * FROM branch_analyses WHERE tree_id=?1 ORDER BY score DESC",
        )
        .bind(tree_id)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows)
    }
    pub async fn get_source_coverage(
        &self,
        tree_id: i64,
    ) -> Result<Option<SourceCoverageRow>, StorageError> {
        let row = sqlx::query_as::<_, SourceCoverageRow>(
            "SELECT * FROM source_coverages WHERE tree_id=?1 ORDER BY id DESC LIMIT 1",
        )
        .bind(tree_id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row)
    }
    pub async fn get_analysis_runs(
        &self,
        tree_id: i64,
    ) -> Result<Vec<AnalysisRunRow>, StorageError> {
        let rows = sqlx::query_as::<_, AnalysisRunRow>(
            "SELECT * FROM analysis_runs WHERE tree_id=?1 ORDER BY id DESC",
        )
        .bind(tree_id)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows)
    }
    pub async fn count(
        &self,
        tree_id: i64,
    ) -> Result<(i64, i64, i64, i64, i64, i64), StorageError> {
        let persons: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM persons WHERE tree_id=?1")
            .bind(tree_id)
            .fetch_one(&self.pool)
            .await?;
        let families: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM families WHERE tree_id=?1")
            .bind(tree_id)
            .fetch_one(&self.pool)
            .await?;
        let events: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM events WHERE tree_id=?1")
            .bind(tree_id)
            .fetch_one(&self.pool)
            .await?;
        let sources: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM sources WHERE tree_id=?1")
            .bind(tree_id)
            .fetch_one(&self.pool)
            .await?;
        let findings: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM findings WHERE tree_id=?1")
            .bind(tree_id)
            .fetch_one(&self.pool)
            .await?;
        let opps: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM research_opportunities WHERE tree_id=?1")
                .bind(tree_id)
                .fetch_one(&self.pool)
                .await?;
        Ok((persons, families, events, sources, findings, opps))
    }
}
