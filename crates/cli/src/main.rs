use anyhow::{anyhow, Result};
use clap::{Parser, Subcommand, ValueEnum};
use neogenealogy_analyzer::{analyze, ancestry_stats, branch_analyses, source_coverage};
use neogenealogy_core::{GenealogyTree, Severity};
use neogenealogy_gedcom::parse_file;
use neogenealogy_scoring::opportunities;
use neogenealogy_storage::{establish_pool_from_path, run_migrations, Storage};
use std::{collections::HashMap, fs, path::PathBuf};

#[derive(Parser)]
#[command(name = "neogenealogy", version = "0.3")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(ValueEnum, Clone, Debug)]
enum SortField {
    Score,
    Priority,
    Confidence,
}

#[derive(Subcommand)]
enum Command {
    Analyze {
        file: PathBuf,
        #[arg(short, long)]
        output: Option<PathBuf>,
        #[arg(long)]
        explain_score: bool,
        #[arg(long)]
        severity: Option<String>,
        #[arg(long, value_enum, default_value = "score")]
        sort: SortField,
    },
    Import {
        file: PathBuf,
        #[arg(long)]
        db: Option<PathBuf>,
    },
    Stats {
        #[arg(value_name = "FILE")]
        file: Option<PathBuf>,
        #[arg(long)]
        db: Option<PathBuf>,
    },
    Report {
        #[arg(value_name = "FILE")]
        file: Option<PathBuf>,
        #[arg(short, long, default_value = "report.html")]
        output: PathBuf,
        #[arg(long)]
        db: Option<PathBuf>,
    },
    Serve {
        #[arg(long, env = "NEOGENEALOGY_DATABASE_URL")]
        db: Option<PathBuf>,
        #[arg(long, env = "NEOGENEALOGY_HOST", default_value = "127.0.0.1")]
        host: String,
        #[arg(long, env = "NEOGENEALOGY_PORT", default_value = "3000")]
        port: u16,
    },
}

type TreeResult = (
    GenealogyTree,
    Vec<neogenealogy_core::Finding>,
    Vec<neogenealogy_core::ResearchOpportunity>,
    neogenealogy_core::SourceCoverage,
    Vec<neogenealogy_core::BranchAnalysis>,
);
#[allow(clippy::type_complexity)]
fn run_with_tree(file: &std::path::Path) -> Result<TreeResult> {
    let t = parse_file(file)?;
    let f = analyze(&t);
    let mut o = opportunities(&t, &f);
    o.sort_by_key(|b| std::cmp::Reverse(b.score));
    let sc = source_coverage(&t);
    let branches = branch_analyses(&t, &o);
    Ok((t, f, o, sc, branches))
}

fn severity_filter(
    findings: Vec<neogenealogy_core::Finding>,
    level: &str,
) -> Vec<neogenealogy_core::Finding> {
    if let Some(min_sev) = Severity::from_str(level) {
        let min_rank = min_sev.rank();
        findings
            .into_iter()
            .filter(|f| f.severity.rank() >= min_rank)
            .collect()
    } else {
        findings
    }
}

fn sort_opportunities(
    mut opps: Vec<neogenealogy_core::ResearchOpportunity>,
    sort: &SortField,
) -> Vec<neogenealogy_core::ResearchOpportunity> {
    match sort {
        SortField::Score => opps.sort_by_key(|b| std::cmp::Reverse(b.score)),
        SortField::Priority => opps.sort_by_key(|b| std::cmp::Reverse(b.priority.rank())),
        SortField::Confidence => opps.sort_by(|a, b| {
            b.confidence
                .partial_cmp(&a.confidence)
                .unwrap_or(std::cmp::Ordering::Equal)
        }),
    }
    opps
}

