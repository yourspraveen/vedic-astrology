//! Optional AI writer: any OpenAI-compatible chat-completions service (OpenAI, Azure-style
//! gateways, Ollama, LM Studio, vLLM...). It only rewrites and expands readings the rule engine has
//! already produced, using chart facts and passages retrieved from the knowledge folder; it never
//! calculates positions. Nothing is sent anywhere unless the person enables it.

use serde::{Deserialize, Serialize};
use serde_json::json;
use crate::knowledge;
use crate::report::AiTask;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Mutex;
use std::time::Duration;

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
#[serde(default)]
pub struct LlmConfig {
    pub enabled: bool,
    /// e.g. https://api.openai.com/v1 or http://localhost:11434/v1
    pub base_url: String,
    pub api_key: String,
    pub model: String,
    /// Optional end-user id sent as the `user` field.
    pub user: String,
    /// Optional organisation id sent as the OpenAI-Organization header.
    pub organization: String,
    pub temperature: f32,
    pub max_tokens: u32,
    /// Knowledge folder used for retrieval (and rule overrides).
    pub knowledge_dir: String,
    /// Passages sent with each section.
    pub passages: usize,
}

impl Default for LlmConfig {
    fn default() -> Self {
        LlmConfig {
            enabled: false,
            base_url: "https://api.openai.com/v1".into(),
            api_key: String::new(),
            model: "gpt-4o-mini".into(),
            user: String::new(),
            organization: String::new(),
            temperature: 0.4,
            // A reasoning model spends part of this budget on hidden "thinking" tokens, so a low cap
            // truncates the visible answer mid-sentence. Leave generous headroom for both.
            max_tokens: 2000,
            knowledge_dir: crate::knowledge::default_dir().display().to_string(),
            passages: 4,
        }
    }
}

/// One chat-completions call; returns the reply text and the service's `finish_reason`
/// ("stop" when the model finished on its own, "length" when the token budget cut it off,
/// or None when the gateway omits it).
fn chat_once(cfg: &LlmConfig, system: &str, user_msg: &str) -> Result<(String, Option<String>), String> {
    let url = format!("{}/chat/completions", cfg.base_url.trim_end_matches('/'));
    let mut body = json!({
        "model": cfg.model,
        "messages": [
            {"role": "system", "content": system},
            {"role": "user", "content": user_msg}
        ],
        "temperature": cfg.temperature,
        "max_tokens": cfg.max_tokens,
    });
    if !cfg.user.trim().is_empty() {
        body["user"] = json!(cfg.user.trim());
    }
    let agent: ureq::Agent = ureq::Agent::config_builder().timeout_global(Some(Duration::from_secs(180))).http_status_as_error(false).build().into();
    let mut req = agent.post(&url).header("Content-Type", "application/json");
    if !cfg.api_key.trim().is_empty() {
        req = req.header("Authorization", &format!("Bearer {}", cfg.api_key.trim()));
    }
    if !cfg.organization.trim().is_empty() {
        req = req.header("OpenAI-Organization", cfg.organization.trim());
    }
    let resp = req.send_json(&body).map_err(|e| format!("could not reach {url}: {e}"))?;
    let status = resp.status().as_u16();
    let value: serde_json::Value = resp.into_body().read_json().map_err(|e| format!("unreadable reply ({status}): {e}"))?;
    if status >= 400 {
        let msg = value["error"]["message"].as_str().unwrap_or("request refused");
        return Err(format!("{status}: {msg}"));
    }
    let choice = &value["choices"][0];
    let text = choice["message"]["content"].as_str().map(|s| s.trim().to_string()).ok_or_else(|| "the reply had no text".to_string())?;
    let finish = choice["finish_reason"].as_str().map(str::to_string);
    Ok((text, finish))
}

/// One chat-completions call; returns the reply text.
pub fn chat(cfg: &LlmConfig, system: &str, user_msg: &str) -> Result<String, String> {
    chat_once(cfg, system, user_msg).map(|(text, _)| text)
}

/// True when a reply looks dropped or cut off: empty, reported cut by the token budget
/// (`finish_reason` "length"), or — when the service does not say why it stopped — ending
/// mid-sentence instead of on terminal punctuation. When the service explicitly says it
/// finished ("stop"/"end_turn"), we trust it even if the text has no closing period, so a
/// legitimately short answer is not retried forever.
fn looks_dropped(text: &str, finish: Option<&str>) -> bool {
    let t = text.trim();
    if t.is_empty() {
        return true;
    }
    match finish {
        Some("length") => true,
        Some("stop") | Some("end_turn") | Some("eos") => false,
        _ => {
            let last = t.chars().rev().find(|c| !c.is_whitespace()).unwrap_or(' ');
            // A finished paragraph ends on sentence-final punctuation (allowing a closing
            // quote/bracket, the Devanagari danda, an ellipsis, or a trailing rule-id ']' / '*').
            !".!?।॥…\"')]»*".contains(last)
        }
    }
}

