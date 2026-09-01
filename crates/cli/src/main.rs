use anyhow::Result;
use clap::{Parser, Subcommand, ValueEnum};
use neogenealogy_analyzer::{analyze, ancestry_stats, branch_analyses, source_coverage};
use neogenealogy_core::{GenealogyTree, Severity};
use neogenealogy_gedcom::parse_file;
use neogenealogy_scoring::opportunities;
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
    },
    Stats {
        file: PathBuf,
    },
    Report {
        file: PathBuf,
        #[arg(short, long, default_value = "report.html")]
        output: PathBuf,
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
    // ensure sorted by default score
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
    // Estimate generations as max ancestry depth among all persons
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
            // check missing generation?
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

fn main() -> Result<()> {
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
            // severity filter applies to findings
            let findings = if let Some(ref sev) = severity {
                severity_filter(findings_raw, sev)
            } else {
                findings_raw
            };
            let mut opps = sort_opportunities(opps_raw, &sort);
            // also filter opportunities by severity if requested? Spec says findings; but we also filter opps for consistency
            if let Some(ref sev) = severity {
                if let Some(min_sev) = Severity::from_str(sev) {
                    let min_rank = min_sev.rank();
                    opps.retain(|o| o.priority.rank() >= min_rank);
                }
            }

            if let Some(p) = output {
                // JSON stable output: deepest person as root for ancestral depth
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
                // also if explain_score, each opportunity breakdown is shown via compact_output
            }
        }
        Command::Import { file } => {
            let (t, _, _, _, _) = run_with_tree(&file)?;
            println!(
                "Importadas {} personas, {} familias y {} fuentes.",
                t.persons.len(),
                t.families.len(),
                t.sources.len()
            );
        }
        Command::Stats { file } => {
            let (t, findings, opps, sc, branches) = run_with_tree(&file)?;
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
            // show branch summary quickly
            for b in branches.iter().take(3) {
                println!(
                    "  Branch {}: score {}/100, opps {}, high {}, coverage {:.0}%",
                    b.name, b.score, b.opportunity_count, b.high_priority_count, b.source_coverage
                );
            }
        }
        Command::Report { file, output } => {
            let (t, findings, opps, sc, branches) = run_with_tree(&file)?;
            let best_root = t
                .persons
                .iter()
                .max_by_key(|p| t.generation_distance(&p.gedcom_id).unwrap_or(0))
                .map(|p| p.gedcom_id.as_str())
                .unwrap_or("");
            let ancestry = if best_root.is_empty() {
                serde_json::json!({})
            } else {
                serde_json::to_value(ancestry_stats(&t, best_root)).unwrap_or(serde_json::json!({}))
            };
            // Build HTML sections
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
        }
    }
    Ok(())
}
