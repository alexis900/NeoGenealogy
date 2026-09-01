use anyhow::Result;
use clap::{Parser, Subcommand};
use neogenealogy_analyzer::analyze;
use neogenealogy_gedcom::parse_file;
use neogenealogy_scoring::opportunities;
use std::{fs, path::PathBuf};
#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}
#[derive(Subcommand)]
enum Command {
    Analyze {
        file: PathBuf,
        #[arg(short, long)]
        output: Option<PathBuf>,
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
fn run(
    file: &std::path::Path,
) -> Result<(
    neogenealogy_core::GenealogyTree,
    Vec<neogenealogy_core::Finding>,
    Vec<neogenealogy_core::ResearchOpportunity>,
)> {
    let t = parse_file(file)?;
    let f = analyze(&t);
    let o = opportunities(&t, &f);
    Ok((t, f, o))
}
fn main() -> Result<()> {
    match Cli::parse().command {
        Command::Analyze { file, output } => {
            let (t, f, o) = run(&file)?;
            let v = serde_json::json!({"tree":t,"findings":f,"opportunities":o});
            let s = serde_json::to_string_pretty(&v)?;
            if let Some(p) = output {
                fs::write(p, s)?
            } else {
                println!("{s}");
            }
        }
        Command::Import { file } => {
            let (t, _, _) = run(&file)?;
            println!(
                "Importadas {} personas, {} familias y {} fuentes.",
                t.persons.len(),
                t.families.len(),
                t.sources.len()
            );
        }
        Command::Stats { file } => {
            let (t, f, o) = run(&file)?;
            println!(
                "Persons: {}\nFamilies: {}\nSources: {}\nFindings: {}\nResearch opportunities: {}",
                t.persons.len(),
                t.families.len(),
                t.sources.len(),
                f.len(),
                o.len()
            );
        }
        Command::Report { file, output } => {
            let (t, f, o) = run(&file)?;
            let body = o
                .iter()
                .take(20)
                .map(|x| {
                    format!(
                        "<li><strong>{}/100</strong> {} — {}</li>",
                        x.score,
                        x.person_id,
                        x.reasons.join("; ")
                    )
                })
                .collect::<String>();
            let html=format!("<!doctype html><meta charset='utf-8'><title>NeoGenealogy Report</title><h1>NEOGENEALOGY REPORT</h1><p>Persons: {} · Families: {} · Findings: {}</p><h2>Top research opportunities</h2><ol>{}</ol>",t.persons.len(),t.families.len(),f.len(),body);
            fs::write(output, html)?;
        }
    }
    Ok(())
}