/// A chat call that guards against the service dropping or truncating the reply. If the first
/// answer comes back empty or cut off, it retries with more token headroom — reasoning models
/// spend part of the budget on hidden "thinking", so a wider cap usually lets the visible answer
/// complete — up to three attempts, and returns the fullest answer it saw rather than a fragment.
/// Used for the report body and the Ask box, where a half sentence is worse than a short wait.
pub fn chat_guarded(cfg: &LlmConfig, system: &str, user_msg: &str) -> Result<String, String> {
    let mut cfg = cfg.clone();
    let mut best = String::new();
    let mut last_err = String::new();
    for attempt in 0..3 {
        match chat_once(&cfg, system, user_msg) {
            Ok((text, finish)) => {
                if !looks_dropped(&text, finish.as_deref()) {
                    return Ok(text);
                }
                if text.trim().len() > best.trim().len() {
                    best = text;
                }
                // Give the next try room for the hidden thinking plus the full answer.
                cfg.max_tokens = cfg.max_tokens.saturating_mul(2).min(8000);
            }
            Err(e) => {
                last_err = e;
                if attempt == 2 {
                    break;
                }
            }
        }
    }
    if !best.trim().is_empty() {
        Ok(best)
    } else if !last_err.is_empty() {
        Err(last_err)
    } else {
        Err("the reply kept coming back truncated".to_string())
    }
}

/// A short request to check the settings work.
pub fn test(cfg: &LlmConfig) -> Result<String, String> {
    chat(cfg, "You answer in one short line.", "Reply with the word OK.")
}

/// The models the service offers (GET `{base_url}/models`), newest-looking first.
/// Used to fill the model picker in Settings so the person can switch without typing.
pub fn list_models(cfg: &LlmConfig) -> Result<Vec<String>, String> {
    let url = format!("{}/models", cfg.base_url.trim_end_matches('/'));
    let agent: ureq::Agent = ureq::Agent::config_builder().timeout_global(Some(Duration::from_secs(30))).http_status_as_error(false).build().into();
    let mut req = agent.get(&url).header("Accept", "application/json");
    if !cfg.api_key.trim().is_empty() {
        req = req.header("Authorization", &format!("Bearer {}", cfg.api_key.trim()));
    }
    if !cfg.organization.trim().is_empty() {
        req = req.header("OpenAI-Organization", cfg.organization.trim());
    }
    let resp = req.call().map_err(|e| format!("could not reach {url}: {e}"))?;
    let status = resp.status().as_u16();
    let value: serde_json::Value = resp.into_body().read_json().map_err(|e| format!("unreadable reply ({status}): {e}"))?;
    if status >= 400 {
        let msg = value["error"]["message"].as_str().unwrap_or("request refused");
        return Err(format!("{status}: {msg}"));
    }
    // OpenAI shape {"data":[{"id":...}]}; some gateways return a bare list or {"models":[...]}.
    let items = value["data"].as_array().or_else(|| value["models"].as_array()).or_else(|| value.as_array());
    let mut ids: Vec<String> = match items {
        Some(arr) => arr.iter().filter_map(|m| m["id"].as_str().or_else(|| m["name"].as_str()).or_else(|| m.as_str()).map(str::to_string)).collect(),
        None => return Err("the reply had no model list".into()),
    };
    if ids.is_empty() {
        return Err("the service returned no models".into());
    }
    ids.sort();
    ids.dedup();
    Ok(ids)
}

/// Progress of an AI-writer run, shared with the window.
#[derive(Default)]
pub struct Job {
    pub done: usize,
    pub total: usize,
    /// Section key → (model, text).
    pub results: HashMap<String, (String, String)>,
    pub errors: Vec<String>,
    pub finished: bool,
}

const SYSTEM: &str = "You are a warm, careful guide writing a Vedic astrology (Jyotisha) report for the person whose chart it is. Expand the rule-based reading you are given into three to five short paragraphs of plain, encouraging {lang}. Shape each section as a small guided journey: first explain the idea in everyday words, so a newcomer understands what this part of the chart actually means; then describe what the person can realistically expect from it; then give balanced, forward-looking guidance — how to make the most of the supportive influences and how to meet the harder ones constructively, through practical planning rather than fear. Keep a calm, hopeful tone and describe tendencies, never fixed fate. Use only the chart facts, the rules and the reference passages provided: do not add planetary positions, dates, yogas or doshas that are not in them, do not predict death or name specific illnesses, and give no medical, legal or financial instructions. If the section is about remedies or fasting, present them as traditional remedial practices a person may choose to follow for peace of mind and focus, not as calculated facts or obligations. Where a statement rests on a rule, put its rule id in square brackets, for example [BHAVESH-02-05]. Separate paragraphs with a blank line and reply with the paragraphs only.";

/// Words of a text worth searching for (any script; punctuation and numbers dropped).
fn query_words(text: &str) -> Vec<String> {
    let mut seen = std::collections::HashSet::new();
    text.split(|c: char| c.is_whitespace() || c.is_ascii_punctuation() || "—–‘’“”।".contains(c))
        .filter(|w| w.chars().count() > 3 && !w.chars().all(|c| c.is_ascii_digit()))
        .map(str::to_lowercase)
        .filter(|w| seen.insert(w.clone()))
        .take(80)
        .collect()
}

