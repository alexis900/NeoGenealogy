use neogenealogy_core::*;
use std::collections::HashSet;

fn researchability_for(person: &Person) -> Researchability {
    let has_place = person.birth_place.is_some();
    let has_date = person.birth_date.is_some();
    let year = person.birth_date.as_ref().and_then(|d| d.year).unwrap_or(0);
    // Very old records are harder
    if year != 0 && year < 1500 {
        return Researchability::Low;
    }
    match (has_place, has_date) {
        (true, true) => Researchability::High,
        (true, false) | (false, true) => Researchability::Medium,
        (false, false) => Researchability::Low,
    }
}

fn confidence_for(person: &Person, tree: &GenealogyTree, findings: &[Finding]) -> f32 {
    let mut conf: f32 = 0.5;
    if let Some(d) = &person.birth_date {
        conf += match d.precision {
            DatePrecision::Exact => 0.25,
            DatePrecision::Year => 0.18,
            DatePrecision::About => 0.10,
            DatePrecision::Between | DatePrecision::FromTo => 0.12,
            DatePrecision::Before | DatePrecision::After => 0.07,
            DatePrecision::Unknown => 0.0,
        };
        if !d.approximate {
            conf += 0.05;
        }
    }
    if person.birth_place.is_some() {
        conf += 0.08;
    }
    if !person.sources.is_empty() {
        // check if any source has citation (higher quality)
        let has_citation = person.sources.iter().any(|sid| {
            tree.sources
                .iter()
                .find(|s| &s.gedcom_id == sid)
                .map(|s| s.citation.is_some())
                .unwrap_or(false)
        });
        if has_citation {
            conf += 0.15;
        } else {
            conf += 0.08;
        }
    }
    // findings affect confidence: anomalies reduce confidence
    let anomaly_count = findings
        .iter()
        .filter(|f| {
            f.person_id.as_deref() == Some(&person.gedcom_id)
                && (f.kind == "chronology"
                    || f.kind == "AGE_ANOMALY"
                    || f.kind == "RELATIONSHIP_ANOMALY")
        })
        .count();
    if anomaly_count > 0 {
        conf -= 0.1 * anomaly_count as f32;
    }
    conf.clamp(0.1, 0.98)
}

fn is_direct_ancestor(person: &Person, tree: &GenealogyTree) -> bool {
    // A person is a direct ancestor if they have descendants
    // or they are on an ancestor path of any leaf person.
    let has_descendants = tree.families.iter().any(|f| {
        f.husband_id.as_deref() == Some(&person.gedcom_id)
            || f.wife_id.as_deref() == Some(&person.gedcom_id)
    });
    if has_descendants {
        return true;
    }
    // Also if they appear as parent somewhere, even without children list?
    // fallback: check if anyone has this person as ancestor
    for other in &tree.persons {
        if other.gedcom_id == person.gedcom_id {
            continue;
        }
        if tree
            .ancestors(&other.gedcom_id)
            .iter()
            .any(|(_, pid)| pid == &person.gedcom_id)
        {
            return true;
        }
    }
    false
}

