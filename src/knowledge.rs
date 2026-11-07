//! The Jyotisha knowledge base: structured rule files (lord in house, dasha readings) and any
//! source passages the person adds. Rules come from the knowledge folder when present (so they can
//! be edited without rebuilding) and otherwise from copies built into the program. The same folder
//! is searched for passages to give the optional AI writer (retrieval-augmented generation).

use crate::i18n::{self, Lang};
use serde_json::Value;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

/// Files built into the program: (path inside the knowledge folder, contents).
const EMBEDDED: [(&str, &str); 19] = [
    ("ai/instructions.md", include_str!("../knowledge/ai/instructions.md")),
    ("README.md", include_str!("../knowledge/README.md")),
    ("sources.json", include_str!("../knowledge/sources.json")),
    ("texts/README.md", include_str!("../knowledge/texts/README.md")),
    ("texts/concepts-nakshatra.md", include_str!("../knowledge/texts/concepts-nakshatra.md")),
    ("texts/concepts-gemstones.md", include_str!("../knowledge/texts/concepts-gemstones.md")),
    ("texts/concepts-remedies.md", include_str!("../knowledge/texts/concepts-remedies.md")),
    ("texts/concepts-yogas.md", include_str!("../knowledge/texts/concepts-yogas.md")),
    ("texts/concepts-bhava.md", include_str!("../knowledge/texts/concepts-bhava.md")),
    ("texts/concepts-planet-in-sign.md", include_str!("../knowledge/texts/concepts-planet-in-sign.md")),
    ("texts/concepts-compatibility.md", include_str!("../knowledge/texts/concepts-compatibility.md")),
    ("rules/en/lord_in_house.json", include_str!("../knowledge/rules/en/lord_in_house.json")),
    ("rules/ta/lord_in_house.json", include_str!("../knowledge/rules/ta/lord_in_house.json")),
    ("rules/hi/lord_in_house.json", include_str!("../knowledge/rules/hi/lord_in_house.json")),
    ("rules/en/dasha.json", include_str!("../knowledge/rules/en/dasha.json")),
    ("rules/ta/dasha.json", include_str!("../knowledge/rules/ta/dasha.json")),
    ("rules/hi/dasha.json", include_str!("../knowledge/rules/hi/dasha.json")),
    ("sanskrit/glossary.json", include_str!("../knowledge/sanskrit/glossary.json")),
    ("sanskrit/concepts.md", include_str!("../knowledge/sanskrit/concepts.md")),
];

static DIR: Mutex<Option<PathBuf>> = Mutex::new(None);
static CACHE: Mutex<Option<HashMap<String, Value>>> = Mutex::new(None);
static SUMMARY: Mutex<Option<(usize, usize, usize)>> = Mutex::new(None);

/// The default knowledge folder: ~/.vedic_astrology/knowledge.
pub fn default_dir() -> PathBuf {
    let home = std::env::var_os("HOME").or_else(|| std::env::var_os("USERPROFILE")).map(PathBuf::from).unwrap_or_else(|| PathBuf::from("."));
    home.join(".vedic_astrology").join("knowledge")
}

/// Use this folder for rules and passages (an empty path means the built-in copies only).
pub fn set_dir(dir: Option<PathBuf>) {
    *DIR.lock().unwrap_or_else(|e| e.into_inner()) = dir;
    *CACHE.lock().unwrap_or_else(|e| e.into_inner()) = None;
    *SUMMARY.lock().unwrap_or_else(|e| e.into_inner()) = None;
}

pub fn dir() -> Option<PathBuf> {
    DIR.lock().unwrap_or_else(|e| e.into_inner()).clone()
}

/// Write any built-in file that `dir` doesn't have yet (so new versions add files to an existing
/// folder). Files already there, edited or not, are never touched.
pub fn seed(dir: &Path) -> std::io::Result<()> {
    for (rel, text) in EMBEDDED {
        let path = dir.join(rel);
        if path.exists() {
            continue;
        }
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(path, text)?;
    }
    Ok(())
}

