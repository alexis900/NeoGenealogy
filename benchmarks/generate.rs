//! Generador sintético de GEDCOM para benchmarks
//! Uso: rustc benchmarks/generate.rs -o /tmp/gen && /tmp/gen 1000 > test-data/bench-1000.ged
//! o: cargo run --release --manifest-path benchmarks/Cargo.toml -- 1000

use std::env;

fn main() {
    let n: usize = env::args()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(1000);
    println!("0 HEAD");
    println!("1 CHAR UTF-8");
    println!("1 SOUR NeoGenealogy synthetic");
    // Create sources
    println!("0 @S1@ SOUR");
    println!("1 TITL Parish registers synthetic");
    println!("1 AUTH Synthetic");
    // Generate persons and families deterministically
    // Strategy: create couples and children in generations
    let surnames = ["García", "López", "Martínez", "Sánchez", "Pérez", "Ruiz", "Torres", "Moreno"];
    let given_m = ["Juan", "Francisco", "Pedro", "Miguel", "José", "Antonio", "Manuel", "Diego"];
    let given_f = ["María", "Carmen", "Ana", "Isabel", "Dolores", "Juana", "Francisca", "Antonia"];
    let places = ["Alcalá la Real", "Granada", "Priego de Córdoba", "Montefrío", "Jaén", "Madrid"];

    let mut person_id = 1usize;
    let mut family_id = 1usize;
    let mut persons: Vec<String> = Vec::new();
    let mut families: Vec<(String, String, String, Vec<String>)> = Vec::new();

    // Create initial generation 0 couples
    let mut current_gen_parents: Vec<String> = Vec::new();
    while person_id <= n {
        let surname_idx = (person_id / 2) % surnames.len();
        let is_male = person_id % 2 == 1;
        let given = if is_male {
            given_m[person_id % given_m.len()]
        } else {
            given_f[person_id % given_f.len()]
        };
        let surname = surnames[surname_idx];
        let birth_year = 1700 + (person_id % 200) as i32;
        let place = places[person_id % places.len()];
        let pid = format!("@I{}@", person_id);
        println!("0 {} INDI", pid);
        println!("1 NAME {given} /{surname}/");
        let date_variant = person_id % 7;
        let date_str = match date_variant {
            0 => format!("ABT {}", birth_year),
            1 => format!("BET {} AND {}", birth_year, birth_year + 2),
            2 => format!("BEF {}", birth_year + 1),
            3 => format!("AFT {}", birth_year - 1),
            4 => format!("FROM {} TO {}", birth_year, birth_year + 5),
            _ => birth_year.to_string(),
        };
        println!("1 BIRT");
        println!("2 DATE {}", date_str);
        println!("2 PLAC {}", place);
        if person_id % 3 == 0 {
            println!("1 SOUR @S1@");
        }
        if person_id % 10 == 0 {
            // duplicate-like second person with similar name/date/place for testing
        }
        persons.push(pid.clone());
        person_id += 1;

        // Every 2 persons form a family and possibly have children
        if persons.len() % 2 == 0 && persons.len() >= 2 && family_id * 2 <= n {
            let husband = persons[persons.len() - 2].clone();
            let wife = persons[persons.len() - 1].clone();
            let fid = format!("@F{}@", family_id);
            // children: create 1-3 children linked to next ids if available
            let child_count = if family_id % 3 == 0 { 2 } else if family_id % 2 == 0 { 1 } else { 0 };
            let mut children = Vec::new();
            for _ in 0..child_count {
                if person_id > n {
                    break;
                }
                // child will be generated in next loop iterations, but we need forward reference
                // Instead, we will assign children after they're created: link via FAMS/FAMC later
                // For now, placeholder
            }
            families.push((fid.clone(), husband, wife, children));
            current_gen_parents.push(fid.clone());
            family_id += 1;
        }
        if person_id > n { break; }
    }

    // Now create families output: link persons to families (simplified, no forward child creation complexity)
    // For benchmark purposes, linear families with 2 parents + 1-2 children using modulo
    let mut fam_idx = 1usize;
    let mut child_cursor = (surnames.len() * 2 + 1) as usize;
    // Re-output families (simplified: each family gets next 1-2 persons as children if exist)
    // We'll generate families sequentially
    let total_families = (n / 5).max(1);
    for fid in 1..=total_families {
        if fid * 2 + 1 > n { break; }
        let husb = format!("@I{}@", (fid * 2 - 1));
        let wife = format!("@I{}@", (fid * 2));
        let mut children = Vec::new();
        for c in 0..(fid % 3) {
            let cid = fid * 4 + c + total_families * 2;
            if cid <= n {
                children.push(format!("@I{}@", cid));
            }
        }
        println!("0 @F{}@ FAM", fid);
        println!("1 HUSB {}", husb);
        println!("1 WIFE {}", wife);
        for ch in &children {
            println!("1 CHIL {}", ch);
        }
        println!("1 MARR");
        println!("2 DATE {}", 1720 + (fid % 200) as i32);
        println!("2 PLAC {}", places[fid % places.len()]);
        if fid % 2 == 0 {
            println!("1 SOUR @S1@");
        }
        let _ = fam_idx;
        let _ = child_cursor;
    }

    println!("0 TRLR");
    eprintln!("Generated {} persons, {} families synthetic GEDCOM (requested {})", n, total_families, n);
}