pub fn opportunities(tree: &GenealogyTree, findings: &[Finding]) -> Vec<ResearchOpportunity> {
    let mut out = Vec::new();
    // To deduplicate opportunities, we already group per person.
    // Collect findings per person quickly
    for person in &tree.persons {
        let person_findings: Vec<&Finding> = findings
            .iter()
            .filter(|f| f.person_id.as_deref() == Some(&person.gedcom_id))
            .collect();

        let mut components: Vec<ScoreComponent> = Vec::new();
        let mut missing: Vec<String> = Vec::new();
        let mut what_known: Vec<String> = Vec::new();
        let why: String;
        // --- Genealogical importance ---
        let direct = is_direct_ancestor(person, tree);
        if direct {
            components.push(ScoreComponent {
                name: "Direct ancestor".into(),
                points: 30,
                reason: "Persona en línea ancestral directa; avanzar aquí desbloquea generaciones."
                    .into(),
            });
            why = "Antepasado directo y posible punto de bloqueo de la línea.".into();
        } else if !person.family_spouse.is_empty() {
            components.push(ScoreComponent {
                name: "Collateral branch".into(),
                points: 12,
                reason: "Rama colateral con descendencia; relevante pero no troncal.".into(),
            });
            why = "Rama colateral con potencial para completar contexto familiar.".into();
        } else {
            why = "Persona relevante para completar huecos de información.".into();
        }

        // --- Missing parents ---
        if person.family_child.is_none() {
            components.push(ScoreComponent {
                name: "Missing parent".into(),
                points: 20,
                reason: "No se conocen progenitores; hueco genealógico claro.".into(),
            });
            missing.push("progenitores".into());
        } else {
            // check if family exists but missing one parent
            if let Some(fid) = &person.family_child {
                if let Some(f) = tree.family(fid) {
                    if f.husband_id.is_none() || f.wife_id.is_none() {
                        components.push(ScoreComponent {
                            name: "Missing one parent".into(),
                            points: 12,
                            reason: "Falta uno de los progenitores en la familia.".into(),
                        });
                        missing.push("un progenitor".into());
                    }
                }
            }
        }

        // --- Known locality ---
        if let Some(place) = &person.birth_place {
            components.push(ScoreComponent {
                name: "Known locality".into(),
                points: 15,
                reason: format!("Lugar conocido: {place} permite focalizar archivos."),
            });
            what_known.push(format!("Birth: {}", place));
        }

        // --- Birth date ---
        if let Some(d) = &person.birth_date {
            let label = match d.precision {
                DatePrecision::About => "Approximate birth date",
                DatePrecision::Before => "Bounded birth date (before)",
                DatePrecision::After => "Bounded birth date (after)",
                DatePrecision::Between | DatePrecision::FromTo => "Interval birth date",
                DatePrecision::Exact | DatePrecision::Year | DatePrecision::Unknown => {
                    "Known birth date"
                }
            };
            components.push(ScoreComponent {
                name: label.into(),
                points: 10,
                reason: format!("Fecha conocida {} facilita búsqueda en registros.", d.raw),
            });
            what_known.push(format!("Birth date: {}", d.raw));
            if person.birth_date.is_none() {
                missing.push("fecha de nacimiento".into());
            }
        } else {
            missing.push("fecha de nacimiento".into());
        }

        // --- Known marriage / death ---
        if let Some(fam_id) = person.family_spouse.first() {
            if let Some(fam) = tree.family(fam_id) {
                if let Some(d) = &fam.marriage_date {
                    what_known.push(format!("Marriage: {}", d.raw));
                }
            }
        }
        if let Some(d) = &person.death_date {
            what_known.push(format!("Death: {}", d.raw));
        }
        // children count
        let child_count: usize = tree
            .families
            .iter()
            .filter(|f| {
                f.husband_id.as_deref() == Some(&person.gedcom_id)
                    || f.wife_id.as_deref() == Some(&person.gedcom_id)
            })
            .map(|f| f.children.len())
            .sum();
        if child_count > 0 {
            what_known.push(format!("Children: {child_count}"));
        }

        // --- Existing related evidence / findings ---
        if !person_findings.is_empty() {
            // cap at 10 points
            components.push(ScoreComponent {
                name: "Existing related evidence".into(),
                points: 10,
                reason: format!(
                    "{} hallazgo(s) asociados que requieren verificación o ampliación.",
                    person_findings.len()
                ),
            });
        }

        // --- Source gap (research potential) ---
        if person.sources.is_empty() {
            components.push(ScoreComponent {
                name: "No source linked".into(),
                points: 8,
                reason: "Sin fuentes vinculadas; oportunidad de documentar.".into(),
            });
            missing.push("fuente documental".into());
        } else {
            let has_citation = person.sources.iter().any(|sid| {
                tree.sources
                    .iter()
                    .find(|s| &s.gedcom_id == sid)
                    .map(|s| s.citation.is_some())
                    .unwrap_or(false)
            });
            if has_citation {
                // good source, less urgent but still opportunity
                components.push(ScoreComponent {
                    name: "Source with citation".into(),
                    points: 2,
                    reason: "Existe cita concreta; se puede profundizar.".into(),
                });
            } else {
                components.push(ScoreComponent {
                    name: "Generic source only".into(),
                    points: 5,
                    reason: "Fuente genérica sin cita específica; mejorar calidad documental."
                        .into(),
                });
            }
        }

        // --- Researchability ---
        let researchability = researchability_for(person);
        match researchability {
            Researchability::High => components.push(ScoreComponent {
                name: "High researchability".into(),
                points: 9,
                reason: "Lugar y fecha suficientes para búsqueda en archivos.".into(),
            }),
            Researchability::Medium => components.push(ScoreComponent {
                name: "Medium researchability".into(),
                points: 5,
                reason: "Información parcial; investigable con esfuerzo.".into(),
            }),
            Researchability::Low => components.push(ScoreComponent {
                name: "Low researchability".into(),
                points: 1,
                reason: "Poca información contextual; investigación difícil.".into(),
            }),
        }

        // If no components? shouldn't happen but handle
        if components.is_empty() {
            continue;
        }

        // Calculate total with cap 0-100
        let raw_total: i32 = components.iter().map(|c| c.points).sum();
        let total = raw_total.clamp(0, 100) as u8;

        // Filter trivial low scores? Keep threshold >0 (already). Avoid 5 redundant opportunities: we already have one per person.
        // But skip very low scores <10 if not missing parents? keep all with missing parents or score >=15
        if total < 15 && !missing.contains(&"progenitores".to_string()) {
            // still allow if has other missing
            if missing.is_empty() {
                continue;
            }
        }

        // Confidence separate
        let confidence = confidence_for(person, tree, findings);

        let priority = if total >= 85 {
            Severity::Critical
        } else if total >= 65 {
            Severity::High
        } else if total >= 35 {
            Severity::Medium
        } else {
            Severity::Low
        };

        // Suggested sources based on what is missing/known
        let mut suggested = Vec::new();
        let mut potential = Vec::new();
        if missing
            .iter()
            .any(|m| m.contains("progenitores") || m.contains("progenitor"))
        {
            suggested.push("registros parroquiales".into());
            suggested.push("matrimonios".into());
            potential.push("Baptism".into());
            potential.push("Marriage".into());
            potential.push("Parish register".into());
        }
        if person.birth_place.is_some() {
            suggested.push("archivos locales".into());
            suggested.push("matrimonios".into());
        }
        if person
            .birth_date
            .as_ref()
            .map(|d| d.approximate)
            .unwrap_or(false)
        {
            suggested.push("padrones".into());
        }
        if child_count > 0 {
            suggested.push("protocolos notariales".into());
            suggested.push("matrimonios".into());
        }
        if suggested.is_empty() {
            suggested = vec![
                "registros parroquiales".into(),
                "matrimonios".into(),
                "padrones".into(),
                "protocolos notariales".into(),
            ];
        }
        // Dedup
        suggested.sort();
        suggested.dedup();
        potential.sort();
        potential.dedup();
        if potential.is_empty() {
            potential = vec!["Parish register".into(), "Civil registry".into()];
        }

        let reasons = components
            .iter()
            .map(|c| format!("+{} {}", c.points, c.name))
            .collect();

        let breakdown = ScoreBreakdown {
            total,
            components: components.clone(),
        };

        // Enrich missing: also check death, marriage
        if person.death_date.is_none() {
            // only add if not too recent? but add
            missing.push("fecha de defunción".into());
        }
        missing.sort();
        missing.dedup();

        // Deduplicate opportunities: Ensure we don't create duplicate for same problem
        // Already one per person handled.

        out.push(ResearchOpportunity {
            person_id: person.gedcom_id.clone(),
            score: total,
            confidence,
            priority,
            researchability,
            breakdown,
            reasons,
            suggested_sources: suggested.clone(),
            missing_information: missing,
            why_it_matters: why,
            what_is_known: what_known,
            potential_sources: potential,
        });
    }

    // Sort by score descending, then confidence
    out.sort_by(|a, b| {
        b.score
            .cmp(&a.score)
            .then_with(|| b.confidence.partial_cmp(&a.confidence).unwrap())
    });
    // Deduplicate by person_id (should already be unique)
    let mut seen = HashSet::new();
    out.retain(|o| seen.insert(o.person_id.clone()));
    out
}
