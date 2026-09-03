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
        self.list_research_tasks_filtered(
            tree_id,
            status,
            person_id,
            opportunity_id,
            None,
            limit,
            offset,
        )
        .await
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn list_research_tasks_filtered(
        &self,
        tree_id: i64,
        status: Option<&str>,
        person_id: Option<i64>,
        opportunity_id: Option<i64>,
        has_outcome: Option<bool>,
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
        if let Some(has) = has_outcome {
            if has {
                sql.push_str(" AND EXISTS (SELECT 1 FROM research_outcomes WHERE research_outcomes.task_id = research_tasks.id)");
                count_sql.push_str(" AND EXISTS (SELECT 1 FROM research_outcomes WHERE research_outcomes.task_id = research_tasks.id)");
            } else {
                sql.push_str(" AND NOT EXISTS (SELECT 1 FROM research_outcomes WHERE research_outcomes.task_id = research_tasks.id)");
                count_sql.push_str(" AND NOT EXISTS (SELECT 1 FROM research_outcomes WHERE research_outcomes.task_id = research_tasks.id)");
            }
        }
        sql.push_str(" ORDER BY CASE status WHEN 'IN_PROGRESS' THEN 0 WHEN 'OPEN' THEN 1 ELSE 2 END, updated_at DESC LIMIT ? OFFSET ?");
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

    pub async fn get_tasks_has_outcome_map(
        &self,
        task_ids: &[i64],
    ) -> Result<std::collections::HashMap<i64, bool>, StorageError> {
        if task_ids.is_empty() {
            return Ok(std::collections::HashMap::new());
        }
        let placeholders = task_ids.iter().map(|_| "?").collect::<Vec<_>>().join(",");
        let sql =
            format!("SELECT task_id FROM research_outcomes WHERE task_id IN ({placeholders})");
        let mut q = sqlx::query_scalar::<_, i64>(&sql);
        for id in task_ids {
            q = q.bind(id);
        }
        let with_outcome = q.fetch_all(&self.pool).await?;
        let set: std::collections::HashSet<i64> = with_outcome.into_iter().collect();
        let mut map = std::collections::HashMap::new();
        for id in task_ids {
            map.insert(*id, set.contains(id));
        }
        Ok(map)
    }

    pub async fn research_summary(&self, tree_id: i64) -> Result<serde_json::Value, StorageError> {
        let opp_high: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM research_opportunities WHERE tree_id=?1 AND lower(priority)='high'",
        )
        .bind(tree_id)
        .fetch_one(&self.pool)
        .await?;
        let opp_medium: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM research_opportunities WHERE tree_id=?1 AND lower(priority)='medium'",
        )
        .bind(tree_id)
        .fetch_one(&self.pool)
        .await?;
        let opp_low: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM research_opportunities WHERE tree_id=?1 AND lower(priority)='low'",
        )
        .bind(tree_id)
        .fetch_one(&self.pool)
        .await?;
        let task_open: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM research_tasks WHERE tree_id=?1 AND status='OPEN'",
        )
        .bind(tree_id)
        .fetch_one(&self.pool)
        .await?;
        let task_in_progress: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM research_tasks WHERE tree_id=?1 AND status='IN_PROGRESS'",
        )
        .bind(tree_id)
        .fetch_one(&self.pool)
        .await?;
        let task_resolved: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM research_tasks WHERE tree_id=?1 AND status='RESOLVED'",
        )
        .bind(tree_id)
        .fetch_one(&self.pool)
        .await?;
        let task_rejected: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM research_tasks WHERE tree_id=?1 AND status='REJECTED'",
        )
        .bind(tree_id)
        .fetch_one(&self.pool)
        .await?;
        let task_inconclusive: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM research_tasks WHERE tree_id=?1 AND status='INCONCLUSIVE'",
        )
        .bind(tree_id)
        .fetch_one(&self.pool)
        .await?;
        let outcomes_total: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM research_outcomes WHERE tree_id=?1")
                .bind(tree_id)
                .fetch_one(&self.pool)
                .await?;
        let sources_total: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM research_sources WHERE tree_id=?1")
                .bind(tree_id)
                .fetch_one(&self.pool)
                .await
                .unwrap_or(0);
        let evidence_total: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM evidence WHERE tree_id=?1")
                .bind(tree_id)
                .fetch_one(&self.pool)
                .await
                .unwrap_or(0);
        let evidence_supporting: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM outcome_evidence oe JOIN research_outcomes ro ON oe.outcome_id = ro.id WHERE ro.tree_id=?1 AND oe.relationship='SUPPORTS'",
        )
        .bind(tree_id)
        .fetch_one(&self.pool)
        .await
        .unwrap_or(0);
        let evidence_contradicting: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM outcome_evidence oe JOIN research_outcomes ro ON oe.outcome_id = ro.id WHERE ro.tree_id=?1 AND oe.relationship='CONTRADICTS'",
        )
        .bind(tree_id)
        .fetch_one(&self.pool)
        .await
        .unwrap_or(0);
        // assessment distribution
        let outcome_ids: Vec<i64> =
            sqlx::query_scalar("SELECT id FROM research_outcomes WHERE tree_id=?1")
                .bind(tree_id)
                .fetch_all(&self.pool)
                .await
                .unwrap_or_default();
        let assessments = self
            .get_outcomes_assessments(&outcome_ids)
            .await
            .unwrap_or_default();
        let mut cnt_no = 0;
        let mut cnt_weak = 0;
        let mut cnt_mixed = 0;
        let mut cnt_supported = 0;
        let mut cnt_strong = 0;
        for a in assessments.values() {
            match a.status.as_str() {
                "NO_EVIDENCE" => cnt_no += 1,
                "WEAK" => cnt_weak += 1,
                "MIXED" => cnt_mixed += 1,
                "SUPPORTED" => cnt_supported += 1,
                "STRONGLY_SUPPORTED" => cnt_strong += 1,
                _ => {}
            }
        }
        // outcomes with no evidence not in map are NO_EVIDENCE (already counted via batch fills)
        // but batch fills missing with NO_EVIDENCE, so counts are correct
        // gaps counts by severity
        let gaps_map = self
            .get_outcomes_gaps(&outcome_ids)
            .await
            .unwrap_or_default();
        let mut cnt_gaps_critical = 0;
        let mut cnt_gaps_warning = 0;
        let mut cnt_gaps_info = 0;
        for gaps in gaps_map.values() {
            for g in gaps {
                match g.severity.as_str() {
                    "CRITICAL" => cnt_gaps_critical += 1,
                    "WARNING" => cnt_gaps_warning += 1,
                    "INFO" => cnt_gaps_info += 1,
                    _ => {}
                }
            }
        }
        // followups counts by priority
        let mut cnt_fu_high = 0;
        let mut cnt_fu_medium = 0;
        let mut cnt_fu_low = 0;
        let stats_map = self
            .get_outcomes_evidence_stats(&outcome_ids)
            .await
            .unwrap_or_default();
        // need type map for followups
        let mut type_for_fu: std::collections::HashMap<i64, String> =
            std::collections::HashMap::new();
        if !outcome_ids.is_empty() {
            let placeholders = outcome_ids
                .iter()
                .map(|_| "?")
                .collect::<Vec<_>>()
                .join(",");
            let sql =
                format!("SELECT id, type FROM research_outcomes WHERE id IN ({placeholders})");
            let mut q = sqlx::query_as::<_, (i64, String)>(&sql);
            for id in &outcome_ids {
                q = q.bind(id);
            }
            let rows: Vec<(i64, String)> = q.fetch_all(&self.pool).await.unwrap_or_default();
            for (id, t) in rows {
                type_for_fu.insert(id, t);
            }
        }
        for oid in &outcome_ids {
            let gaps = gaps_map.get(oid);
            let gaps_vec = match gaps {
                Some(v) => v.clone(),
                None => Vec::new(),
            };
            if gaps_vec.is_empty() {
                let stats =
                    stats_map
                        .get(oid)
                        .cloned()
                        .unwrap_or(crate::assessment::EvidenceStats {
                            evidence_total: 0,
                            supporting_count: 0,
                            contradicting_count: 0,
                            sources_count: 0,
                            cited_count: 0,
                            uncited_count: 0,
                            cited_supporting_count: 0,
                        });
                let t = type_for_fu
                    .get(oid)
                    .map(|s| s.as_str())
                    .unwrap_or("INCONCLUSIVE");
                let gaps2 = crate::assessment::calculate_evidence_gaps(t, &stats);
                // shouldn't happen as gaps_map already has gaps, but fallback
                for fu in crate::assessment::calculate_research_followups(t, &stats, &gaps2) {
                    match fu.priority.as_str() {
                        "HIGH" => cnt_fu_high += 1,
                        "MEDIUM" => cnt_fu_medium += 1,
                        "LOW" => cnt_fu_low += 1,
                        _ => {}
                    }
                }
            } else {
                let stats =
                    stats_map
                        .get(oid)
                        .cloned()
                        .unwrap_or(crate::assessment::EvidenceStats {
                            evidence_total: 0,
                            supporting_count: 0,
                            contradicting_count: 0,
                            sources_count: 0,
                            cited_count: 0,
                            uncited_count: 0,
                            cited_supporting_count: 0,
                        });
                let t = type_for_fu
                    .get(oid)
                    .map(|s| s.as_str())
                    .unwrap_or("INCONCLUSIVE");
                for fu in crate::assessment::calculate_research_followups(t, &stats, &gaps_vec) {
                    match fu.priority.as_str() {
                        "HIGH" => cnt_fu_high += 1,
                        "MEDIUM" => cnt_fu_medium += 1,
                        "LOW" => cnt_fu_low += 1,
                        _ => {}
                    }
                }
            }
        }
        let fa_counts = self
            .count_followup_actions_by_status(tree_id)
            .await
            .unwrap_or_default();
        let fa_open = fa_counts.get("OPEN").cloned().unwrap_or(0);
        let fa_completed = fa_counts.get("COMPLETED").cloned().unwrap_or(0);
        let fa_skipped = fa_counts.get("SKIPPED").cloned().unwrap_or(0);
        Ok(serde_json::json!({
            "opportunities": { "high": opp_high, "medium": opp_medium, "low": opp_low },
            "tasks": { "open": task_open, "in_progress": task_in_progress, "resolved": task_resolved, "rejected": task_rejected, "inconclusive": task_inconclusive },
            "outcomes": { "total": outcomes_total },
            "sources": { "total": sources_total },
            "evidence": { "total": evidence_total, "supporting": evidence_supporting, "contradicting": evidence_contradicting },
            "assessment": { "no_evidence": cnt_no, "weak": cnt_weak, "mixed": cnt_mixed, "supported": cnt_supported, "strongly_supported": cnt_strong },
            "evidence_gaps": { "critical": cnt_gaps_critical, "warning": cnt_gaps_warning, "info": cnt_gaps_info },
            "research_followups": { "high": cnt_fu_high, "medium": cnt_fu_medium, "low": cnt_fu_low },
            "followup_actions": { "open": fa_open, "completed": fa_completed, "skipped": fa_skipped }
        }))
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
        sql.push_str(" ORDER BY ro.created_at DESC LIMIT ? OFFSET ?");
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
            sql.push_str(" ORDER BY ro.created_at DESC LIMIT ? OFFSET ?");
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

    // --- Research Sources ---
    pub async fn create_research_source(
        &self,
        tree_id: i64,
        title: &str,
        author: Option<&str>,
        publication: Option<&str>,
        date: Option<&str>,
        source_type: &str,
    ) -> Result<ResearchSourceRow, StorageError> {
        let valid = [
            "BOOK",
            "REGISTER",
            "CENSUS",
            "CIVIL_RECORD",
            "PARISH_RECORD",
            "NEWSPAPER",
            "WEBSITE",
            "OTHER",
        ];
        if !valid.contains(&source_type) {
            return Err(StorageError::Import(format!(
                "invalid source type {source_type}"
            )));
        }
        if title.trim().is_empty() {
            return Err(StorageError::Import("title must not be empty".into()));
        }
        let now = crate::models::now_iso();
        let res = sqlx::query(
            "INSERT INTO research_sources (tree_id, title, author, publication, date, type, created_at, updated_at) VALUES (?1,?2,?3,?4,?5,?6,?7,?8)",
        )
        .bind(tree_id)
        .bind(title)
        .bind(author)
        .bind(publication)
        .bind(date)
        .bind(source_type)
        .bind(&now)
        .bind(&now)
        .execute(&self.pool)
        .await?;
        let id = res.last_insert_rowid();
        let row =
            sqlx::query_as::<_, ResearchSourceRow>("SELECT * FROM research_sources WHERE id=?1")
                .bind(id)
                .fetch_one(&self.pool)
                .await?;
        Ok(row)
    }

    pub async fn get_research_source(
        &self,
        id: i64,
    ) -> Result<Option<ResearchSourceRow>, StorageError> {
        let row =
            sqlx::query_as::<_, ResearchSourceRow>("SELECT * FROM research_sources WHERE id=?1")
                .bind(id)
                .fetch_optional(&self.pool)
                .await?;
        Ok(row)
    }

    pub async fn list_research_sources(
        &self,
        tree_id: i64,
        source_type: Option<&str>,
        limit: i64,
        offset: i64,
    ) -> Result<(Vec<ResearchSourceRow>, i64), StorageError> {
        let mut sql = "SELECT * FROM research_sources WHERE tree_id = ?".to_string();
        let mut count_sql = "SELECT COUNT(*) FROM research_sources WHERE tree_id = ?".to_string();
        if source_type.is_some() {
            sql.push_str(" AND type = ?");
            count_sql.push_str(" AND type = ?");
        }
        sql.push_str(" ORDER BY updated_at DESC LIMIT ? OFFSET ?");
        let mut cq = sqlx::query_scalar::<_, i64>(&count_sql).bind(tree_id);
        let mut q = sqlx::query_as::<_, ResearchSourceRow>(&sql).bind(tree_id);
        if let Some(t) = source_type {
            cq = cq.bind(t);
            q = q.bind(t);
        }
        let total = cq.fetch_one(&self.pool).await?;
        q = q.bind(limit).bind(offset);
        let rows = q.fetch_all(&self.pool).await?;
        Ok((rows, total))
    }

    pub async fn update_research_source(
        &self,
        id: i64,
        title: Option<&str>,
        author: Option<&str>,
        publication: Option<&str>,
        date: Option<&str>,
        source_type: Option<&str>,
    ) -> Result<ResearchSourceRow, StorageError> {
        let existing = self
            .get_research_source(id)
            .await?
            .ok_or_else(|| StorageError::NotFound(format!("source {id} not found")))?;
        let new_title = title.unwrap_or(&existing.title).to_string();
        if new_title.trim().is_empty() {
            return Err(StorageError::Import("title must not be empty".into()));
        }
        let new_type = source_type.unwrap_or(&existing.r#type).to_string();
        let valid = [
            "BOOK",
            "REGISTER",
            "CENSUS",
            "CIVIL_RECORD",
            "PARISH_RECORD",
            "NEWSPAPER",
            "WEBSITE",
            "OTHER",
        ];
        if !valid.contains(&new_type.as_str()) {
            return Err(StorageError::Import(format!(
                "invalid source type {new_type}"
            )));
        }
        let new_author = author.or(existing.author.as_deref());
        let new_pub = publication.or(existing.publication.as_deref());
        let new_date = date.or(existing.date.as_deref());
        let now = crate::models::now_iso();
        sqlx::query(
            "UPDATE research_sources SET title=?1, author=?2, publication=?3, date=?4, type=?5, updated_at=?6 WHERE id=?7",
        )
        .bind(&new_title)
        .bind(new_author)
        .bind(new_pub)
        .bind(new_date)
        .bind(&new_type)
        .bind(&now)
        .bind(id)
        .execute(&self.pool)
        .await?;
        let row = self.get_research_source(id).await?.unwrap();
        Ok(row)
    }

    pub async fn delete_research_source(&self, id: i64) -> Result<(), StorageError> {
        sqlx::query("DELETE FROM research_sources WHERE id=?1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    // --- Research Citations ---
    pub async fn create_research_citation(
        &self,
        source_id: i64,
        locator: Option<&str>,
        text: Option<&str>,
    ) -> Result<ResearchCitationRow, StorageError> {
        let source = self
            .get_research_source(source_id)
            .await?
            .ok_or_else(|| StorageError::NotFound(format!("source {source_id} not found")))?;
        let _ = source;
        let now = crate::models::now_iso();
        let res = sqlx::query(
            "INSERT INTO research_citations (source_id, locator, text, created_at, updated_at) VALUES (?1,?2,?3,?4,?5)",
        )
        .bind(source_id)
        .bind(locator)
        .bind(text)
        .bind(&now)
        .bind(&now)
        .execute(&self.pool)
        .await?;
        let id = res.last_insert_rowid();
        let row = sqlx::query_as::<_, ResearchCitationRow>(
            "SELECT * FROM research_citations WHERE id=?1",
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await?;
        Ok(row)
    }

    pub async fn get_research_citation(
        &self,
        id: i64,
    ) -> Result<Option<ResearchCitationRow>, StorageError> {
        let row = sqlx::query_as::<_, ResearchCitationRow>(
            "SELECT * FROM research_citations WHERE id=?1",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row)
    }

    pub async fn list_research_citations(
        &self,
        source_id: i64,
        limit: i64,
        offset: i64,
    ) -> Result<(Vec<ResearchCitationRow>, i64), StorageError> {
        let total: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM research_citations WHERE source_id=?1")
                .bind(source_id)
                .fetch_one(&self.pool)
                .await?;
        let rows = sqlx::query_as::<_, ResearchCitationRow>(
            "SELECT * FROM research_citations WHERE source_id=?1 ORDER BY updated_at DESC LIMIT ?2 OFFSET ?3",
        )
        .bind(source_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;
        Ok((rows, total))
    }

    pub async fn update_research_citation(
        &self,
        id: i64,
        locator: Option<&str>,
        text: Option<&str>,
    ) -> Result<ResearchCitationRow, StorageError> {
        let existing = self
            .get_research_citation(id)
            .await?
            .ok_or_else(|| StorageError::NotFound(format!("citation {id} not found")))?;
        let new_locator = locator.or(existing.locator.as_deref());
        let new_text = text.or(existing.text.as_deref());
        let now = crate::models::now_iso();
        sqlx::query("UPDATE research_citations SET locator=?1, text=?2, updated_at=?3 WHERE id=?4")
            .bind(new_locator)
            .bind(new_text)
            .bind(&now)
            .bind(id)
            .execute(&self.pool)
            .await?;
        let row = self.get_research_citation(id).await?.unwrap();
        Ok(row)
    }

    pub async fn delete_research_citation(&self, id: i64) -> Result<(), StorageError> {
        sqlx::query("DELETE FROM research_citations WHERE id=?1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    // --- Evidence ---
    pub async fn create_evidence(
        &self,
        tree_id: i64,
        source_id: i64,
        citation_id: Option<i64>,
        statement: &str,
        notes: Option<&str>,
    ) -> Result<EvidenceRow, StorageError> {
        if statement.trim().is_empty() {
            return Err(StorageError::Import("statement must not be empty".into()));
        }
        let source = self
            .get_research_source(source_id)
            .await?
            .ok_or_else(|| StorageError::NotFound(format!("source {source_id} not found")))?;
        if source.tree_id != tree_id {
            return Err(StorageError::NotFound(format!(
                "source {source_id} not in tree {tree_id}"
            )));
        }
        if let Some(cid) = citation_id {
            let cit = self
                .get_research_citation(cid)
                .await?
                .ok_or_else(|| StorageError::NotFound(format!("citation {cid} not found")))?;
            if cit.source_id != source_id {
                return Err(StorageError::Import(
                    "citation does not belong to source".into(),
                ));
            }
        }
        let now = crate::models::now_iso();
        let res = sqlx::query(
            "INSERT INTO evidence (tree_id, source_id, citation_id, statement, notes, created_at, updated_at) VALUES (?1,?2,?3,?4,?5,?6,?7)",
        )
        .bind(tree_id)
        .bind(source_id)
        .bind(citation_id)
        .bind(statement)
        .bind(notes)
        .bind(&now)
        .bind(&now)
        .execute(&self.pool)
        .await?;
        let id = res.last_insert_rowid();
        let row = sqlx::query_as::<_, EvidenceRow>("SELECT * FROM evidence WHERE id=?1")
            .bind(id)
            .fetch_one(&self.pool)
            .await?;
        Ok(row)
    }

    pub async fn get_evidence(&self, id: i64) -> Result<Option<EvidenceRow>, StorageError> {
        let row = sqlx::query_as::<_, EvidenceRow>("SELECT * FROM evidence WHERE id=?1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await?;
        Ok(row)
    }

    pub async fn list_evidence(
        &self,
        tree_id: i64,
        limit: i64,
        offset: i64,
    ) -> Result<(Vec<EvidenceRow>, i64), StorageError> {
        let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM evidence WHERE tree_id=?1")
            .bind(tree_id)
            .fetch_one(&self.pool)
            .await?;
        let rows = sqlx::query_as::<_, EvidenceRow>(
            "SELECT * FROM evidence WHERE tree_id=?1 ORDER BY updated_at DESC LIMIT ?2 OFFSET ?3",
        )
        .bind(tree_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;
        Ok((rows, total))
    }

    pub async fn update_evidence(
        &self,
        id: i64,
        statement: Option<&str>,
        notes: Option<&str>,
        citation_id: Option<Option<i64>>,
    ) -> Result<EvidenceRow, StorageError> {
        let existing = self
            .get_evidence(id)
            .await?
            .ok_or_else(|| StorageError::NotFound(format!("evidence {id} not found")))?;
        let new_statement = statement.unwrap_or(&existing.statement).to_string();
        if new_statement.trim().is_empty() {
            return Err(StorageError::Import("statement must not be empty".into()));
        }
        let new_notes = notes.or(existing.notes.as_deref());
        let new_citation = match citation_id {
            Some(inner) => inner,
            None => existing.citation_id,
        };
        if let Some(cid) = new_citation {
            let cit = self
                .get_research_citation(cid)
                .await?
                .ok_or_else(|| StorageError::NotFound(format!("citation {cid} not found")))?;
            if cit.source_id != existing.source_id {
                return Err(StorageError::Import(
                    "citation does not belong to source".into(),
                ));
            }
        }
        let now = crate::models::now_iso();
        sqlx::query(
            "UPDATE evidence SET statement=?1, notes=?2, citation_id=?3, updated_at=?4 WHERE id=?5",
        )
        .bind(&new_statement)
        .bind(new_notes)
        .bind(new_citation)
        .bind(&now)
        .bind(id)
        .execute(&self.pool)
        .await?;
        let row = self.get_evidence(id).await?.unwrap();
        Ok(row)
    }

    pub async fn delete_evidence(&self, id: i64) -> Result<(), StorageError> {
        sqlx::query("DELETE FROM evidence WHERE id=?1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    // --- OutcomeEvidence ---
    pub async fn attach_evidence_to_outcome(
        &self,
        outcome_id: i64,
        evidence_id: i64,
        relationship: &str,
    ) -> Result<OutcomeEvidenceRow, StorageError> {
        let valid = ["SUPPORTS", "CONTRADICTS"];
        if !valid.contains(&relationship) {
            return Err(StorageError::Import(format!(
                "invalid relationship {relationship}"
            )));
        }
        let outcome = self
            .get_research_outcome(outcome_id)
            .await?
            .ok_or_else(|| StorageError::NotFound(format!("outcome {outcome_id} not found")))?;
        let evidence = self
            .get_evidence(evidence_id)
            .await?
            .ok_or_else(|| StorageError::NotFound(format!("evidence {evidence_id} not found")))?;
        if outcome.tree_id != evidence.tree_id {
            return Err(StorageError::NotFound(format!(
                "evidence {evidence_id} not in same tree as outcome {outcome_id}"
            )));
        }
        let existing: Option<i64> = sqlx::query_scalar(
            "SELECT outcome_id FROM outcome_evidence WHERE outcome_id=?1 AND evidence_id=?2",
        )
        .bind(outcome_id)
        .bind(evidence_id)
        .fetch_optional(&self.pool)
        .await?
        .flatten();
        if existing.is_some() {
            return Err(StorageError::Import("evidence already attached".into()));
        }
        sqlx::query("INSERT INTO outcome_evidence (outcome_id, evidence_id, relationship) VALUES (?1,?2,?3)")
            .bind(outcome_id)
            .bind(evidence_id)
            .bind(relationship)
            .execute(&self.pool)
            .await?;
        let row = sqlx::query_as::<_, OutcomeEvidenceRow>(
            "SELECT * FROM outcome_evidence WHERE outcome_id=?1 AND evidence_id=?2",
        )
        .bind(outcome_id)
        .bind(evidence_id)
        .fetch_one(&self.pool)
        .await?;
        Ok(row)
    }

    pub async fn detach_evidence_from_outcome(
        &self,
        outcome_id: i64,
        evidence_id: i64,
    ) -> Result<(), StorageError> {
        sqlx::query("DELETE FROM outcome_evidence WHERE outcome_id=?1 AND evidence_id=?2")
            .bind(outcome_id)
            .bind(evidence_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn list_outcome_evidence(
        &self,
        outcome_id: i64,
    ) -> Result<Vec<OutcomeEvidenceRow>, StorageError> {
        let rows = sqlx::query_as::<_, OutcomeEvidenceRow>(
            "SELECT * FROM outcome_evidence WHERE outcome_id=?1",
        )
        .bind(outcome_id)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows)
    }

    pub async fn list_outcome_evidence_detailed(
        &self,
        outcome_id: i64,
    ) -> Result<Vec<serde_json::Value>, StorageError> {
        let links = self.list_outcome_evidence(outcome_id).await?;
        if links.is_empty() {
            return Ok(vec![]);
        }
        let mut result = Vec::new();
        for link in links {
            let evidence = self.get_evidence(link.evidence_id).await?.unwrap();
            let source = self.get_research_source(evidence.source_id).await?.unwrap();
            let citation = if let Some(cid) = evidence.citation_id {
                self.get_research_citation(cid).await?
            } else {
                None
            };
            result.push(serde_json::json!({
                "id": evidence.id,
                "relationship": link.relationship,
                "statement": evidence.statement,
                "notes": evidence.notes,
                "source": {
                    "id": source.id,
                    "title": source.title,
                    "type": source.r#type,
                    "author": source.author,
                    "publication": source.publication,
                    "date": source.date
                },
                "citation": citation.map(|c| serde_json::json!({
                    "id": c.id,
                    "locator": c.locator,
                    "text": c.text
                })),
                "created_at": evidence.created_at,
                "updated_at": evidence.updated_at
            }));
        }
        Ok(result)
    }

    pub async fn get_outcome_evidence_stats(
        &self,
        outcome_id: i64,
    ) -> Result<crate::assessment::EvidenceStats, StorageError> {
        #[derive(sqlx::FromRow)]
        struct Row {
            evidence_total: i64,
            supporting: Option<i64>,
            contradicting: Option<i64>,
            sources: Option<i64>,
            cited: Option<i64>,
            uncited: Option<i64>,
            cited_supporting: Option<i64>,
        }
        let row = sqlx::query_as::<_, Row>(
            "SELECT COUNT(*) as evidence_total,
                    COALESCE(SUM(CASE WHEN oe.relationship='SUPPORTS' THEN 1 ELSE 0 END),0) as supporting,
                    COALESCE(SUM(CASE WHEN oe.relationship='CONTRADICTS' THEN 1 ELSE 0 END),0) as contradicting,
                    COALESCE(COUNT(DISTINCT e.source_id),0) as sources,
                    COALESCE(SUM(CASE WHEN e.citation_id IS NOT NULL THEN 1 ELSE 0 END),0) as cited,
                    COALESCE(SUM(CASE WHEN e.citation_id IS NULL THEN 1 ELSE 0 END),0) as uncited,
                    COALESCE(SUM(CASE WHEN oe.relationship='SUPPORTS' AND e.citation_id IS NOT NULL THEN 1 ELSE 0 END),0) as cited_supporting
             FROM outcome_evidence oe
             JOIN evidence e ON e.id = oe.evidence_id
             WHERE oe.outcome_id = ?1",
        )
        .bind(outcome_id)
        .fetch_one(&self.pool)
        .await?;
        Ok(crate::assessment::EvidenceStats {
            evidence_total: row.evidence_total,
            supporting_count: row.supporting.unwrap_or(0),
            contradicting_count: row.contradicting.unwrap_or(0),
            sources_count: row.sources.unwrap_or(0),
            cited_count: row.cited.unwrap_or(0),
            uncited_count: row.uncited.unwrap_or(0),
            cited_supporting_count: row.cited_supporting.unwrap_or(0),
        })
    }

    pub async fn get_outcomes_evidence_stats(
        &self,
        outcome_ids: &[i64],
    ) -> Result<std::collections::HashMap<i64, crate::assessment::EvidenceStats>, StorageError>
    {
        if outcome_ids.is_empty() {
            return Ok(std::collections::HashMap::new());
        }
        #[derive(sqlx::FromRow)]
        struct Row {
            outcome_id: i64,
            evidence_total: i64,
            supporting: Option<i64>,
            contradicting: Option<i64>,
            sources: Option<i64>,
            cited: Option<i64>,
            uncited: Option<i64>,
            cited_supporting: Option<i64>,
        }
        let placeholders = outcome_ids
            .iter()
            .map(|_| "?")
            .collect::<Vec<_>>()
            .join(",");
        let sql = format!(
            "SELECT oe.outcome_id as outcome_id,
                    COUNT(*) as evidence_total,
                    COALESCE(SUM(CASE WHEN oe.relationship='SUPPORTS' THEN 1 ELSE 0 END),0) as supporting,
                    COALESCE(SUM(CASE WHEN oe.relationship='CONTRADICTS' THEN 1 ELSE 0 END),0) as contradicting,
                    COALESCE(COUNT(DISTINCT e.source_id),0) as sources,
                    COALESCE(SUM(CASE WHEN e.citation_id IS NOT NULL THEN 1 ELSE 0 END),0) as cited,
                    COALESCE(SUM(CASE WHEN e.citation_id IS NULL THEN 1 ELSE 0 END),0) as uncited,
                    COALESCE(SUM(CASE WHEN oe.relationship='SUPPORTS' AND e.citation_id IS NOT NULL THEN 1 ELSE 0 END),0) as cited_supporting
             FROM outcome_evidence oe
             JOIN evidence e ON e.id = oe.evidence_id
             WHERE oe.outcome_id IN ({placeholders})
             GROUP BY oe.outcome_id"
        );
        let mut q = sqlx::query_as::<_, Row>(&sql);
        for id in outcome_ids {
            q = q.bind(id);
        }
        let rows = q.fetch_all(&self.pool).await?;
        let mut map: std::collections::HashMap<i64, crate::assessment::EvidenceStats> =
            std::collections::HashMap::new();
        for r in rows {
            map.insert(
                r.outcome_id,
                crate::assessment::EvidenceStats {
                    evidence_total: r.evidence_total,
                    supporting_count: r.supporting.unwrap_or(0),
                    contradicting_count: r.contradicting.unwrap_or(0),
                    sources_count: r.sources.unwrap_or(0),
                    cited_count: r.cited.unwrap_or(0),
                    uncited_count: r.uncited.unwrap_or(0),
                    cited_supporting_count: r.cited_supporting.unwrap_or(0),
                },
            );
        }
        // fill missing with zero stats
        for id in outcome_ids {
            map.entry(*id).or_insert(crate::assessment::EvidenceStats {
                evidence_total: 0,
                supporting_count: 0,
                contradicting_count: 0,
                sources_count: 0,
                cited_count: 0,
                uncited_count: 0,
                cited_supporting_count: 0,
            });
        }
        Ok(map)
    }

    pub async fn get_outcome_assessment(
        &self,
        outcome_id: i64,
    ) -> Result<crate::assessment::EvidenceAssessment, StorageError> {
        let stats = self.get_outcome_evidence_stats(outcome_id).await?;
        Ok(crate::assessment::calculate_evidence_assessment(&stats))
    }

    pub async fn get_outcomes_assessments(
        &self,
        outcome_ids: &[i64],
    ) -> Result<std::collections::HashMap<i64, crate::assessment::EvidenceAssessment>, StorageError>
    {
        let stats_map = self.get_outcomes_evidence_stats(outcome_ids).await?;
        let mut res = std::collections::HashMap::new();
        for (id, stats) in stats_map {
            res.insert(id, crate::assessment::calculate_evidence_assessment(&stats));
        }
        Ok(res)
    }

    pub async fn get_outcome_gaps(
        &self,
        outcome_id: i64,
    ) -> Result<Vec<crate::assessment::EvidenceGap>, StorageError> {
        let outcome = self
            .get_research_outcome(outcome_id)
            .await?
            .ok_or_else(|| StorageError::NotFound(format!("outcome {outcome_id} not found")))?;
        let stats = self.get_outcome_evidence_stats(outcome_id).await?;
        Ok(crate::assessment::calculate_evidence_gaps(
            &outcome.r#type,
            &stats,
        ))
    }

    pub async fn get_outcomes_gaps(
        &self,
        outcome_ids: &[i64],
    ) -> Result<std::collections::HashMap<i64, Vec<crate::assessment::EvidenceGap>>, StorageError>
    {
        if outcome_ids.is_empty() {
            return Ok(std::collections::HashMap::new());
        }
        let stats_map = self.get_outcomes_evidence_stats(outcome_ids).await?;
        // need outcome types
        let placeholders = outcome_ids
            .iter()
            .map(|_| "?")
            .collect::<Vec<_>>()
            .join(",");
        let sql = format!("SELECT id, type FROM research_outcomes WHERE id IN ({placeholders})");
        let mut q = sqlx::query_as::<_, (i64, String)>(&sql);
        for id in outcome_ids {
            q = q.bind(id);
        }
        let rows: Vec<(i64, String)> = q.fetch_all(&self.pool).await.unwrap_or_default();
        let type_map: std::collections::HashMap<i64, String> = rows.into_iter().collect();
        let mut res = std::collections::HashMap::new();
        for id in outcome_ids {
            let t = type_map
                .get(id)
                .map(|s| s.as_str())
                .unwrap_or("INCONCLUSIVE");
            if let Some(stats) = stats_map.get(id) {
                res.insert(*id, crate::assessment::calculate_evidence_gaps(t, stats));
            } else {
                res.insert(
                    *id,
                    crate::assessment::calculate_evidence_gaps(
                        t,
                        &crate::assessment::EvidenceStats {
                            evidence_total: 0,
                            supporting_count: 0,
                            contradicting_count: 0,
                            sources_count: 0,
                            cited_count: 0,
                            uncited_count: 0,
                            cited_supporting_count: 0,
                        },
                    ),
                );
            }
        }
        Ok(res)
    }

    pub async fn get_outcome_followups(
        &self,
        outcome_id: i64,
    ) -> Result<Vec<crate::assessment::ResearchFollowUp>, StorageError> {
        let outcome = self
            .get_research_outcome(outcome_id)
            .await?
            .ok_or_else(|| StorageError::NotFound(format!("outcome {outcome_id} not found")))?;
        let stats = self.get_outcome_evidence_stats(outcome_id).await?;
        let gaps = crate::assessment::calculate_evidence_gaps(&outcome.r#type, &stats);
        Ok(crate::assessment::calculate_research_followups(
            &outcome.r#type,
            &stats,
            &gaps,
        ))
    }

    pub async fn get_outcomes_followups(
        &self,
        outcome_ids: &[i64],
    ) -> Result<
        std::collections::HashMap<i64, Vec<crate::assessment::ResearchFollowUp>>,
        StorageError,
    > {
        if outcome_ids.is_empty() {
            return Ok(std::collections::HashMap::new());
        }
        let stats_map = self.get_outcomes_evidence_stats(outcome_ids).await?;
        let gaps_map = self.get_outcomes_gaps(outcome_ids).await?;
        let mut res = std::collections::HashMap::new();
        // need types for consistency
        let placeholders = outcome_ids
            .iter()
            .map(|_| "?")
            .collect::<Vec<_>>()
            .join(",");
        let sql = format!("SELECT id, type FROM research_outcomes WHERE id IN ({placeholders})");
        let mut q = sqlx::query_as::<_, (i64, String)>(&sql);
        for id in outcome_ids {
            q = q.bind(id);
        }
        let rows: Vec<(i64, String)> = q.fetch_all(&self.pool).await.unwrap_or_default();
        let type_map: std::collections::HashMap<i64, String> = rows.into_iter().collect();
        for id in outcome_ids {
            let t = type_map
                .get(id)
                .map(|s| s.as_str())
                .unwrap_or("INCONCLUSIVE");
            let stats = stats_map
                .get(id)
                .cloned()
                .unwrap_or(crate::assessment::EvidenceStats {
                    evidence_total: 0,
                    supporting_count: 0,
                    contradicting_count: 0,
                    sources_count: 0,
                    cited_count: 0,
                    uncited_count: 0,
                    cited_supporting_count: 0,
                });
            let gaps = gaps_map.get(id).cloned().unwrap_or_default();
            res.insert(
                *id,
                crate::assessment::calculate_research_followups(t, &stats, &gaps),
            );
        }
        Ok(res)
    }

    // --- Followup Actions ---
    pub async fn create_followup_action(
        &self,
        tree_id: i64,
        task_id: i64,
        outcome_id: i64,
        followup_code: &str,
        notes: Option<&str>,
    ) -> Result<ResearchFollowupActionRow, StorageError> {
        let valid_codes = [
            "ADD_SUPPORTING_EVIDENCE",
            "ADD_CITATION",
            "REVIEW_CONTRADICTION",
            "ADD_SECOND_SUPPORTING_EVIDENCE",
            "REVIEW_SOURCE_COVERAGE",
        ];
        if !valid_codes.contains(&followup_code) {
            return Err(StorageError::Import(format!(
                "invalid followup_code {followup_code}"
            )));
        }
        // Validate outcome exists and belongs to tree and task
        let outcome = self
            .get_research_outcome(outcome_id)
            .await?
            .ok_or_else(|| StorageError::NotFound(format!("outcome {outcome_id} not found")))?;
        if outcome.tree_id != tree_id {
            return Err(StorageError::NotFound(format!(
                "outcome {outcome_id} not in tree {tree_id}"
            )));
        }
        if outcome.task_id != task_id {
            return Err(StorageError::NotFound(format!(
                "task {task_id} does not match outcome task {}",
                outcome.task_id
            )));
        }
        let task = self
            .get_research_task(task_id)
            .await?
            .ok_or_else(|| StorageError::NotFound(format!("task {task_id} not found")))?;
        if task.tree_id != tree_id {
            return Err(StorageError::NotFound(format!(
                "task {task_id} not in tree {tree_id}"
            )));
        }
        let now = crate::models::now_iso();
        let res = sqlx::query(
            "INSERT INTO research_followup_actions (tree_id, task_id, outcome_id, followup_code, status, notes, created_at, updated_at, completed_at) VALUES (?1,?2,?3,?4,'OPEN',?5,?6,?7,NULL)",
        )
        .bind(tree_id)
        .bind(task_id)
        .bind(outcome_id)
        .bind(followup_code)
        .bind(notes)
        .bind(&now)
        .bind(&now)
        .execute(&self.pool)
        .await?;
        let id = res.last_insert_rowid();
        let row = sqlx::query_as::<_, ResearchFollowupActionRow>(
            "SELECT * FROM research_followup_actions WHERE id=?1",
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await?;
        Ok(row)
    }

    pub async fn get_followup_action(
        &self,
        id: i64,
    ) -> Result<Option<ResearchFollowupActionRow>, StorageError> {
        let row = sqlx::query_as::<_, ResearchFollowupActionRow>(
            "SELECT * FROM research_followup_actions WHERE id=?1",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row)
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn list_followup_actions(
        &self,
        tree_id: i64,
        task_id: Option<i64>,
        outcome_id: Option<i64>,
        status: Option<&str>,
        followup_code: Option<&str>,
        limit: i64,
        offset: i64,
    ) -> Result<(Vec<ResearchFollowupActionRow>, i64), StorageError> {
        let mut sql = "SELECT * FROM research_followup_actions WHERE tree_id = ?".to_string();
        let mut count_sql =
            "SELECT COUNT(*) FROM research_followup_actions WHERE tree_id = ?".to_string();
        if task_id.is_some() {
            sql.push_str(" AND task_id = ?");
            count_sql.push_str(" AND task_id = ?");
        }
        if outcome_id.is_some() {
            sql.push_str(" AND outcome_id = ?");
            count_sql.push_str(" AND outcome_id = ?");
        }
        if status.is_some() {
            sql.push_str(" AND status = ?");
            count_sql.push_str(" AND status = ?");
        }
        if followup_code.is_some() {
            sql.push_str(" AND followup_code = ?");
            count_sql.push_str(" AND followup_code = ?");
        }
        sql.push_str(" ORDER BY updated_at DESC LIMIT ? OFFSET ?");
        let mut cq = sqlx::query_scalar::<_, i64>(&count_sql).bind(tree_id);
        let mut q = sqlx::query_as::<_, ResearchFollowupActionRow>(&sql).bind(tree_id);
        if let Some(tid) = task_id {
            cq = cq.bind(tid);
            q = q.bind(tid);
        }
        if let Some(oid) = outcome_id {
            cq = cq.bind(oid);
            q = q.bind(oid);
        }
        if let Some(s) = status {
            cq = cq.bind(s);
            q = q.bind(s);
        }
        if let Some(c) = followup_code {
            cq = cq.bind(c);
            q = q.bind(c);
        }
        let total = cq.fetch_one(&self.pool).await?;
        q = q.bind(limit).bind(offset);
        let rows = q.fetch_all(&self.pool).await?;
        Ok((rows, total))
    }

    pub async fn list_task_followup_actions(
        &self,
        task_id: i64,
        limit: i64,
        offset: i64,
    ) -> Result<(Vec<ResearchFollowupActionRow>, i64), StorageError> {
        let total: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM research_followup_actions WHERE task_id=?1")
                .bind(task_id)
                .fetch_one(&self.pool)
                .await?;
        let rows = sqlx::query_as::<_, ResearchFollowupActionRow>(
            "SELECT * FROM research_followup_actions WHERE task_id=?1 ORDER BY updated_at DESC LIMIT ?2 OFFSET ?3",
        )
        .bind(task_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;
        Ok((rows, total))
    }

    pub async fn list_outcome_followup_actions(
        &self,
        outcome_id: i64,
    ) -> Result<Vec<ResearchFollowupActionRow>, StorageError> {
        let rows = sqlx::query_as::<_, ResearchFollowupActionRow>(
            "SELECT * FROM research_followup_actions WHERE outcome_id=?1 ORDER BY updated_at DESC",
        )
        .bind(outcome_id)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows)
    }

    pub async fn get_outcomes_followup_actions_counts(
        &self,
        outcome_ids: &[i64],
    ) -> Result<std::collections::HashMap<i64, i64>, StorageError> {
        if outcome_ids.is_empty() {
            return Ok(std::collections::HashMap::new());
        }
        let placeholders = outcome_ids
            .iter()
            .map(|_| "?")
            .collect::<Vec<_>>()
            .join(",");
        let sql = format!(
            "SELECT outcome_id, COUNT(*) as cnt FROM research_followup_actions WHERE outcome_id IN ({placeholders}) GROUP BY outcome_id"
        );
        let mut q = sqlx::query_as::<_, (i64, i64)>(&sql);
        for id in outcome_ids {
            q = q.bind(id);
        }
        let rows: Vec<(i64, i64)> = q.fetch_all(&self.pool).await?;
        let mut map: std::collections::HashMap<i64, i64> = std::collections::HashMap::new();
        for (oid, cnt) in rows {
            map.insert(oid, cnt);
        }
        for id in outcome_ids {
            map.entry(*id).or_insert(0);
        }
        Ok(map)
    }

    pub async fn update_followup_action(
        &self,
        id: i64,
        status: Option<&str>,
        notes: Option<Option<&str>>,
    ) -> Result<ResearchFollowupActionRow, StorageError> {
        let existing = self
            .get_followup_action(id)
            .await?
            .ok_or_else(|| StorageError::NotFound(format!("followup_action {id} not found")))?;
        let new_status = status.unwrap_or(&existing.status).to_string();
        let valid = ["OPEN", "COMPLETED", "SKIPPED"];
        if !valid.contains(&new_status.as_str()) {
            return Err(StorageError::Import(format!("invalid status {new_status}")));
        }
        // notes handling: Some(Some(v)) => set, Some(None) => set null, None => keep existing
        let new_notes = match notes {
            Some(inner) => inner.map(|s| s.to_string()),
            None => existing.notes.clone(),
        };
        let now = crate::models::now_iso();
        let completed_at = match new_status.as_str() {
            "COMPLETED" | "SKIPPED" => {
                if existing.completed_at.is_some() && existing.status == new_status {
                    existing.completed_at.clone()
                } else {
                    Some(now.clone())
                }
            }
            "OPEN" => None,
            _ => existing.completed_at.clone(),
        };
        // if status stays same but was already completed, keep original completed_at unless transitioning to OPEN
        // above logic handles: if already completed and same status, keep
        sqlx::query(
            "UPDATE research_followup_actions SET status=?1, notes=?2, updated_at=?3, completed_at=?4 WHERE id=?5",
        )
        .bind(&new_status)
        .bind(&new_notes)
        .bind(&now)
        .bind(&completed_at)
        .bind(id)
        .execute(&self.pool)
        .await?;
        let row = self.get_followup_action(id).await?.unwrap();
        Ok(row)
    }

    pub async fn delete_followup_action(&self, id: i64) -> Result<(), StorageError> {
        sqlx::query("DELETE FROM research_followup_actions WHERE id=?1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn count_followup_actions_by_status(
        &self,
        tree_id: i64,
    ) -> Result<std::collections::HashMap<String, i64>, StorageError> {
        let rows: Vec<(String, i64)> = sqlx::query_as(
            "SELECT status, COUNT(*) FROM research_followup_actions WHERE tree_id=?1 GROUP BY status",
        )
        .bind(tree_id)
        .fetch_all(&self.pool)
        .await?;
        let mut map = std::collections::HashMap::new();
        for (s, c) in rows {
            map.insert(s, c);
        }
        Ok(map)
    }

    pub async fn get_research_case_summary(
        &self,
        tree_id: i64,
        task_id: i64,
    ) -> Result<crate::case_summary::ResearchCaseSummary, StorageError> {
        // Fetch task with tree isolation
        let task = self
            .get_research_task(task_id)
            .await?
            .ok_or_else(|| StorageError::NotFound(format!("task {task_id} not found")))?;
        if task.tree_id != tree_id {
            return Err(StorageError::NotFound(format!(
                "task {task_id} not in tree {tree_id}"
            )));
        }

        // Person (batch single query)
        let person = if let Some(pid) = task.person_id {
            let p =
                sqlx::query_as::<_, PersonRow>("SELECT * FROM persons WHERE id=?1 AND tree_id=?2")
                    .bind(pid)
                    .bind(tree_id)
                    .fetch_optional(&self.pool)
                    .await?;
            p.map(|row| crate::case_summary::CaseSummaryPerson {
                person_id: row.id,
                person_name: row.display_name.clone().unwrap_or_else(|| {
                    let gn = row.given_name.clone().unwrap_or_default();
                    let sn = row.surname.clone().unwrap_or_default();
                    let combined = format!("{} {}", gn, sn).trim().to_string();
                    if combined.is_empty() {
                        row.gedcom_id.clone()
                    } else {
                        combined
                    }
                }),
            })
        } else {
            None
        };

        // Opportunity (single query)
        let opportunity = if let Some(oid) = task.opportunity_id {
            let opp = sqlx::query_as::<_, ResearchOpportunityRow>(
                "SELECT * FROM research_opportunities WHERE id=?1 AND tree_id=?2",
            )
            .bind(oid)
            .bind(tree_id)
            .fetch_optional(&self.pool)
            .await?;
            opp.map(|o| crate::case_summary::CaseSummaryOpportunity {
                opportunity_id: o.id,
                score: o.score,
                priority: o.priority.clone(),
                researchability: o.researchability.clone(),
                confidence: o.confidence,
                title: o
                    .why
                    .clone()
                    .or_else(|| o.what.clone())
                    .or(Some(format!("Opportunity {}", o.id))),
            })
        } else {
            None
        };

        // Outcome (single query)
        let outcome_row = self.get_research_outcome_by_task(task_id).await?;
        let outcome_dto = outcome_row
            .as_ref()
            .map(|o| crate::case_summary::CaseSummaryOutcome {
                outcome_id: o.id,
                r#type: o.r#type.clone(),
                summary: o.summary.clone(),
                details: o.details.clone(),
                created_at: o.created_at.clone(),
                updated_at: o.updated_at.clone(),
            });

        // Evidence assessment / gaps / followups (reuse existing pure functions, single stats query if outcome exists)
        let (assessment, gaps, followups) = if let Some(ref o) = outcome_row {
            let stats = self.get_outcome_evidence_stats(o.id).await?;
            let ass = crate::assessment::calculate_evidence_assessment(&stats);
            let g = crate::assessment::calculate_evidence_gaps(&o.r#type, &stats);
            let fu = crate::assessment::calculate_research_followups(&o.r#type, &stats, &g);
            (Some(ass), g, fu)
        } else {
            (None, Vec::new(), Vec::new())
        };

        // Follow-up actions (single query, no N+1)
        let followup_actions = sqlx::query_as::<_, ResearchFollowupActionRow>(
            "SELECT * FROM research_followup_actions WHERE task_id=?1 AND tree_id=?2 ORDER BY updated_at DESC",
        )
        .bind(task_id)
        .bind(tree_id)
        .fetch_all(&self.pool)
        .await?;

        // Timeline (derived, no extra table)
        let timeline =
            crate::case_summary::build_timeline(&task, outcome_row.as_ref(), &followup_actions);

        // Closure warnings (pure)
        let warnings = crate::case_summary::calculate_closure_warnings(
            &task.status,
            outcome_row.as_ref().map(|o| o.r#type.as_str()),
            assessment.as_ref().map(|a| a.status.as_str()),
            &gaps,
        );

        let task_dto = crate::case_summary::CaseSummaryTask {
            id: task.id,
            title: task.title.clone(),
            description: task.description.clone(),
            status: task.status.clone(),
            resolution: task.resolution.clone(),
            created_at: task.created_at.clone(),
            started_at: task.started_at.clone(),
            completed_at: task.completed_at.clone(),
            updated_at: task.updated_at.clone(),
        };

        Ok(crate::case_summary::ResearchCaseSummary {
            task: task_dto,
            person,
            opportunity,
            outcome: outcome_dto,
            evidence_assessment: assessment,
            evidence_gaps: gaps,
            research_followups: followups,
            followup_actions,
            timeline,
            closure_warnings: warnings,
        })
    }

    pub async fn get_research_planning_candidates(
        &self,
        tree_id: i64,
    ) -> Result<Vec<crate::planning::PlanningCandidate>, StorageError> {
        // 1 query: all opportunities for tree (O(1))
        let opps = sqlx::query_as::<_, ResearchOpportunityRow>(
            "SELECT * FROM research_opportunities WHERE tree_id=?1 ORDER BY id",
        )
        .bind(tree_id)
        .fetch_all(&self.pool)
        .await?;
        if opps.is_empty() {
            return Ok(Vec::new());
        }
        let opp_ids: Vec<i64> = opps.iter().map(|o| o.id).collect();

        // 2 query: all tasks for those opportunities in this tree (single query, no N+1)
        // Pick latest task per opportunity (max id)
        let placeholders = opp_ids.iter().map(|_| "?").collect::<Vec<_>>().join(",");
        let sql = format!(
            "SELECT * FROM research_tasks WHERE tree_id=? AND opportunity_id IN ({placeholders}) ORDER BY id DESC"
        );
        let mut q = sqlx::query_as::<_, ResearchTaskRow>(&sql).bind(tree_id);
        for oid in &opp_ids {
            q = q.bind(oid);
        }
        let tasks = q.fetch_all(&self.pool).await?;
        use std::collections::HashMap;
        let mut opp_to_task: HashMap<i64, ResearchTaskRow> = HashMap::new();
        for t in tasks {
            if let Some(oid) = t.opportunity_id {
                opp_to_task.entry(oid).or_insert(t);
            }
        }
        let task_ids: Vec<i64> = opp_to_task.values().map(|t| t.id).collect();

        // 3 query: outcomes for those tasks (single query)
        let mut task_to_outcome: HashMap<i64, ResearchOutcomeRow> = HashMap::new();
        let mut outcome_ids: Vec<i64> = Vec::new();
        if !task_ids.is_empty() {
            let placeholders2 = task_ids.iter().map(|_| "?").collect::<Vec<_>>().join(",");
            let sql2 = format!(
                "SELECT * FROM research_outcomes WHERE tree_id=? AND task_id IN ({placeholders2})"
            );
            let mut q2 = sqlx::query_as::<_, ResearchOutcomeRow>(&sql2).bind(tree_id);
            for tid in &task_ids {
                q2 = q2.bind(tid);
            }
            let outcomes = q2.fetch_all(&self.pool).await?;
            for o in outcomes {
                outcome_ids.push(o.id);
                task_to_outcome.insert(o.task_id, o);
            }
        }

        // 4: gaps for outcomes (batch, O(1) – uses 2 queries internally but not N)
        let gaps_map = if outcome_ids.is_empty() {
            HashMap::new()
        } else {
            self.get_outcomes_gaps(&outcome_ids).await?
        };

        // Build candidates
        let mut candidates = Vec::with_capacity(opps.len());
        for opp in opps {
            let title = opp
                .why
                .clone()
                .or_else(|| opp.what.clone())
                .unwrap_or_else(|| format!("Research opportunity {}", opp.id));
            // Trim JSON if what is JSON string – but why is plain string, use it
            let gaps = if let Some(task) = opp_to_task.get(&opp.id) {
                if let Some(outcome) = task_to_outcome.get(&task.id) {
                    gaps_map.get(&outcome.id).cloned().unwrap_or_default()
                } else {
                    Vec::new()
                }
            } else {
                Vec::new()
            };
            let task_status = opp_to_task.get(&opp.id).map(|t| t.status.clone());
            candidates.push(crate::planning::PlanningCandidate {
                opportunity_id: opp.id,
                person_id: opp.person_id,
                title,
                priority: opp.priority.clone().unwrap_or_else(|| "low".to_string()),
                research_score: opp.score.unwrap_or(0),
                researchability: opp.researchability.clone(),
                confidence: opp.confidence,
                task_status,
                gaps,
            });
        }
        Ok(candidates)
    }

    // --- Research Sessions ---
    fn validate_session_status(status: &str) -> Result<(), StorageError> {
        let allowed = ["PLANNED", "ACTIVE", "COMPLETED", "ABANDONED"];
        if !allowed.contains(&status) {
            return Err(StorageError::Import(format!(
                "invalid session status {status}"
            )));
        }
        Ok(())
    }

    pub async fn create_research_session(
        &self,
        tree_id: i64,
        title: &str,
        description: Option<&str>,
        person_id: Option<i64>,
        opportunity_id: Option<i64>,
    ) -> Result<ResearchSessionRow, StorageError> {
        if title.trim().is_empty() {
            return Err(StorageError::Import("title must not be empty".into()));
        }
        // validate tree
        let tree = self.get_tree(tree_id).await?;
        if tree.is_none() {
            return Err(StorageError::NotFound(format!("tree {tree_id} not found")));
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
        if let Some(oid) = opportunity_id {
            let o_tree: Option<i64> =
                sqlx::query_scalar("SELECT tree_id FROM research_opportunities WHERE id=?1")
                    .bind(oid)
                    .fetch_optional(&self.pool)
                    .await?
                    .flatten();
            if o_tree != Some(tree_id) {
                return Err(StorageError::NotFound(format!(
                    "opportunity {oid} not in tree {tree_id}"
                )));
            }
        }
        let now = crate::models::now_iso();
        let res = sqlx::query(
            "INSERT INTO research_sessions (tree_id, title, description, status, person_id, opportunity_id, created_at, updated_at, started_at, completed_at) VALUES (?1,?2,?3,'PLANNED',?4,?5,?6,?7,NULL,NULL)",
        )
        .bind(tree_id)
        .bind(title)
        .bind(description)
        .bind(person_id)
        .bind(opportunity_id)
        .bind(&now)
        .bind(&now)
        .execute(&self.pool)
        .await?;
        let id = res.last_insert_rowid();
        let row =
            sqlx::query_as::<_, ResearchSessionRow>("SELECT * FROM research_sessions WHERE id=?1")
                .bind(id)
                .fetch_one(&self.pool)
                .await?;
        Ok(row)
    }

    pub async fn get_research_session(
        &self,
        id: i64,
    ) -> Result<Option<ResearchSessionRow>, StorageError> {
        let row =
            sqlx::query_as::<_, ResearchSessionRow>("SELECT * FROM research_sessions WHERE id=?1")
                .bind(id)
                .fetch_optional(&self.pool)
                .await?;
        Ok(row)
    }

    pub async fn list_research_sessions(
        &self,
        tree_id: i64,
        status: Option<&str>,
        person_id: Option<i64>,
        opportunity_id: Option<i64>,
        limit: i64,
        offset: i64,
    ) -> Result<(Vec<ResearchSessionRow>, i64), StorageError> {
        let mut sql = "SELECT * FROM research_sessions WHERE tree_id = ?".to_string();
        let mut count_sql = "SELECT COUNT(*) FROM research_sessions WHERE tree_id = ?".to_string();
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
        sql.push_str(" ORDER BY CASE status WHEN 'ACTIVE' THEN 0 WHEN 'PLANNED' THEN 1 WHEN 'COMPLETED' THEN 2 WHEN 'ABANDONED' THEN 3 ELSE 4 END, updated_at DESC LIMIT ? OFFSET ?");
        let mut cq = sqlx::query_scalar::<_, i64>(&count_sql).bind(tree_id);
        let mut q = sqlx::query_as::<_, ResearchSessionRow>(&sql).bind(tree_id);
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

    pub async fn update_research_session(
        &self,
        id: i64,
        title: Option<&str>,
        description: Option<Option<&str>>,
        status: Option<&str>,
        person_id: Option<Option<i64>>,
        opportunity_id: Option<Option<i64>>,
    ) -> Result<ResearchSessionRow, StorageError> {
        let existing = self
            .get_research_session(id)
            .await?
            .ok_or_else(|| StorageError::NotFound(format!("session {id} not found")))?;
        let new_title = title.unwrap_or(&existing.title).to_string();
        if new_title.trim().is_empty() {
            return Err(StorageError::Import("title must not be empty".into()));
        }
        let new_status = status.unwrap_or(&existing.status).to_string();
        Self::validate_session_status(&new_status)?;
        let new_description = match description {
            Some(inner) => inner.map(|s| s.to_string()),
            None => existing.description.clone(),
        };
        let new_person_id = match person_id {
            Some(inner) => inner,
            None => existing.person_id,
        };
        let new_opportunity_id = match opportunity_id {
            Some(inner) => inner,
            None => existing.opportunity_id,
        };
        // validate person/opportunity tree isolation if changed
        if let Some(pid) = new_person_id {
            let p_tree: Option<i64> = sqlx::query_scalar("SELECT tree_id FROM persons WHERE id=?1")
                .bind(pid)
                .fetch_optional(&self.pool)
                .await?
                .flatten();
            if p_tree != Some(existing.tree_id) {
                return Err(StorageError::NotFound(format!(
                    "person {pid} not in tree {}",
                    existing.tree_id
                )));
            }
        }
        if let Some(oid) = new_opportunity_id {
            let o_tree: Option<i64> =
                sqlx::query_scalar("SELECT tree_id FROM research_opportunities WHERE id=?1")
                    .bind(oid)
                    .fetch_optional(&self.pool)
                    .await?
                    .flatten();
            if o_tree != Some(existing.tree_id) {
                return Err(StorageError::NotFound(format!(
                    "opportunity {oid} not in tree {}",
                    existing.tree_id
                )));
            }
        }
        let now = crate::models::now_iso();
        let mut started_at = existing.started_at.clone();
        let mut completed_at = existing.completed_at.clone();
        if new_status == "ACTIVE" && existing.status != "ACTIVE" {
            started_at = Some(now.clone());
            // when reopening, clear completed_at
            if ["COMPLETED", "ABANDONED"].contains(&existing.status.as_str()) {
                completed_at = None;
            }
        }
        if ["COMPLETED", "ABANDONED"].contains(&new_status.as_str()) && completed_at.is_none() {
            completed_at = Some(now.clone());
        }
        if new_status == "ACTIVE" && ["COMPLETED", "ABANDONED"].contains(&existing.status.as_str())
        {
            // reopening: clear completed_at already done, keep started_at as now?
            // keep started_at as existing or now? spec says completed_at = NULL when reopening
            // started_at remains
        }
        // if moving from ACTIVE to PLANNED etc, keep started_at? but spec only cares for ACTIVE->started_at and COMPLETED/ABANDONED -> completed_at
        // if reopening to ACTIVE, completed_at null already
        if new_status != "COMPLETED" && new_status != "ABANDONED" {
            // if status is not terminal, ensure completed_at is null unless previously terminal and now active (already cleared)
            // for PLANNED -> ACTIVE, keep completed_at null
            // for COMPLETED -> ACTIVE we cleared
            // for other transitions keep as is?
            // To satisfy spec: when reopens COMPLETED/ABANDONED → ACTIVE, completed_at = NULL
            if ["COMPLETED", "ABANDONED"].contains(&existing.status.as_str())
                && new_status == "ACTIVE"
            {
                completed_at = None;
            }
        }
        // If new status is PLANNED from COMPLETED/ABANDONED, should we clear completed_at? spec only mentions reopening to ACTIVE, but we treat generically: if leaving terminal to non-terminal, clear completed_at
        if ["COMPLETED", "ABANDONED"].contains(&existing.status.as_str())
            && !["COMPLETED", "ABANDONED"].contains(&new_status.as_str())
        {
            completed_at = None;
        }
        sqlx::query(
            "UPDATE research_sessions SET title=?1, description=?2, status=?3, person_id=?4, opportunity_id=?5, updated_at=?6, started_at=?7, completed_at=?8 WHERE id=?9",
        )
        .bind(&new_title)
        .bind(&new_description)
        .bind(&new_status)
        .bind(new_person_id)
        .bind(new_opportunity_id)
        .bind(&now)
        .bind(&started_at)
        .bind(&completed_at)
        .bind(id)
        .execute(&self.pool)
        .await?;
        let row = self.get_research_session(id).await?.unwrap();
        Ok(row)
    }

    pub async fn delete_research_session(&self, id: i64) -> Result<(), StorageError> {
        sqlx::query("DELETE FROM research_sessions WHERE id=?1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn assign_task_to_session(
        &self,
        task_id: i64,
        session_id: i64,
    ) -> Result<ResearchTaskRow, StorageError> {
        let task = self
            .get_research_task(task_id)
            .await?
            .ok_or_else(|| StorageError::NotFound(format!("task {task_id} not found")))?;
        let session = self
            .get_research_session(session_id)
            .await?
            .ok_or_else(|| StorageError::NotFound(format!("session {session_id} not found")))?;
        if task.tree_id != session.tree_id {
            return Err(StorageError::NotFound(format!(
                "task {task_id} not in same tree as session {session_id}"
            )));
        }
        // tree isolation already ensures same tree
        let now = crate::models::now_iso();
        sqlx::query("UPDATE research_tasks SET session_id=?1, updated_at=?2 WHERE id=?3")
            .bind(session_id)
            .bind(&now)
            .bind(task_id)
            .execute(&self.pool)
            .await?;
        let row = self.get_research_task(task_id).await?.unwrap();
        Ok(row)
    }

    pub async fn remove_task_from_session(
        &self,
        task_id: i64,
    ) -> Result<ResearchTaskRow, StorageError> {
        self.get_research_task(task_id)
            .await?
            .ok_or_else(|| StorageError::NotFound(format!("task {task_id} not found")))?;
        let now = crate::models::now_iso();
        sqlx::query("UPDATE research_tasks SET session_id=NULL, updated_at=?1 WHERE id=?2")
            .bind(&now)
            .bind(task_id)
            .execute(&self.pool)
            .await?;
        let row = self.get_research_task(task_id).await?.unwrap();
        Ok(row)
    }

    pub async fn list_tasks_for_session(
        &self,
        session_id: i64,
    ) -> Result<Vec<ResearchTaskRow>, StorageError> {
        let rows = sqlx::query_as::<_, ResearchTaskRow>(
            "SELECT * FROM research_tasks WHERE session_id=?1 ORDER BY updated_at DESC",
        )
        .bind(session_id)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows)
    }

    pub async fn get_session_summary(
        &self,
        session_id: i64,
    ) -> Result<serde_json::Value, StorageError> {
        let session = self
            .get_research_session(session_id)
            .await?
            .ok_or_else(|| StorageError::NotFound(format!("session {session_id} not found")))?;
        let tasks = self.list_tasks_for_session(session_id).await?;
        let total = tasks.len() as i64;
        let open = tasks.iter().filter(|t| t.status == "OPEN").count() as i64;
        let in_progress = tasks.iter().filter(|t| t.status == "IN_PROGRESS").count() as i64;
        let terminal = tasks
            .iter()
            .filter(|t| ["RESOLVED", "REJECTED", "INCONCLUSIVE"].contains(&t.status.as_str()))
            .count() as i64;
        // outcomes count: count outcomes for those task ids, no N+1 single query
        let outcomes_count = if tasks.is_empty() {
            0
        } else {
            let tids: Vec<i64> = tasks.iter().map(|t| t.id).collect();
            let placeholders = tids.iter().map(|_| "?").collect::<Vec<_>>().join(",");
            let sql = format!("SELECT COUNT(*) FROM research_outcomes WHERE task_id IN ({placeholders}) AND tree_id=?");
            let mut q = sqlx::query_scalar::<_, i64>(&sql);
            for tid in tids {
                q = q.bind(tid);
            }
            q = q.bind(session.tree_id);
            q.fetch_one(&self.pool).await.unwrap_or(0)
        };
        Ok(serde_json::json!({
            "total_tasks": total,
            "open_tasks": open,
            "in_progress_tasks": in_progress,
            "terminal_tasks": terminal,
            "outcomes_count": outcomes_count
        }))
    }

    pub async fn get_session_detail(
        &self,
        session_id: i64,
    ) -> Result<serde_json::Value, StorageError> {
        let session = self
            .get_research_session(session_id)
            .await?
            .ok_or_else(|| StorageError::NotFound(format!("session {session_id} not found")))?;
        // person batch single query
        let person = if let Some(pid) = session.person_id {
            sqlx::query_as::<_, PersonRow>("SELECT * FROM persons WHERE id=?1")
                .bind(pid)
                .fetch_optional(&self.pool)
                .await?
                .map(|p| {
                    let name = p.display_name.clone().unwrap_or_else(|| {
                        let gn = p.given_name.clone().unwrap_or_default();
                        let sn = p.surname.clone().unwrap_or_default();
                        let c = format!("{} {}", gn, sn).trim().to_string();
                        if c.is_empty() {
                            p.gedcom_id.clone()
                        } else {
                            c
                        }
                    });
                    serde_json::json!({ "id": p.id, "name": name, "gedcom_id": p.gedcom_id })
                })
        } else {
            None
        };
        let opportunity = if let Some(oid) = session.opportunity_id {
            sqlx::query_as::<_, ResearchOpportunityRow>(
                "SELECT * FROM research_opportunities WHERE id=?1",
            )
            .bind(oid)
            .fetch_optional(&self.pool)
            .await?
            .map(|o| {
                serde_json::json!({
                    "id": o.id,
                    "title": o.why.clone().unwrap_or_else(|| format!("Opportunity {}", o.id)),
                    "priority": o.priority,
                    "score": o.score,
                    "person_id": o.person_id
                })
            })
        } else {
            None
        };
        let tasks = self.list_tasks_for_session(session_id).await?;
        // Batch has_outcome and outcomes? For summary we already have method, but for detail we need tasks with has_outcome
        let task_ids: Vec<i64> = tasks.iter().map(|t| t.id).collect();
        let has_map = self
            .get_tasks_has_outcome_map(&task_ids)
            .await
            .unwrap_or_default();
        let tasks_json: Vec<serde_json::Value> = tasks
            .iter()
            .map(|t| {
                serde_json::json!({
                    "id": t.id,
                    "tree_id": t.tree_id,
                    "opportunity_id": t.opportunity_id,
                    "person_id": t.person_id,
                    "title": t.title,
                    "description": t.description,
                    "status": t.status,
                    "session_id": t.session_id,
                    "created_at": t.created_at,
                    "updated_at": t.updated_at,
                    "started_at": t.started_at,
                    "completed_at": t.completed_at,
                    "resolution": t.resolution,
                    "has_outcome": has_map.get(&t.id).copied().unwrap_or(false)
                })
            })
            .collect();
        let summary = self.get_session_summary(session_id).await?;
        Ok(serde_json::json!({
            "session": {
                "id": session.id,
                "tree_id": session.tree_id,
                "title": session.title,
                "description": session.description,
                "status": session.status,
                "person_id": session.person_id,
                "opportunity_id": session.opportunity_id,
                "created_at": session.created_at,
                "updated_at": session.updated_at,
                "started_at": session.started_at,
                "completed_at": session.completed_at
            },
            "person": person,
            "opportunity": opportunity,
            "tasks": tasks_json,
            "summary": summary
        }))
    }

    pub async fn get_tasks_session_map(
        &self,
        task_ids: &[i64],
    ) -> Result<std::collections::HashMap<i64, ResearchSessionRow>, StorageError> {
        if task_ids.is_empty() {
            return Ok(std::collections::HashMap::new());
        }
        // fetch tasks to get session_ids
        let placeholders = task_ids.iter().map(|_| "?").collect::<Vec<_>>().join(",");
        let sql = format!("SELECT * FROM research_tasks WHERE id IN ({placeholders})");
        let mut q = sqlx::query_as::<_, ResearchTaskRow>(&sql);
        for id in task_ids {
            q = q.bind(id);
        }
        let tasks = q.fetch_all(&self.pool).await?;
        let session_ids: Vec<i64> = tasks.iter().filter_map(|t| t.session_id).collect();
        if session_ids.is_empty() {
            return Ok(std::collections::HashMap::new());
        }
        let placeholders2 = session_ids
            .iter()
            .map(|_| "?")
            .collect::<Vec<_>>()
            .join(",");
        let sql2 = format!("SELECT * FROM research_sessions WHERE id IN ({placeholders2})");
        let mut q2 = sqlx::query_as::<_, ResearchSessionRow>(&sql2);
        for sid in &session_ids {
            q2 = q2.bind(sid);
        }
        let sessions = q2.fetch_all(&self.pool).await?;
        let sess_map: std::collections::HashMap<i64, ResearchSessionRow> =
            sessions.into_iter().map(|s| (s.id, s)).collect();
        let mut result = std::collections::HashMap::new();
        for t in tasks {
            if let Some(sid) = t.session_id {
                if let Some(sess) = sess_map.get(&sid) {
                    result.insert(t.id, sess.clone());
                }
            }
        }
        Ok(result)
    }

    pub async fn get_active_sessions_by_opportunity(
        &self,
        tree_id: i64,
    ) -> Result<std::collections::HashMap<i64, ResearchSessionRow>, StorageError> {
        let rows = sqlx::query_as::<_, ResearchSessionRow>(
            "SELECT * FROM research_sessions WHERE tree_id=?1 AND status='ACTIVE' AND opportunity_id IS NOT NULL",
        )
        .bind(tree_id)
        .fetch_all(&self.pool)
        .await?;
        let mut map = std::collections::HashMap::new();
        for r in rows {
            if let Some(oid) = r.opportunity_id {
                map.insert(oid, r);
            }
        }
        Ok(map)
    }
}