fn lang_code() -> &'static str {
    match i18n::lang() {
        Lang::En => "en",
        Lang::Ta => "ta",
        Lang::Hi => "hi",
    }
}

/// A rule file for the current language: from the knowledge folder if readable, else built in.
fn rule_file(name: &str) -> Option<Value> {
    knowledge_file(&format!("rules/{}/{}.json", lang_code(), name))
}

/// A JSON file of the knowledge base: from the knowledge folder if readable, else built in.
fn knowledge_file(rel: &str) -> Option<Value> {
    let rel = rel.to_string();
    let mut cache = CACHE.lock().unwrap_or_else(|e| e.into_inner());
    let map = cache.get_or_insert_with(HashMap::new);
    if let Some(v) = map.get(&rel) {
        return Some(v.clone());
    }
    let from_dir = dir().and_then(|d| std::fs::read_to_string(d.join(&rel)).ok()).and_then(|s| serde_json::from_str::<Value>(&s).ok());
    let value = from_dir.or_else(|| EMBEDDED.iter().find(|e| e.0 == rel).and_then(|e| serde_json::from_str(e.1).ok()))?;
    map.insert(rel, value.clone());
    Some(value)
}

/// A rule's text with its id and source label.
pub struct RuleText {
    pub id: String,
    pub text: String,
    pub source: String,
    /// True only when the rule file says it was checked against a named edition.
    pub verified: bool,
}

fn source_label(file: &Value) -> String {
    let book = file["source"]["book"].as_str().unwrap_or("");
    let chapter = file["source"]["chapter"].as_str().unwrap_or("");
    format!("{book}, {chapter}")
}

/// "Lord of house `lord_of` placed in house `placed_in`" (BHAVESH rules).
pub fn lord_in_house(lord_of: usize, placed_in: usize) -> Option<RuleText> {
    let file = rule_file("lord_in_house")?;
    let rule = file["rules"].as_array()?.iter().find(|r| r["lord_of"].as_u64() == Some(lord_of as u64) && r["placed_in"].as_u64() == Some(placed_in as u64))?;
    Some(RuleText { id: rule["id"].as_str()?.to_string(), text: rule["text"].as_str()?.to_string(), source: source_label(&file), verified: file["status"].as_str() == Some("verified") })
}

/// Mahadasha readings for a planet (English name): (general, strong, weak).
pub fn dasha(planet_en: &str) -> Option<(RuleText, String, String)> {
    let file = rule_file("dasha")?;
    let rule = file["rules"].as_array()?.iter().find(|r| r["planet"].as_str() == Some(planet_en))?;
    Some((
        RuleText { id: rule["id"].as_str()?.to_string(), text: rule["general"].as_str()?.to_string(), source: source_label(&file), verified: file["status"].as_str() == Some("verified") },
        rule["strong"].as_str().unwrap_or("").to_string(),
        rule["weak"].as_str().unwrap_or("").to_string(),
    ))
}

/// Sanskrit facts about a house from the glossary.
pub struct BhavaInfo {
    pub sanskrit: String,
    pub devanagari: String,
    /// English planet names of its significators (karakas).
    pub karakas: Vec<String>,
    /// kendra, trikona, dusthana, upachaya, maraka...
    pub classes: Vec<String>,
}

pub fn bhava(house: usize) -> Option<BhavaInfo> {
    let file = knowledge_file("sanskrit/glossary.json")?;
    let b = file["bhavas"].as_array()?.iter().find(|b| b["house"].as_u64() == Some(house as u64))?;
    let list = |k: &str| b[k].as_array().map(|a| a.iter().filter_map(|x| x.as_str().map(String::from)).collect()).unwrap_or_default();
    Some(BhavaInfo { sanskrit: b["sanskrit"].as_str()?.to_string(), devanagari: b["devanagari"].as_str().unwrap_or("").to_string(), karakas: list("karakas"), classes: list("classes") })
}

