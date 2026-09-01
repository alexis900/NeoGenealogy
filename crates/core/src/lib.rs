use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DateValue {
    pub raw: String,
    pub year: Option<i32>,
    pub date: Option<NaiveDate>,
    pub approximate: bool,
}

impl DateValue {
    pub fn year(year: i32) -> Self { Self { raw: year.to_string(), year: Some(year), date: None, approximate: false } }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RawTag { pub level: u8, pub tag: String, pub value: String }

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Person {
    pub id: usize, pub gedcom_id: String, pub given_name: String, pub surname: String,
    pub sex: Option<String>, pub birth_date: Option<DateValue>, pub birth_place: Option<String>,
    pub death_date: Option<DateValue>, pub death_place: Option<String>, pub occupation: Option<String>,
    pub notes: Vec<String>, pub confidence: Option<f32>, pub family_child: Option<String>,
    pub family_spouse: Vec<String>, pub sources: Vec<String>, pub raw: Vec<RawTag>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Family {
    pub id: usize, pub gedcom_id: String, pub husband_id: Option<String>, pub wife_id: Option<String>,
    pub children: Vec<String>, pub marriage_date: Option<DateValue>, pub marriage_place: Option<String>,
    pub sources: Vec<String>, pub raw: Vec<RawTag>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Event { pub id: usize, pub person_id: Option<String>, pub family_id: Option<String>, pub event_type: String, pub date: Option<DateValue>, pub place: Option<String>, pub description: Option<String>, pub sources: Vec<String> }

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Source { pub id: usize, pub gedcom_id: String, pub title: String, pub author: Option<String>, pub repository: Option<String>, pub url: Option<String>, pub citation: Option<String>, pub confidence: Option<f32> }

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Finding { pub id: String, pub kind: String, pub severity: Severity, pub person_id: Option<String>, pub related_person_id: Option<String>, pub description: String, pub evidence: Vec<String>, pub confidence: f32 }

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum Severity { Critical, High, Medium, Low }
impl Default for Severity { fn default() -> Self { Self::Low } }

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ResearchOpportunity { pub person_id: String, pub score: u8, pub priority: Severity, pub reasons: Vec<String>, pub suggested_sources: Vec<String>, pub missing_information: Vec<String> }

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GenealogyTree { pub persons: Vec<Person>, pub families: Vec<Family>, pub events: Vec<Event>, pub sources: Vec<Source>, pub places: Vec<Place> }

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Place { pub id: usize, pub name: String, pub normalized_name: String, pub latitude: Option<f64>, pub longitude: Option<f64>, pub historical_names: Vec<String> }

impl GenealogyTree {
    pub fn person(&self, id: &str) -> Option<&Person> { self.persons.iter().find(|p| p.gedcom_id == id) }
    pub fn family(&self, id: &str) -> Option<&Family> { self.families.iter().find(|f| f.gedcom_id == id) }
    pub fn ancestors(&self, id: &str) -> Vec<(usize, String)> { let mut out=Vec::new(); self.ancestor_walk(id,0,&mut out); out }
    fn ancestor_walk(&self,id:&str,depth:usize,out:&mut Vec<(usize,String)>) { if let Some(p)=self.person(id) { out.push((depth,id.to_string())); if let Some(fid)=&p.family_child { if let Some(f)=self.family(fid) { for parent in [&f.husband_id,&f.wife_id].into_iter().flatten() { self.ancestor_walk(parent,depth+1,out); } } } } }
}

