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

    pub async fn count_trees(&self) -> Result<i64, StorageError> {
        let cnt: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM trees")
            .fetch_one(&self.pool)
            .await?;
        Ok(cnt)
    }

    pub async fn count_persons(&self, tree_id: i64) -> Result<i64, StorageError> {
        let cnt: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM persons WHERE tree_id=?1")
            .bind(tree_id)
            .fetch_one(&self.pool)
            .await?;
        Ok(cnt)
    }

    pub async fn count_families(&self, tree_id: i64) -> Result<i64, StorageError> {
        let cnt: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM families WHERE tree_id=?1")
            .bind(tree_id)
            .fetch_one(&self.pool)
            .await?;
        Ok(cnt)
    }

    pub async fn list_findings_filtered(
        &self,
        tree_id: i64,
        severity: Option<&str>,
        finding_type: Option<&str>,
        person_id: Option<i64>,
        limit: i64,
        offset: i64,
    ) -> Result<(Vec<FindingRow>, i64), StorageError> {
        let mut sql = "SELECT * FROM findings WHERE tree_id = ?".to_string();
        let mut count_sql = "SELECT COUNT(*) FROM findings WHERE tree_id = ?".to_string();
        if severity.is_some() {
            sql.push_str(" AND severity = ?");
            count_sql.push_str(" AND severity = ?");
        }
        if finding_type.is_some() {
            sql.push_str(" AND finding_type = ?");
            count_sql.push_str(" AND finding_type = ?");
        }
        if person_id.is_some() {
            sql.push_str(" AND person_id = ?");
            count_sql.push_str(" AND person_id = ?");
        }
        sql.push_str(" ORDER BY id LIMIT ? OFFSET ?");
        let mut cq = sqlx::query_scalar::<_, i64>(&count_sql).bind(tree_id);
        let mut q = sqlx::query_as::<_, FindingRow>(&sql).bind(tree_id);
        if let Some(sev) = severity {
            cq = cq.bind(sev.to_lowercase());
            q = q.bind(sev.to_lowercase());
        }
        if let Some(t) = finding_type {
            cq = cq.bind(t);
            q = q.bind(t);
        }
        if let Some(pid) = person_id {
            cq = cq.bind(pid);
            q = q.bind(pid);
        }
        let total = cq.fetch_one(&self.pool).await?;
        q = q.bind(limit).bind(offset);
        let rows = q.fetch_all(&self.pool).await?;
        Ok((rows, total))
    }

    pub async fn list_opportunities_filtered(
        &self,
        tree_id: i64,
        priority: Option<&str>,
        min_score: Option<i64>,
        sort: Option<&str>,
        limit: i64,
        offset: i64,
    ) -> Result<(Vec<ResearchOpportunityRow>, i64), StorageError> {
        let mut sql = "SELECT * FROM research_opportunities WHERE tree_id = ?".to_string();
        let mut count_sql =
            "SELECT COUNT(*) FROM research_opportunities WHERE tree_id = ?".to_string();
        if priority.is_some() {
            sql.push_str(" AND priority = ?");
            count_sql.push_str(" AND priority = ?");
        }
        if min_score.is_some() {
            sql.push_str(" AND score >= ?");
            count_sql.push_str(" AND score >= ?");
        }
        let order = match sort.map(|s| s.to_lowercase()).as_deref() {
            Some("priority") => " ORDER BY CASE priority WHEN 'critical' THEN 5 WHEN 'high' THEN 4 WHEN 'medium' THEN 3 WHEN 'warning' THEN 2 WHEN 'info' THEN 1 ELSE 0 END DESC",
            Some("confidence") => " ORDER BY confidence DESC",
            _ => " ORDER BY score DESC",
        };
        sql.push_str(order);
        sql.push_str(" LIMIT ? OFFSET ?");
        let mut cq = sqlx::query_scalar::<_, i64>(&count_sql).bind(tree_id);
        let mut q = sqlx::query_as::<_, ResearchOpportunityRow>(&sql).bind(tree_id);
        if let Some(p) = priority {
            cq = cq.bind(p.to_lowercase());
            q = q.bind(p.to_lowercase());
        }
        if let Some(ms) = min_score {
            cq = cq.bind(ms);
            q = q.bind(ms);
        }
        let total = cq.fetch_one(&self.pool).await?;
        q = q.bind(limit).bind(offset);
        let rows = q.fetch_all(&self.pool).await?;
        Ok((rows, total))
    }

    pub async fn get_family_members(
        &self,
        family_ids: &[i64],
    ) -> Result<Vec<FamilyMemberRow>, StorageError> {
        if family_ids.is_empty() {
            return Ok(Vec::new());
        }
        let placeholders = family_ids.iter().map(|_| "?").collect::<Vec<_>>().join(",");
        let query = format!("SELECT * FROM family_members WHERE family_id IN ({placeholders})");
        let mut q = sqlx::query_as::<_, FamilyMemberRow>(&query);
        for id in family_ids {
            q = q.bind(id);
        }
        let rows = q.fetch_all(&self.pool).await?;
        Ok(rows)
    }

    pub async fn get_family_members_for_family(
        &self,
        family_id: i64,
    ) -> Result<Vec<FamilyMemberRow>, StorageError> {
        let rows =
            sqlx::query_as::<_, FamilyMemberRow>("SELECT * FROM family_members WHERE family_id=?1")
                .bind(family_id)
                .fetch_all(&self.pool)
                .await?;
        Ok(rows)
    }

    // --- Research Tasks ---
    pub async fn create_research_task(
        &self,
        tree_id: i64,
        opportunity_id: Option<i64>,
        person_id: Option<i64>,
        title: &str,
        description: Option<&str>,
    ) -> Result<ResearchTaskRow, StorageError> {
        // Validate tree isolation for opportunity/person if provided
        if let Some(oid) = opportunity_id {
            let opp_tree: Option<i64> =
                sqlx::query_scalar("SELECT tree_id FROM research_opportunities WHERE id=?1")
                    .bind(oid)
                    .fetch_optional(&self.pool)
                    .await?
                    .flatten();
            if opp_tree != Some(tree_id) {
                return Err(StorageError::NotFound(format!(
                    "opportunity {oid} not in tree {tree_id}"
                )));
            }
        }
        if let Some(pid) = person_id {
            let p_tree: Option<i64> = sqlx::query_scalar("SELECT tree_id FROM persons WHERE id=?1")
                .bind(pid)
                .fetch_optional(&self.pool)
                .await?
                .flatten();
            if p_tree != Some(tree_id) {
                return Err(StorageError::NotFound(format!(
                    "person {pid} not in tree {tree_id}"
                )));
            }
        }
        // Check duplicate active task for same opportunity
        if let Some(oid) = opportunity_id {
            let existing: Option<i64> = sqlx::query_scalar(
                "SELECT id FROM research_tasks WHERE opportunity_id=?1 AND status IN ('OPEN','IN_PROGRESS') LIMIT 1",
            )
            .bind(oid)
            .fetch_optional(&self.pool)
            .await?
            .flatten();
            if let Some(eid) = existing {
                // Return existing to avoid duplicate (spec: reuse)
                let row = sqlx::query_as::<_, ResearchTaskRow>(
                    "SELECT * FROM research_tasks WHERE id=?1",
                )
                .bind(eid)
                .fetch_one(&self.pool)
                .await?;
                return Ok(row);
            }
        }
        let now = crate::models::now_iso();
        let res = sqlx::query(
            "INSERT INTO research_tasks (tree_id, opportunity_id, person_id, title, description, status, created_at, updated_at) VALUES (?1,?2,?3,?4,?5,'OPEN',?6,?7)",
        )
        .bind(tree_id)
        .bind(opportunity_id)
        .bind(person_id)
        .bind(title)
        .bind(description)
        .bind(&now)
        .bind(&now)
        .execute(&self.pool)
        .await;
        match res {
            Ok(r) => {
                let id = r.last_insert_rowid();
                let row = sqlx::query_as::<_, ResearchTaskRow>(
                    "SELECT * FROM research_tasks WHERE id=?1",
                )
                .bind(id)
                .fetch_one(&self.pool)
                .await?;
                Ok(row)
            }
            Err(e) => {
                // Handle unique constraint violation as conflict
                if e.to_string().contains("UNIQUE") {
                    return Err(StorageError::Import(
                        "duplicate active task for opportunity".into(),
                    ));
                }
                Err(e.into())
            }
        }
    }

    pub async fn get_research_task(
        &self,
        id: i64,
    ) -> Result<Option<ResearchTaskRow>, StorageError> {
        let row = sqlx::query_as::<_, ResearchTaskRow>("SELECT * FROM research_tasks WHERE id=?1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await?;
        Ok(row)
    }

    pub async fn list_research_tasks(
        &self,
        tree_id: i64,
        status: Option<&str>,
        person_id: Option<i64>,
        opportunity_id: Option<i64>,
        limit: i64,
        offset: i64,
    ) -> Result<(Vec<ResearchTaskRow>, i64), StorageError> {
        let mut sql = "SELECT * FROM research_tasks WHERE tree_id = ?".to_string();
        let mut count_sql = "SELECT COUNT(*) FROM research_tasks WHERE tree_id = ?".to_string();
        if status.is_some() {
            sql.push_str(" AND status = ?");
            count_sql.push_str(" AND status = ?");
        }
        if person_id.is_some() {
            sql.push_str(" AND person_id = ?");
            count_sql.push_str(" AND person_id = ?");
        }
        if opportunity_id.is_some() {
            sql.push_str(" AND opportunity_id = ?");
            count_sql.push_str(" AND opportunity_id = ?");
        }
        sql.push_str(" ORDER BY updated_at DESC LIMIT ? OFFSET ?");
        let mut cq = sqlx::query_scalar::<_, i64>(&count_sql).bind(tree_id);
        let mut q = sqlx::query_as::<_, ResearchTaskRow>(&sql).bind(tree_id);
        if let Some(s) = status {
            cq = cq.bind(s);
            q = q.bind(s);
        }
        if let Some(pid) = person_id {
            cq = cq.bind(pid);
            q = q.bind(pid);
        }
        if let Some(oid) = opportunity_id {
            cq = cq.bind(oid);
            q = q.bind(oid);
        }
        let total = cq.fetch_one(&self.pool).await?;
        q = q.bind(limit).bind(offset);
        let rows = q.fetch_all(&self.pool).await?;
        Ok((rows, total))
    }

    pub async fn update_research_task(
        &self,
        id: i64,
        title: Option<&str>,
        description: Option<&str>,
        status: Option<&str>,
        resolution: Option<&str>,
    ) -> Result<ResearchTaskRow, StorageError> {
        let existing = self
            .get_research_task(id)
            .await?
            .ok_or_else(|| StorageError::NotFound(format!("task {id} not found")))?;
        let new_title = title.unwrap_or(&existing.title).to_string();
        let new_desc = description.or(existing.description.as_deref());
        let new_status = status.unwrap_or(&existing.status).to_string();
        let valid = [
            "OPEN",
            "IN_PROGRESS",
            "RESOLVED",
            "REJECTED",
            "INCONCLUSIVE",
        ];
        if !valid.contains(&new_status.as_str()) {
            return Err(StorageError::Import(format!("invalid status {new_status}")));
        }
        let now = crate::models::now_iso();
        let mut started_at = existing.started_at.clone();
        let mut completed_at = existing.completed_at.clone();
        if new_status == "IN_PROGRESS" && started_at.is_none() {
            started_at = Some(now.clone());
        }
        if ["RESOLVED", "REJECTED", "INCONCLUSIVE"].contains(&new_status.as_str())
            && completed_at.is_none()
        {
            completed_at = Some(now.clone());
        }
        let new_resolution = resolution.or(existing.resolution.as_deref());
        sqlx::query(
            "UPDATE research_tasks SET title=?1, description=?2, status=?3, resolution=?4, updated_at=?5, started_at=?6, completed_at=?7 WHERE id=?8",
        )
        .bind(&new_title)
        .bind(new_desc)
        .bind(&new_status)
        .bind(new_resolution)
        .bind(&now)
        .bind(&started_at)
        .bind(&completed_at)
        .bind(id)
        .execute(&self.pool)
        .await?;
        let row = self.get_research_task(id).await?.unwrap();
        Ok(row)
    }

    pub async fn delete_research_task(&self, id: i64) -> Result<(), StorageError> {
        sqlx::query("DELETE FROM research_tasks WHERE id=?1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    // --- Research Outcomes ---
    pub async fn create_research_outcome(
        &self,
        tree_id: i64,
        task_id: i64,
        outcome_type: &str,
        summary: &str,
        details: Option<&str>,
    ) -> Result<ResearchOutcomeRow, StorageError> {
        let valid = [
            "CONFIRMED",
            "FALSE_LEAD",
            "INCONCLUSIVE",
            "NEW_LEAD",
            "NO_EVIDENCE",
        ];
        if !valid.contains(&outcome_type) {
            return Err(StorageError::Import(format!(
                "invalid outcome type {outcome_type}"
            )));
        }
        if summary.trim().is_empty() {
            return Err(StorageError::Import("summary must not be empty".into()));
        }
        // Validate task exists and belongs to same tree
        let task = self
            .get_research_task(task_id)
            .await?
            .ok_or_else(|| StorageError::NotFound(format!("task {task_id} not found")))?;
        if task.tree_id != tree_id {
            return Err(StorageError::NotFound(format!(
                "task {task_id} not in tree {tree_id}"
            )));
        }
        // Check unique task_id
        let existing: Option<i64> =
            sqlx::query_scalar("SELECT id FROM research_outcomes WHERE task_id=?1")
                .bind(task_id)
                .fetch_optional(&self.pool)
                .await?
                .flatten();
        if existing.is_some() {
            return Err(StorageError::Import(
                "outcome already exists for task".into(),
            ));
        }
        let now = crate::models::now_iso();
        let res = sqlx::query(
            "INSERT INTO research_outcomes (tree_id, task_id, type, summary, details, created_at, updated_at) VALUES (?1,?2,?3,?4,?5,?6,?7)",
        )
        .bind(tree_id)
        .bind(task_id)
        .bind(outcome_type)
        .bind(summary)
        .bind(details)
        .bind(&now)
        .bind(&now)
        .execute(&self.pool)
        .await;
        match res {
            Ok(r) => {
                let id = r.last_insert_rowid();
                let row = sqlx::query_as::<_, ResearchOutcomeRow>(
                    "SELECT * FROM research_outcomes WHERE id=?1",
                )
                .bind(id)
                .fetch_one(&self.pool)
                .await?;
                Ok(row)
            }
            Err(e) => {
                if e.to_string().contains("UNIQUE") {
                    return Err(StorageError::Import(
                        "outcome already exists for task".into(),
                    ));
                }
                Err(e.into())
            }
        }
    }

    pub async fn get_research_outcome(
        &self,
        id: i64,
    ) -> Result<Option<ResearchOutcomeRow>, StorageError> {
        let row =
            sqlx::query_as::<_, ResearchOutcomeRow>("SELECT * FROM research_outcomes WHERE id=?1")
                .bind(id)
                .fetch_optional(&self.pool)
                .await?;
        Ok(row)
    }

    pub async fn get_research_outcome_by_task(
        &self,
        task_id: i64,
    ) -> Result<Option<ResearchOutcomeRow>, StorageError> {
        let row = sqlx::query_as::<_, ResearchOutcomeRow>(
            "SELECT * FROM research_outcomes WHERE task_id=?1",
        )
        .bind(task_id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row)
    }

    pub async fn list_research_outcomes(
        &self,
        tree_id: i64,
        outcome_type: Option<&str>,
        task_id: Option<i64>,
        limit: i64,
        offset: i64,
    ) -> Result<(Vec<ResearchOutcomeRow>, i64), StorageError> {
        // For person_id filter, join via research_tasks
        // This method supports type, task_id; person_id filter is handled via join in a separate method or via task filter
        let mut sql = "SELECT ro.* FROM research_outcomes ro WHERE ro.tree_id = ?".to_string();
        let mut count_sql =
            "SELECT COUNT(*) FROM research_outcomes ro WHERE ro.tree_id = ?".to_string();
        if outcome_type.is_some() {
            sql.push_str(" AND ro.type = ?");
            count_sql.push_str(" AND ro.type = ?");
        }
        if task_id.is_some() {
            sql.push_str(" AND ro.task_id = ?");
            count_sql.push_str(" AND ro.task_id = ?");
        }
        sql.push_str(" ORDER BY ro.updated_at DESC LIMIT ? OFFSET ?");
        let mut cq = sqlx::query_scalar::<_, i64>(&count_sql).bind(tree_id);
        let mut q = sqlx::query_as::<_, ResearchOutcomeRow>(&sql).bind(tree_id);
        if let Some(t) = outcome_type {
            cq = cq.bind(t);
            q = q.bind(t);
        }
        if let Some(tid) = task_id {
            cq = cq.bind(tid);
            q = q.bind(tid);
        }
        let total = cq.fetch_one(&self.pool).await?;
        q = q.bind(limit).bind(offset);
        let rows = q.fetch_all(&self.pool).await?;
        Ok((rows, total))
    }

    pub async fn list_research_outcomes_with_person(
        &self,
        tree_id: i64,
        outcome_type: Option<&str>,
        person_id: Option<i64>,
        limit: i64,
        offset: i64,
    ) -> Result<(Vec<ResearchOutcomeRow>, i64), StorageError> {
        if let Some(pid) = person_id {
            // Join to filter by person_id via task
            let mut sql = "SELECT ro.* FROM research_outcomes ro JOIN research_tasks rt ON ro.task_id = rt.id WHERE ro.tree_id = ? AND rt.person_id = ?".to_string();
            let mut count_sql = "SELECT COUNT(*) FROM research_outcomes ro JOIN research_tasks rt ON ro.task_id = rt.id WHERE ro.tree_id = ? AND rt.person_id = ?".to_string();
            if outcome_type.is_some() {
                sql.push_str(" AND ro.type = ?");
                count_sql.push_str(" AND ro.type = ?");
            }
            sql.push_str(" ORDER BY ro.updated_at DESC LIMIT ? OFFSET ?");
            let mut cq = sqlx::query_scalar::<_, i64>(&count_sql)
                .bind(tree_id)
                .bind(pid);
            let mut q = sqlx::query_as::<_, ResearchOutcomeRow>(&sql)
                .bind(tree_id)
                .bind(pid);
            if let Some(t) = outcome_type {
                cq = cq.bind(t);
                q = q.bind(t);
            }
            let total = cq.fetch_one(&self.pool).await?;
            q = q.bind(limit).bind(offset);
            let rows = q.fetch_all(&self.pool).await?;
            Ok((rows, total))
        } else {
            self.list_research_outcomes(tree_id, outcome_type, None, limit, offset)
                .await
        }
    }

    pub async fn update_research_outcome(
        &self,
        id: i64,
        outcome_type: Option<&str>,
        summary: Option<&str>,
        details: Option<&str>,
    ) -> Result<ResearchOutcomeRow, StorageError> {
        let existing = self
            .get_research_outcome(id)
            .await?
            .ok_or_else(|| StorageError::NotFound(format!("outcome {id} not found")))?;
        let new_type = outcome_type.unwrap_or(&existing.r#type).to_string();
        let valid = [
            "CONFIRMED",
            "FALSE_LEAD",
            "INCONCLUSIVE",
            "NEW_LEAD",
            "NO_EVIDENCE",
        ];
        if !valid.contains(&new_type.as_str()) {
            return Err(StorageError::Import(format!(
                "invalid outcome type {new_type}"
            )));
        }
        let new_summary = summary.unwrap_or(&existing.summary).to_string();
        if new_summary.trim().is_empty() {
            return Err(StorageError::Import("summary must not be empty".into()));
        }
        let new_details = details.or(existing.details.as_deref());
        let now = crate::models::now_iso();
        sqlx::query(
            "UPDATE research_outcomes SET type=?1, summary=?2, details=?3, updated_at=?4 WHERE id=?5",
        )
        .bind(&new_type)
        .bind(&new_summary)
        .bind(new_details)
        .bind(&now)
        .bind(id)
        .execute(&self.pool)
        .await?;
        let row = self.get_research_outcome(id).await?.unwrap();
        Ok(row)
    }

    pub async fn delete_research_outcome(&self, id: i64) -> Result<(), StorageError> {
        sqlx::query("DELETE FROM research_outcomes WHERE id=?1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}