/// What the knowledge base holds now: (rule records, glossary entries, searchable passages in the folder).
pub fn summary() -> (usize, usize, usize) {
    if let Some(v) = *SUMMARY.lock().unwrap_or_else(|e| e.into_inner()) {
        return v;
    }
    let rules = ["lord_in_house", "dasha"].iter().filter_map(|n| rule_file(n)).map(|f| f["rules"].as_array().map_or(0, |a| a.len())).sum();
    let glossary = knowledge_file("sanskrit/glossary.json")
        .map(|g| ["bhavas", "grahas", "rashis", "nakshatras"].iter().map(|k| g[*k].as_array().map_or(0, |a| a.len())).sum())
        .unwrap_or(0);
    let passages = dir().map(|d| passages(&d).len()).unwrap_or(0);
    *SUMMARY.lock().unwrap_or_else(|e| e.into_inner()) = Some((rules, glossary, passages));
    (rules, glossary, passages)
}

/// The AI writer's instructions for one purpose ("Report sections" or "Questions"): the text under
/// that heading of `ai/instructions.md`, from the knowledge folder if present, else built in.
pub fn ai_instructions(section: &str) -> Option<String> {
    let text = dir()
        .and_then(|d| std::fs::read_to_string(d.join("ai/instructions.md")).ok())
        .unwrap_or_else(|| EMBEDDED.iter().find(|e| e.0 == "ai/instructions.md").map(|e| e.1.to_string()).unwrap_or_default());
    let head = format!("## {section}");
    let start = text.find(&head)? + head.len();
    let rest = &text[start..];
    let body = rest[..rest.find("\n## ").unwrap_or(rest.len())].trim();
    (!body.is_empty()).then(|| body.to_string())
}

/// Path of the editable instructions file in the knowledge folder (it may not exist yet).
pub fn ai_instructions_path() -> Option<PathBuf> {
    dir().map(|d| d.join("ai/instructions.md"))
}

// ---------------------------------------------------------------- retrieval for the AI writer

/// A passage from the knowledge folder.
pub struct Passage {
    pub file: String,
    pub text: String,
    score: f64,
}

/// Split every .md, .txt and .json file under `root` into passages.
fn passages(root: &Path) -> Vec<Passage> {
    let mut out = Vec::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(d) = stack.pop() {
        let Ok(entries) = std::fs::read_dir(&d) else { continue };
        for e in entries.flatten() {
            let p = e.path();
            if p.is_dir() {
                stack.push(p);
                continue;
            }
            let ext = p.extension().and_then(|x| x.to_str()).unwrap_or("").to_lowercase();
            let Ok(text) = std::fs::read_to_string(&p) else { continue };
            let name = p.strip_prefix(root).unwrap_or(&p).display().to_string();
            // The AI's own instructions are not reference material.
            if name.starts_with("ai/") || name.starts_with("ai\\") {
                continue;
            }
            if ext == "json" {
                // One passage per rule object, so a search can find a single rule.
                if let Ok(v) = serde_json::from_str::<Value>(&text)
                    && let Some(rules) = v["rules"].as_array() {
                        for r in rules {
                            out.push(Passage { file: name.clone(), text: r.to_string(), score: 0.0 });
                        }
                        continue;
                    }
                out.push(Passage { file: name, text: text.chars().take(1500).collect(), score: 0.0 });
            } else if ext == "md" || ext == "txt" {
                let mut chunk = String::new();
                for para in text.split("\n\n") {
                    if chunk.len() + para.len() > 1200 && !chunk.is_empty() {
                        out.push(Passage { file: name.clone(), text: std::mem::take(&mut chunk), score: 0.0 });
                    }
                    chunk.push_str(para);
                    chunk.push_str("\n\n");
                }
                if !chunk.trim().is_empty() {
                    out.push(Passage { file: name.clone(), text: chunk, score: 0.0 });
                }
            }
        }
    }
    out
}