fn compact_output(
    t: &GenealogyTree,
    findings: &[neogenealogy_core::Finding],
    opportunities: &[neogenealogy_core::ResearchOpportunity],
    sc: &neogenealogy_core::SourceCoverage,
    branches: &[neogenealogy_core::BranchAnalysis],
    explain: bool,
) {
    let max_gen = t
        .persons
        .iter()
        .filter_map(|p| t.generation_distance(&p.gedcom_id))
        .max()
        .unwrap_or(0);

    let mut sev_count: HashMap<String, usize> = HashMap::new();
    for f in findings {
        *sev_count
            .entry(f.severity.as_str().to_string())
            .or_default() += 1;
    }
    let mut prio_count: HashMap<String, usize> = HashMap::new();
    for o in opportunities {
        *prio_count
            .entry(o.priority.as_str().to_string())
            .or_default() += 1;
    }

    println!("NeoGenealogy 0.3\n");
    println!("Persons: {}", t.persons.len());
    println!("Families: {}", t.families.len());
    println!("Generations: {}\n", max_gen);

    println!("Findings:");
    for sev in ["critical", "high", "warning", "medium", "info", "low"] {
        if let Some(c) = sev_count.get(sev) {
            println!("  {}: {}", capitalize(sev), c);
        }
    }
    if findings.is_empty() {
        println!("  (none)");
    }
    println!();
    println!("Research opportunities:");
    for prio in ["critical", "high", "medium", "low"] {
        if let Some(c) = prio_count.get(prio) {
            println!("  {}: {}", capitalize(prio), c);
        }
    }
    if opportunities.is_empty() {
        println!("  (none)");
    }
    println!();

    if !opportunities.is_empty() {
        println!("Top opportunities:\n");
        for opp in opportunities.iter().take(5) {
            let person = t.person(&opp.person_id);
            let name = person
                .map(|p| format!("{} {}", p.given_name, p.surname).trim().to_string())
                .unwrap_or_else(|| opp.person_id.clone());
            let icon = match opp.priority {
                Severity::Critical => "🔥",
                Severity::High => "🔥",
                Severity::Medium => "•",
                _ => "·",
            };
            let display_name = if name.is_empty() {
                opp.person_id.clone()
            } else {
                name
            };
            println!(
                "{} {}  {}  {}/100  confidence {:.0}%  researchability {}",
                icon,
                opp.score,
                display_name,
                opp.score,
                opp.confidence * 100.0,
                opp.researchability.as_str()
            );
            if explain {
                println!();
                println!("  Why it matters: {}", opp.why_it_matters);
                if !opp.what_is_known.is_empty() {
                    println!("  Known: {}", opp.what_is_known.join(", "));
                }
                if !opp.missing_information.is_empty() {
                    println!("  Missing: {}", opp.missing_information.join(", "));
                }
                println!("  Potential sources: {}", opp.potential_sources.join(", "));
                println!("  Score breakdown:");
                for c in &opp.breakdown.components {
                    println!("    {:+} {} — {}", c.points, c.name, c.reason);
                }
                println!("    ---");
                println!(
                    "    Total {} (confidence {:.0}%)",
                    opp.breakdown.total,
                    opp.confidence * 100.0
                );
                println!();
            }
        }
        println!();
    }

    if !branches.is_empty() {
        println!("Best branch:");
        let best = &branches[0];
        println!("{} — {}/100", best.name, best.score);
        println!("  Opportunities: {}", best.opportunity_count);
        println!("  High priority: {}", best.high_priority_count);
        println!("  Deepest generation: {}", best.deepest_generation);
        println!("  Source coverage: {:.0}%", best.source_coverage);
        println!();
    }

    println!("Source coverage:");
    println!("  Birth    {:.0}%", sc.birth);
    println!("  Marriage {:.0}%", sc.marriage);
    println!("  Death    {:.0}%", sc.death);
    println!("  Events   {:.0}%", sc.other_events);
    println!("  Overall  {:.0}%", sc.overall);
    println!();

    println!("Ancestral depth:");
    if max_gen == 0 {
        println!("  (no ancestry data)");
    } else {
        for gen in 1..=max_gen {
            let has = t
                .persons
                .iter()
                .any(|p| t.generation_distance(&p.gedcom_id) == Some(gen));
            let mark = if has { "✓" } else { "❌" };
            println!("  Generation {}  {}", gen, mark);
        }
    }
}

