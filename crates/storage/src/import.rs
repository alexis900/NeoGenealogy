use sqlx::SqlitePool;

use crate::{error::StorageError, models::now_iso};
use neogenealogy_analyzer::{analyze, branch_analyses, source_coverage};
use neogenealogy_core::normalize_text;
use neogenealogy_gedcom::{GedcomParser, LegacyGedcomParser};
use neogenealogy_scoring::opportunities;
use std::path::Path;

pub struct ImportResult {
    pub tree_id: i64,
    pub analysis_run_id: i64,
    pub persons: usize,
    pub families: usize,
}

pub async fn import_gedcom_file(
    pool: &SqlitePool,
    gedcom_path: &Path,
    tree_name: Option<String>,
) -> Result<ImportResult, StorageError> {
    let content = tokio::fs::read_to_string(gedcom_path).await?;
    import_gedcom_content(
        pool,
        &content,
        gedcom_path.to_string_lossy().to_string(),
        tree_name,
    )
    .await
}

pub async fn import_gedcom_content(
    pool: &SqlitePool,
    content: &str,
    source_filename: impl Into<String>,
    tree_name: Option<String>,
) -> Result<ImportResult, StorageError> {
    let tree = LegacyGedcomParser
        .parse(content)
        .map_err(|e| StorageError::Import(format!("parse error: {e}")))?;

    // Run analysis in memory before transaction (no DB yet)
    let findings = analyze(&tree);
    let opps = opportunities(&tree, &findings);
    let branches = branch_analyses(&tree, &opps);
    let coverage = source_coverage(&tree);

    let source_filename = source_filename.into();
    let version = env!("CARGO_PKG_VERSION").to_string();
    let name = tree_name.unwrap_or_else(|| {
        Path::new(&source_filename)
            .file_stem()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| "import".into())
    });

    let mut tx = pool.begin().await?;

    // Create tree
    let now = now_iso();
    let tree_id = sqlx::query(
        "INSERT INTO trees (name, source_filename, gedcom_version, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5)",
    )
    .bind(&name)
    .bind(&source_filename)
    .bind("5.5.1")
    .bind(&now)
    .bind(&now)
    .execute(&mut *tx)
    .await?
    .last_insert_rowid();

    // Create analysis_run as running
    let started = now_iso();
    let analysis_run_id = sqlx::query(
        "INSERT INTO analysis_runs (tree_id, started_at, engine_version, status) VALUES (?1, ?2, ?3, 'running')",
    )
    .bind(tree_id)
    .bind(&started)
    .bind(&version)
    .execute(&mut *tx)
    .await?
    .last_insert_rowid();

    // Need maps from gedcom_id to db id
    use std::collections::HashMap;
    let mut person_map: HashMap<String, i64> = HashMap::new();
    let mut family_map: HashMap<String, i64> = HashMap::new();
    let mut place_map: HashMap<String, i64> = HashMap::new();

    // Insert places first (deduplicated per tree)
    // Collect all raw place names from persons, families, events
    let mut place_names: std::collections::HashSet<String> = std::collections::HashSet::new();
    for p in &tree.persons {
        if let Some(pl) = &p.birth_place {
            place_names.insert(pl.clone());
        }
        if let Some(pl) = &p.death_place {
            place_names.insert(pl.clone());
        }
    }
    for f in &tree.families {
        if let Some(pl) = &f.marriage_place {
            place_names.insert(pl.clone());
        }
    }
    for e in &tree.events {
        if let Some(pl) = &e.place {
            place_names.insert(pl.clone());
        }
    }
    for raw in place_names {
        let norm = normalize_text(&raw);
        let pid = sqlx::query(
            "INSERT OR IGNORE INTO places (tree_id, raw_name, normalized_name) VALUES (?1, ?2, ?3)",
        )
        .bind(tree_id)
        .bind(&raw)
        .bind(&norm)
        .execute(&mut *tx)
        .await?
        .last_insert_rowid();
        // If ignored, need to fetch existing
        let place_id = if pid == 0 {
            sqlx::query_scalar::<_, i64>("SELECT id FROM places WHERE tree_id=?1 AND raw_name=?2")
                .bind(tree_id)
                .bind(&raw)
                .fetch_one(&mut *tx)
                .await?
        } else {
            pid
        };
        place_map.insert(raw, place_id);
    }

    // Persons
    for p in &tree.persons {
        let display = format!("{} {}", p.given_name, p.surname).trim().to_string();
        let raw_tags = serde_json::to_string(&p.raw).unwrap_or_else(|_| "[]".to_string());
        let bd = p.birth_date.as_ref();
        let dd = p.death_date.as_ref();
        let id = sqlx::query(
            "INSERT INTO persons (tree_id, gedcom_id, given_name, surname, display_name, sex, raw_name, birth_date_original, birth_date_precision, birth_date_year, birth_date_start, birth_date_end, birth_place, death_date_original, death_date_precision, death_date_year, death_place, occupation, raw_tags) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16,?17,?18,?19)",
        )
        .bind(tree_id)
        .bind(&p.gedcom_id)
        .bind(&p.given_name)
        .bind(&p.surname)
        .bind(&display)
        .bind(&p.sex)
        .bind(&p.name_original)
        .bind(bd.map(|d| d.raw.clone()))
        .bind(bd.map(|d| format!("{:?}", d.precision)))
        .bind(bd.and_then(|d| d.year))
        .bind(bd.and_then(|d| d.range.as_ref().and_then(|r| r.start_year)))
        .bind(bd.and_then(|d| d.range.as_ref().and_then(|r| r.end_year)))
        .bind(&p.birth_place)
        .bind(dd.map(|d| d.raw.clone()))
        .bind(dd.map(|d| format!("{:?}", d.precision)))
        .bind(dd.and_then(|d| d.year))
        .bind(&p.death_place)
        .bind(&p.occupation)
        .bind(&raw_tags)
        .execute(&mut *tx)
        .await?
        .last_insert_rowid();
        person_map.insert(p.gedcom_id.clone(), id);
    }

    // Families
    for f in &tree.families {
        let raw_tags = serde_json::to_string(&f.raw).unwrap_or_else(|_| "[]".to_string());
        let id =
            sqlx::query("INSERT INTO families (tree_id, gedcom_id, raw_tags) VALUES (?1,?2,?3)")
                .bind(tree_id)
                .bind(&f.gedcom_id)
                .bind(&raw_tags)
                .execute(&mut *tx)
                .await?
                .last_insert_rowid();
        family_map.insert(f.gedcom_id.clone(), id);
    }

    // Family members
    for f in &tree.families {
        let fid = *family_map.get(&f.gedcom_id).unwrap();
        if let Some(h) = &f.husband_id {
            if let Some(pid) = person_map.get(h) {
                sqlx::query("INSERT OR IGNORE INTO family_members (family_id, person_id, role) VALUES (?1,?2,'husband')")
                    .bind(fid).bind(pid).execute(&mut *tx).await?;
            }
        }
        if let Some(w) = &f.wife_id {
            if let Some(pid) = person_map.get(w) {
                sqlx::query("INSERT OR IGNORE INTO family_members (family_id, person_id, role) VALUES (?1,?2,'wife')")
                    .bind(fid).bind(pid).execute(&mut *tx).await?;
            }
        }
        for c in &f.children {
            if let Some(pid) = person_map.get(c) {
                sqlx::query("INSERT OR IGNORE INTO family_members (family_id, person_id, role) VALUES (?1,?2,'child')")
                    .bind(fid).bind(pid).execute(&mut *tx).await?;
            }
        }
    }

    // Events (from tree.events + also BIRT/DEAT/MARR already captured as events)
    for e in &tree.events {
        let person_id = e
            .person_id
            .as_ref()
            .and_then(|gid| person_map.get(gid).copied());
        let family_id = e
            .family_id
            .as_ref()
            .and_then(|gid| family_map.get(gid).copied());
        let place_id = e.place.as_ref().and_then(|p| place_map.get(p).copied());
        let date_prec = e.date.as_ref().map(|d| format!("{:?}", d.precision));
        let raw_val = e.description.clone().unwrap_or_default();
        sqlx::query(
            "INSERT INTO events (tree_id, person_id, family_id, event_type, date_original, date_precision, date_start, date_end, date_year, place_id, place_raw, raw_value) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12)",
        )
        .bind(tree_id)
        .bind(person_id)
        .bind(family_id)
        .bind(&e.event_type)
        .bind(e.date.as_ref().map(|d| d.raw.clone()))
        .bind(date_prec)
        .bind(e.date.as_ref().and_then(|d| d.range.as_ref().and_then(|r| r.start_year)))
        .bind(e.date.as_ref().and_then(|d| d.range.as_ref().and_then(|r| r.end_year)))
        .bind(e.date.as_ref().and_then(|d| d.year))
        .bind(place_id)
        .bind(&e.place)
        .bind(&raw_val)
        .execute(&mut *tx)
        .await?;
    }
    // Ensure at least persons' birth/death events are represented even if not in tree.events due to parser variation
    // Already covered.

    // Sources
    let mut source_map: std::collections::HashMap<String, i64> = std::collections::HashMap::new();
    for s in &tree.sources {
        let id = sqlx::query(
            "INSERT INTO sources (tree_id, gedcom_id, title, author, publication, text, repository, url) VALUES (?1,?2,?3,?4,?5,?6,?7,?8)",
        )
        .bind(tree_id)
        .bind(&s.gedcom_id)
        .bind(&s.title)
        .bind(&s.author)
        .bind(&s.citation)
        .bind(&s.citation)
        .bind(&s.repository)
        .bind(&s.url)
        .execute(&mut *tx)
        .await?
        .last_insert_rowid();
        source_map.insert(s.gedcom_id.clone(), id);
    }

    // Citations linking persons to sources (use citations table)
    // For each person.sources list, create citation entry if source exists
    for p in &tree.persons {
        if let Some(pid) = person_map.get(&p.gedcom_id) {
            for src_ged in &p.sources {
                if let Some(sid) = source_map.get(src_ged) {
                    // try to extract page from raw_tags if present
                    let page = None::<String>; // placeholder; could parse raw
                    sqlx::query(
                        "INSERT INTO citations (tree_id, source_id, person_id, page, text) VALUES (?1,?2,?3,?4,?5)",
                    )
                    .bind(tree_id)
                    .bind(sid)
                    .bind(pid)
                    .bind(page)
                    .bind::<Option<String>>(None)
                    .execute(&mut *tx)
                    .await?;
                }
            }
        }
    }
    for f in &tree.families {
        if let Some(fid_db) = family_map.get(&f.gedcom_id) {
            // family sources could be citations too
            for src_ged in &f.sources {
                if let Some(sid) = source_map.get(src_ged) {
                    sqlx::query(
                        "INSERT INTO citations (tree_id, source_id, family_id, page, text) VALUES (?1,?2,?3,?4,?5)",
                    )
                    .bind(tree_id)
                    .bind(sid)
                    .bind(*fid_db)
                    .bind::<Option<String>>(None)
                    .bind::<Option<String>>(None)
                    .execute(&mut *tx)
                    .await?;
                }
            }
        }
    }

    // Findings
    for fin in &findings {
        let person_id = fin
            .person_id
            .as_ref()
            .and_then(|gid| person_map.get(gid).copied());
        let related_id = fin
            .related_person_id
            .as_ref()
            .and_then(|gid| person_map.get(gid).copied());
        // family_id unknown for most findings; attempt to parse if related to family
        let evidence = serde_json::to_string(&fin.evidence).unwrap_or_else(|_| "[]".into());
        let now2 = now_iso();
        sqlx::query(
            "INSERT INTO findings (tree_id, analysis_run_id, person_id, related_person_id, finding_type, severity, confidence, message, evidence, created_at) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10)",
        )
        .bind(tree_id)
        .bind(analysis_run_id)
        .bind(person_id)
        .bind(related_id)
        .bind(&fin.kind)
        .bind(fin.severity.as_str())
        .bind(fin.confidence as f64)
        .bind(&fin.description)
        .bind(&evidence)
        .bind(&now2)
        .execute(&mut *tx)
        .await?;
    }

    // Research opportunities: need to map person gedcom to finding? We'll link via person
    for opp in &opps {
        let pid = person_map.get(&opp.person_id).copied().ok_or_else(|| {
            StorageError::Import(format!("opp person not found {}", opp.person_id))
        })?;
        let breakdown = serde_json::to_string(&opp.breakdown).unwrap_or_else(|_| "{}".into());
        let what = serde_json::to_string(&opp.what_is_known).unwrap_or_else(|_| "[]".into());
        let potential =
            serde_json::to_string(&opp.potential_sources).unwrap_or_else(|_| "[]".into());
        let missing =
            serde_json::to_string(&opp.missing_information).unwrap_or_else(|_| "[]".into());
        let reasons = serde_json::to_string(&opp.reasons).unwrap_or_else(|_| "[]".into());
        sqlx::query(
            "INSERT INTO research_opportunities (tree_id, analysis_run_id, person_id, priority, score, confidence, researchability, why, what, potential_sources, breakdown, missing_information, reasons) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13)",
        )
        .bind(tree_id)
        .bind(analysis_run_id)
        .bind(pid)
        .bind(opp.priority.as_str())
        .bind(opp.score as i64)
        .bind(opp.confidence as f64)
        .bind(opp.researchability.as_str())
        .bind(&opp.why_it_matters)
        .bind(&what)
        .bind(&potential)
        .bind(&breakdown)
        .bind(&missing)
        .bind(&reasons)
        .execute(&mut *tx)
        .await?;
    }

    // Branch analyses
    for b in &branches {
        sqlx::query(
            "INSERT INTO branch_analyses (tree_id, analysis_run_id, name, score, opportunity_count, high_priority_count, deepest_generation, source_coverage) VALUES (?1,?2,?3,?4,?5,?6,?7,?8)",
        )
        .bind(tree_id)
        .bind(analysis_run_id)
        .bind(&b.name)
        .bind(b.score as i64)
        .bind(b.opportunity_count as i64)
        .bind(b.high_priority_count as i64)
        .bind(b.deepest_generation as i64)
        .bind(b.source_coverage as f64)
        .execute(&mut *tx)
        .await?;
    }

    // Source coverage snapshot
    sqlx::query(
        "INSERT INTO source_coverages (tree_id, analysis_run_id, birth, marriage, death, other_events, overall) VALUES (?1,?2,?3,?4,?5,?6,?7)",
    )
    .bind(tree_id)
    .bind(analysis_run_id)
    .bind(coverage.birth as f64)
    .bind(coverage.marriage as f64)
    .bind(coverage.death as f64)
    .bind(coverage.other_events as f64)
    .bind(coverage.overall as f64)
    .execute(&mut *tx)
    .await?;

    // Complete analysis run
    let completed = now_iso();
    sqlx::query("UPDATE analysis_runs SET completed_at=?1, status='completed' WHERE id=?2")
        .bind(&completed)
        .bind(analysis_run_id)
        .execute(&mut *tx)
        .await?;

    tx.commit().await?;

    Ok(ImportResult {
        tree_id,
        analysis_run_id,
        persons: tree.persons.len(),
        families: tree.families.len(),
    })
}