/// Ask the service to expand each section, with passages retrieved from the knowledge folder.
pub fn expand(cfg: &LlmConfig, tasks: &[AiTask], language: &str, job: &Mutex<Job>) {
    let dir = PathBuf::from(cfg.knowledge_dir.trim());
    // The person's editable instructions (knowledge/ai/instructions.md), else the built-in ones.
    let system = knowledge::ai_instructions("Report sections").unwrap_or_else(|| SYSTEM.to_string()).replace("{lang}", language);
    // Run several sections at once: each worker pulls the next task, calls the service and stores its
    // result, so the whole report is about as slow as its slowest few sections, not their sum.
    let next = AtomicUsize::new(0);
    let stop = AtomicBool::new(false);
    let workers = tasks.len().clamp(1, 6);
    std::thread::scope(|scope| {
        for _ in 0..workers {
            scope.spawn(|| {
                while !stop.load(Ordering::Relaxed) {
                    let i = next.fetch_add(1, Ordering::Relaxed);
                    if i >= tasks.len() {
                        break;
                    }
                    let task = &tasks[i];
                    let words = query_words(&format!("{} {}", task.title, task.facts));
                    let passages = if dir.as_os_str().is_empty() { Vec::new() } else { knowledge::search(&dir, &words, cfg.passages) };
                    let mut user = format!("Section: {}\n\nRule-based reading:\n{}\n", task.title, task.facts);
                    if !passages.is_empty() {
                        user.push_str("\nReference passages from the knowledge folder:\n");
                        for p in &passages {
                            let text: String = p.text.chars().take(1200).collect();
                            user.push_str(&format!("--- {} ---\n{}\n", p.file, text));
                        }
                    }
                    let reply = chat_guarded(cfg, &system, &user);
                    let mut j = job.lock().unwrap_or_else(|e| e.into_inner());
                    match reply {
                        Ok(text) => {
                            j.results.insert(task.key.clone(), (cfg.model.clone(), text));
                        }
                        Err(e) => j.errors.push(format!("{}: {e}", task.title)),
                    }
                    j.done += 1;
                    // Give up if the service refuses everything (bad key/model) rather than retry all.
                    if j.errors.len() >= 3 && j.results.is_empty() {
                        stop.store(true, Ordering::Relaxed);
                    }
                }
            });
        }
    });
    job.lock().unwrap_or_else(|e| e.into_inner()).finished = true;
}

/// Answer one question about a chart: the chart summary, the question and the best passages from
/// the knowledge folder go to the service with the "Questions" instructions. Returns the answer and
/// the files the passages came from.
pub fn ask(cfg: &LlmConfig, chart_summary: &str, question: &str, language: &str) -> Result<(String, Vec<String>), String> {
    let system = knowledge::ai_instructions("Questions")
        .unwrap_or_else(|| "Answer the question about this Vedic birth chart in {lang}, briefly, using only the chart summary and passages given.".to_string())
        .replace("{lang}", language);
    let dir = PathBuf::from(cfg.knowledge_dir.trim());
    let passages = if dir.as_os_str().is_empty() { Vec::new() } else { knowledge::search(&dir, &query_words(question), cfg.passages.max(3)) };
    let mut user = format!("Birth chart summary:\n{chart_summary}\n\nQuestion: {question}\n");
    if !passages.is_empty() {
        user.push_str("\nReference passages from the knowledge folder:\n");
        for p in &passages {
            let text: String = p.text.chars().take(1200).collect();
            user.push_str(&format!("--- {} ---\n{}\n", p.file, text));
        }
    }
    let files = passages.iter().map(|p| p.file.clone()).collect();
    chat_guarded(cfg, &system, &user).map(|a| (a, files))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unreachable_service_is_an_error_not_a_crash() {
        let cfg = LlmConfig { base_url: "http://127.0.0.1:9/v1".into(), api_key: "x".into(), ..Default::default() };
        assert!(chat(&cfg, "s", "u").is_err());
    }

    #[test]
    fn truncated_replies_are_detected() {
        // mid-sentence with no reason given — the reported symptom
        assert!(looks_dropped("Think of your fourth house as the", None));
        // service says the budget cut it off
        assert!(looks_dropped("A full sentence that was cut.", Some("length")));
        // nothing came back
        assert!(looks_dropped("", Some("stop")));
        // complete sentences are kept, including ones ending on a rule-id citation
        assert!(!looks_dropped("A calm, complete thought.", None));
        assert!(!looks_dropped("Supported by the rule [BHAVESH-02-05]", None));
        // when the service says it finished, trust it even without a closing period
        assert!(!looks_dropped("A short OK reply", Some("stop")));
    }

    #[test]
    fn query_words_keep_indic_text() {
        let w = query_words("House 2 — செல்வம், குடும்பம் धन, परिवार and wealth");
        assert!(w.contains(&"செல்வம்".to_string()) && w.contains(&"परिवार".to_string()) && w.contains(&"wealth".to_string()));
    }
}