fn capitalize(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Command::Analyze {
            file,
            output,
            explain_score,
            severity,
            sort,
        } => {
            let (t, findings_raw, opps_raw, sc, branches) = run_with_tree(&file)?;
            let findings = if let Some(ref sev) = severity {
                severity_filter(findings_raw, sev)
            } else {
                findings_raw
            };
            let mut opps = sort_opportunities(opps_raw, &sort);
            if let Some(ref sev) = severity {
                if let Some(min_sev) = Severity::from_str(sev) {
                    let min_rank = min_sev.rank();
                    opps.retain(|o| o.priority.rank() >= min_rank);
                }
            }

            if let Some(p) = output {
                let best_root = t
                    .persons
                    .iter()
                    .max_by_key(|p| t.generation_distance(&p.gedcom_id).unwrap_or(0))
                    .map(|p| p.gedcom_id.as_str())
                    .unwrap_or("");
                let ancestry = if best_root.is_empty() {
                    serde_json::json!({})
                } else {
                    let stats = ancestry_stats(&t, best_root);
                    serde_json::to_value(stats).unwrap_or(serde_json::json!({}))
                };
                let v = serde_json::json!({
                    "summary": {
                        "persons": t.persons.len(),
                        "families": t.families.len(),
                        "sources": t.sources.len(),
                        "findings": findings.len(),
                        "opportunities": opps.len()
                    },
                    "tree": t,
                    "findings": findings,
                    "research_opportunities": opps,
                    "opportunities": opps,
                    "branches": branches,
                    "source_coverage": sc,
                    "ancestral_depth": ancestry
                });
                let s = serde_json::to_string_pretty(&v)?;
                fs::write(p, s)?;
            } else {
                compact_output(&t, &findings, &opps, &sc, &branches, explain_score);
            }
        }
        Command::Import { file, db } => {
            let db_path = db.unwrap_or_else(|| PathBuf::from("neogenealogy.db"));
            let pool = establish_pool_from_path(&db_path).await?;
            run_migrations(&pool).await?;
            let res = neogenealogy_storage::import::import_gedcom_file(&pool, &file, None).await?;
            println!(
                "Importado árbol {}: {} personas, {} familias (tree_id={}, run_id={}) en {:?}",
                file.display(),
                res.persons,
                res.families,
                res.tree_id,
                res.analysis_run_id,
                db_path
            );
        }
        Command::Stats { file, db } => {
            if let Some(db_path) = db {
                let pool = establish_pool_from_path(&db_path).await?;
                run_migrations(&pool).await?;
                let storage = Storage::new(pool);
                let trees = storage.list_trees(None, None).await?;
                if trees.is_empty() {
                    println!("No trees in database {:?}", db_path);
                } else {
                    println!("Trees: {}", trees.len());
                    for tr in &trees {
                        let (p, f, e, s, find, opps) = storage.count(tr.id).await?;
                        let runs = storage.get_analysis_runs(tr.id).await?;
                        let coverage = storage.get_source_coverage(tr.id).await?;
                        println!(
                            "\nTree {} ({}): {} persons, {} families, {} events, {} sources, {} findings, {} opps, {} runs",
                            tr.id, tr.name, p, f, e, s, find, opps, runs.len()
                        );
                        if let Some(cov) = coverage {
                            println!(
                                "  Source coverage overall: {:.0}%",
                                cov.overall.unwrap_or(0.0)
                            );
                        }
                        if let Some(run) = runs.first() {
                            println!("  Last analysis: {} status {}", run.id, run.status);
                        }
                    }
                }
            } else if let Some(path) = file {
                let (t, findings, opps, sc, branches) = run_with_tree(&path)?;
                let max_gen = t
                    .persons
                    .iter()
                    .filter_map(|p| t.generation_distance(&p.gedcom_id))
                    .max()
                    .unwrap_or(0);
                println!(
                    "Persons: {}\nFamilies: {}\nSources: {}\nFindings: {}\nResearch opportunities: {}\nGenerations: {}\nSource coverage overall: {:.0}%\nBranches: {}",
                    t.persons.len(),
                    t.families.len(),
                    t.sources.len(),
                    findings.len(),
                    opps.len(),
                    max_gen,
                    sc.overall,
                    branches.len()
                );
                for b in branches.iter().take(3) {
                    println!(
                        "  Branch {}: score {}/100, opps {}, high {}, coverage {:.0}%",
                        b.name,
                        b.score,
                        b.opportunity_count,
                        b.high_priority_count,
                        b.source_coverage
                    );
                }
            } else {
                return Err(anyhow!("stats requires either FILE or --db"));
            }
        }
        Command::Report { file, output, db } => {
            if let Some(db_path) = db {
                let pool = establish_pool_from_path(&db_path).await?;
                run_migrations(&pool).await?;
                let storage = Storage::new(pool);
                let trees = storage.list_trees(None, None).await?;
                let tree = trees.first().ok_or_else(|| anyhow!("no trees in db"))?;
                let findings = storage.get_findings(tree.id, Some(100), None).await?;
                let opps = storage
                    .get_research_opportunities(tree.id, Some(100), None)
                    .await?;
                let branches = storage.get_branches(tree.id).await?;
                let coverage = storage
                    .get_source_coverage(tree.id)
                    .await?
                    .unwrap_or(crate::models_placeholder());
                // Build simple HTML from DB (reuse logic but from stored rows)
                let mut top_list = String::new();
                for o in opps.iter().take(20) {
                    // Need person display name: fetch persons list to map
                    let persons = storage.list_persons(tree.id, Some(1000), None).await?;
                    let person = persons.iter().find(|p| p.id == o.person_id);
                    let name = person
                        .map(|p| {
                            format!(
                                "{} {}",
                                p.given_name.clone().unwrap_or_default(),
                                p.surname.clone().unwrap_or_default()
                            )
                            .trim()
                            .to_string()
                        })
                        .unwrap_or_else(|| format!("person {}", o.person_id));
                    let breakdown_str = o.breakdown.clone().unwrap_or_else(|| "[]".into());
                    top_list.push_str(&format!(
                        "<li><strong>{}/100</strong> {} — priority {}<br><small>{}</small><br><small>breakdown: {}</small></li>",
                        o.score.unwrap_or(0),
                        name,
                        o.priority.clone().unwrap_or_default(),
                        o.why.clone().unwrap_or_default(),
                        breakdown_str
                    ));
                }
                let mut branch_list = String::new();
                for b in branches.iter().take(20) {
                    branch_list.push_str(&format!(
                        "<li><strong>{}</strong> — Score {}/100 · Opportunities {} · High {}</li>",
                        b.name,
                        b.score.unwrap_or(0),
                        b.opportunity_count.unwrap_or(0),
                        b.high_priority_count.unwrap_or(0)
                    ));
                }
                let findings_html = findings
                    .iter()
                    .map(|f| {
                        format!(
                            "<li><strong>{}</strong> [{}] — {}</li>",
                            f.finding_type,
                            f.severity,
                            f.message.clone().unwrap_or_default()
                        )
                    })
                    .collect::<String>();
                let cov_birth = coverage.birth.unwrap_or(0.0);
                let cov_ov = coverage.overall.unwrap_or(0.0);
                let html = format!(
                    r#"<!doctype html><meta charset='utf-8'><title>NeoGenealogy Report (DB)</title>
<h1>NEOGENEALOGY REPORT (DB)</h1>
<p>Tree: {} — Persons (via opps) Findings: {} Branches: {}</p>
<p>Source coverage birth {cov_birth:.0}% overall {cov_ov:.0}%</p>
<h2>TOP RESEARCH OPPORTUNITIES</h2><ol>{top_list}</ol>
<h2>BEST RESEARCH BRANCHES</h2><ol>{branch_list}</ol>
<h2>Findings</h2><ul>{findings_html}</ul>
"#,
                    tree.name,
                    opps.len(),
                    branches.len(),
                    cov_birth = cov_birth,
                    cov_ov = cov_ov,
                    top_list = top_list,
                    branch_list = branch_list,
                    findings_html = findings_html
                );
                fs::write(&output, html)?;
                println!("Report written from DB {:?} to {:?}", tree.name, output);
            } else if let Some(path) = file {
                let (t, findings, opps, sc, branches) = run_with_tree(&path)?;
                let best_root = t
                    .persons
                    .iter()
                    .max_by_key(|p| t.generation_distance(&p.gedcom_id).unwrap_or(0))
                    .map(|p| p.gedcom_id.as_str())
                    .unwrap_or("");
                let ancestry = if best_root.is_empty() {
                    serde_json::json!({})
                } else {
                    serde_json::to_value(ancestry_stats(&t, best_root))
                        .unwrap_or(serde_json::json!({}))
                };
                let mut top_list = String::new();
                for o in opps.iter().take(20) {
                    let person = t.person(&o.person_id);
                    let name = person
                        .map(|p| format!("{} {}", p.given_name, p.surname).trim().to_string())
                        .unwrap_or_else(|| o.person_id.clone());
                    let breakdown_html = o
                        .breakdown
                        .components
                        .iter()
                        .map(|c| format!("<li>{:+} {} — {}</li>", c.points, c.name, c.reason))
                        .collect::<String>();
                    top_list.push_str(&format!(
                        "<li><strong>{}/100</strong> {} — confidence {:.0}% — {}<br><small>{}</small><ul>{}</ul><small>Why: {} | Known: {} | Missing: {} | Sources: {}</small></li>",
                        o.score,
                        name,
                        o.confidence*100.0,
                        o.priority.as_str(),
                        o.reasons.join("; "),
                        breakdown_html,
                        o.why_it_matters,
                        o.what_is_known.join(", "),
                        o.missing_information.join(", "),
                        o.potential_sources.join(", ")
                    ));
                }
                let mut branch_list = String::new();
                for b in branches.iter().take(20) {
                    branch_list.push_str(&format!(
                        "<li><strong>{}</strong> — Score {}/100 · Opportunities {} · High {} · Deepest gen {} · Coverage {:.0}%</li>",
                        b.name, b.score, b.opportunity_count, b.high_priority_count, b.deepest_generation, b.source_coverage
                    ));
                }
                let html = format!(
                    r#"<!doctype html><meta charset='utf-8'><title>NeoGenealogy Report</title>
<style>body{{font-family:system-ui, sans-serif; max-width:900px; margin:2rem auto; line-height:1.5}} h1,h2{{color:#222}} .metric{{display:inline-block; margin-right:1.5rem}} </style>
<h1>NEOGENEALOGY REPORT</h1>
<h2>Overview</h2>
<p><span class="metric">Persons: {persons}</span> <span class="metric">Families: {families}</span> <span class="metric">Sources: {sources}</span> <span class="metric">Findings: {findings_len}</span> <span class="metric">Opportunities: {opps_len}</span></p>
<h2>Statistics</h2>
<p>Source coverage — Birth {birth:.0}% · Marriage {marriage:.0}% · Death {death:.0}% · Events {events:.0}% · Overall {overall:.0}%</p>
<p>Ancestral depth: {ancestry}</p>
<h2>Findings</h2>
<ul>{findings_html}</ul>
<h2>TOP RESEARCH OPPORTUNITIES</h2>
<ol>{top_list}</ol>
<h2>BEST RESEARCH BRANCHES</h2>
<ol>{branch_list}</ol>
<h2>Branches</h2>
<ol>{branch_list2}</ol>
<h2>Source Coverage</h2>
<p>Birth {birth:.0}% | Marriage {marriage:.0}% | Death {death:.0}% | Events {events:.0}% | Overall {overall:.0}%</p>
<h2>Ancestral Depth</h2>
<pre>{ancestry_pretty}</pre>
"#,
                    persons = t.persons.len(),
                    families = t.families.len(),
                    sources = t.sources.len(),
                    findings_len = findings.len(),
                    opps_len = opps.len(),
                    birth = sc.birth,
                    marriage = sc.marriage,
                    death = sc.death,
                    events = sc.other_events,
                    overall = sc.overall,
                    ancestry = ancestry,
                    findings_html = findings
                        .iter()
                        .map(|f| format!(
                            "<li><strong>{}</strong> [{}] {} — {}</li>",
                            f.kind,
                            f.severity.as_str(),
                            f.person_id.clone().unwrap_or_default(),
                            f.description
                        ))
                        .collect::<String>(),
                    top_list = top_list,
                    branch_list = branch_list,
                    branch_list2 = branch_list,
                    ancestry_pretty = serde_json::to_string_pretty(&ancestry).unwrap_or_default()
                );
                fs::write(output, html)?;
            } else {
                return Err(anyhow!("report requires either FILE or --db"));
            }
        }
        Command::Serve { db, host, port } => {
            let db_path = db.unwrap_or_else(|| PathBuf::from("neogenealogy.db"));
            // Support DATABASE_URL env as sqlite url
            let db_path = if let Ok(url) = std::env::var("NEOGENEALOGY_DATABASE_URL") {
                // url may be sqlite://...
                if url.starts_with("sqlite://") {
                    PathBuf::from(url.trim_start_matches("sqlite://"))
                } else {
                    db_path
                }
            } else {
                db_path
            };
            tracing_subscriber::fmt()
                .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
                .init();
            let pool = establish_pool_from_path(&db_path).await?;
            run_migrations(&pool).await?;
            let storage = Storage::new(pool);
            let state = neogenealogy_api::state::AppState::new(storage);
            let app = neogenealogy_api::create_router(state);
            let addr = format!("{host}:{port}");
            println!("NeoGenealogy API");
            println!("Listening on {addr}");
            println!("Database: {:?}", db_path);
            let listener = tokio::net::TcpListener::bind(&addr).await?;
            axum::serve(listener, app).await?;
        }
    }
    Ok(())
}

fn models_placeholder() -> neogenealogy_storage::models::SourceCoverageRow {
    neogenealogy_storage::models::SourceCoverageRow {
        id: 0,
        tree_id: 0,
        analysis_run_id: 0,
        birth: Some(0.0),
        marriage: Some(0.0),
        death: Some(0.0),
        other_events: Some(0.0),
        overall: Some(0.0),
    }
}