/// The `k` passages that best match the query words (simple word-overlap scoring, rarer words count more).
/// Favor our curated interpretive primers and rule records over the raw public-domain astronomy
/// source texts (Panchasiddhantika, Surya Siddhanta…). Those are large and full of generic words,
/// so on a plain-language question like "where does yoni matter" they otherwise win the match and
/// bury the interpretive content. This nudges ranking by provenance without excluding anything.
fn provenance_weight(file: &str) -> f64 {
    let f = file.to_lowercase();
    if f.contains("concepts-") {
        1.6
    } else if f.contains("rules") || f.contains("glossary") || f.contains("readings") || f.contains("sanskrit/concepts") {
        1.3
    } else if ["panchasiddhantika", "surya-siddhanta", "siddhanta", "aryabhat", "ahargana"].iter().any(|kw| f.contains(kw)) {
        0.35
    } else {
        1.0
    }
}

pub fn search(root: &Path, words: &[String], k: usize) -> Vec<Passage> {
    let mut all = passages(root);
    if all.is_empty() || words.is_empty() {
        return Vec::new();
    }
    let lower: Vec<String> = all.iter().map(|p| p.text.to_lowercase()).collect();
    let n = all.len() as f64;
    let weights: Vec<(String, f64)> = words
        .iter()
        .map(|w| w.to_lowercase())
        .filter(|w| w.chars().count() > 2)
        .map(|w| {
            let df = lower.iter().filter(|t| t.contains(&w)).count() as f64;
            let idf = ((n + 1.0) / (df + 1.0)).ln() + 0.1;
            (w, idf)
        })
        .collect();
    for (p, t) in all.iter_mut().zip(&lower) {
        let base: f64 = weights.iter().map(|(w, idf)| t.matches(w.as_str()).count().min(3) as f64 * idf).sum();
        p.score = base * provenance_weight(&p.file);
    }
    all.retain(|p| p.score > 0.0);
    all.sort_by(|a, b| b.score.total_cmp(&a.score));
    all.truncate(k);
    all
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_language_has_all_144_lord_rules() {
        for lang in [Lang::En, Lang::Ta, Lang::Hi] {
            i18n::set_lang(lang);
            for a in 1..=12 {
                for b in 1..=12 {
                    let r = lord_in_house(a, b).unwrap_or_else(|| panic!("{lang:?} {a}->{b}"));
                    assert_eq!(r.id, format!("BHAVESH-{a:02}-{b:02}"));
                }
            }
            for p in ["Sun", "Moon", "Mars", "Mercury", "Jupiter", "Venus", "Saturn", "Rahu", "Ketu"] {
                let (g, s, w) = dasha(p).unwrap_or_else(|| panic!("{lang:?} {p}"));
                assert!(!g.text.is_empty() && !s.is_empty() && !w.is_empty());
            }
        }
        i18n::set_lang(Lang::En);
    }

    #[test]
    fn instructions_have_both_sections() {
        let r = ai_instructions("Report sections").unwrap();
        let q = ai_instructions("Questions").unwrap();
        assert!(r.contains("{lang}") && r.contains("Do not invent") && !r.contains("## Questions"));
        assert!(q.contains("{lang}") && q.contains("question"));
    }

    #[test]
    fn glossary_names_every_bhava() {
        for h in 1..=12 {
            let b = bhava(h).unwrap();
            assert!(b.sanskrit.ends_with("Bhava") && !b.devanagari.is_empty() && !b.karakas.is_empty());
        }
        assert!(bhava(1).unwrap().classes.contains(&"kendra".to_string()));
    }

    #[test]
    fn search_finds_matching_rule() {
        let dir = std::env::temp_dir().join(format!("fj_kb_test_{}", std::process::id()));
        let _ = seed(&dir);
        let hits = search(&dir, &["Saturn".into(), "dasa".into(), "discipline".into()], 3);
        assert!(hits.iter().any(|h| h.text.contains("DASHA-SATURN")), "{:?}", hits.iter().map(|h| &h.file).collect::<Vec<_>>());
    }
}
