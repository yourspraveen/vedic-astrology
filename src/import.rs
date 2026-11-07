//! Download classical texts into the knowledge folder's `texts/`, where the AI writer's search finds
//! them. Two sources:
//!
//! * **jyotisha-corpus** (github.com/maximally0/jyotisha-corpus): public-domain editions (pre-1929)
//!   split into passages, and its extracted rule records. Data licence CC BY 4.0, code MIT; the
//!   texts themselves are public domain. Redistributable with attribution, so the Docker image
//!   includes them.
//! * **indike.org BPHS** (Brihat Parashara Hora Shastra, Sanskrit with English): only on request,
//!   for the person's own study. Its Sanskrit e-text comes from sanskritdocuments.org, whose terms
//!   allow personal study and research but not copying or redistribution, so it goes into
//!   `texts/personal/` with a notice and is never packaged.

use std::path::Path;
use std::time::Duration;

const CORPUS_RAW: &str = "https://raw.githubusercontent.com/maximally0/jyotisha-corpus/HEAD";
/// Public-domain texts in the corpus (`chunks/<id>.jsonl`), as listed in its LEGAL.md.
const CORPUS_TEXTS: [(&str, &str); 10] = [
    ("TXT-brihat-jataka-iyer1905", "Brihat Jataka (Varahamihira), tr. N. Chidambaram Iyer, 1905"),
    ("TXT-brihat-jataka-rao1919", "Brihat Jataka (Varahamihira), tr. V. Subrahmanya Sastri / Rao, 1919"),
    ("TXT-brihat-samhita-iyer1884", "Brihat Samhita (Varahamihira), tr. N. Chidambaram Iyer, 1884"),
    ("TXT-hindu-book-astrology-1902", "The Hindu Book of Astrology, 1902"),
    ("TXT-laghu-parashari-jha1900", "Laghu Parashari, ed. Jha, 1900 (Sanskrit/Hindi OCR)"),
    ("TXT-panchasiddhantika-1882", "Panchasiddhantika (Varahamihira), 1882"),
    ("TXT-panchasiddhantika-dvivedi1889", "Panchasiddhantika, ed. Dvivedi, 1889"),
    ("TXT-panchasiddhantika-thibaut1889", "Panchasiddhantika, tr. Thibaut, 1889"),
    ("TXT-saravali-1907", "Saravali (Kalyanavarma), 1907"),
    ("TXT-surya-siddhanta-burgess1860", "Surya Siddhanta, tr. Ebenezer Burgess, 1860"),
];
const BPHS_INDEX: &str = "https://indike.org/pramana/in.astro.hora.yavana.brhatparasara";

fn agent() -> ureq::Agent {
    ureq::Agent::config_builder().timeout_global(Some(Duration::from_secs(60))).build().into()
}

fn get(agent: &ureq::Agent, url: &str) -> Result<String, String> {
    let resp = agent.get(url).header("User-Agent", "VedicAstrology/0.1 (knowledge import)").call().map_err(|e| format!("{url}: {e}"))?;
    resp.into_body().with_config().limit(64 * 1024 * 1024).read_to_string().map_err(|e| format!("{url}: {e}"))
}

/// What an import did.
#[derive(Default, Debug)]
pub struct Summary {
    pub files: usize,
    pub bytes: usize,
    pub errors: Vec<String>,
}

/// Import into `knowledge_dir` (the folder holding `texts/`). `personal` also fetches the BPHS
/// chapters for personal study, at most `max_chapters` of them. `progress` gets a line per step.
pub fn import(knowledge_dir: &Path, personal: bool, max_chapters: usize, progress: &mut dyn FnMut(String)) -> Summary {
    let mut sum = Summary::default();
    let _ = crate::knowledge::seed(knowledge_dir);
    let agent = agent();
    let corpus_dir = knowledge_dir.join("texts/jyotisha-corpus");
    let write = |path: &Path, text: &str, sum: &mut Summary| {
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        match std::fs::write(path, text) {
            Ok(()) => {
                sum.files += 1;
                sum.bytes += text.len();
            }
            Err(e) => sum.errors.push(format!("{}: {e}", path.display())),
        }
    };

    for (i, (id, title)) in CORPUS_TEXTS.iter().enumerate() {
        progress(format!("jyotisha-corpus {}/{}: {title}", i + 1, CORPUS_TEXTS.len()));
        match get(&agent, &format!("{CORPUS_RAW}/chunks/{id}.jsonl")) {
            Ok(jsonl) => write(&corpus_dir.join(format!("{id}.md")), &chunks_to_markdown(id, title, &jsonl), &mut sum),
            Err(e) => sum.errors.push(e),
        }
    }
    progress("jyotisha-corpus: rule records".into());
    match get(&agent, &format!("{CORPUS_RAW}/data/rules.json")).and_then(|t| rules_file(&t)) {
        Ok(json) => write(&corpus_dir.join("rules.json"), &json, &mut sum),
        Err(e) => sum.errors.push(e),
    }
    write(&knowledge_dir.join("texts/SOURCES.md"), SOURCES_MD, &mut sum);

    if personal {
        let dir = knowledge_dir.join("texts/personal/bphs-indike");
        write(&dir.join("README.txt"), PERSONAL_NOTICE, &mut sum);
        match get(&agent, BPHS_INDEX) {
            Ok(index) => {
                let chapters = chapter_numbers(&index);
                for (k, n) in chapters.iter().take(max_chapters).enumerate() {
                    progress(format!("BPHS (personal study) chapter {n} ({}/{})", k + 1, chapters.len().min(max_chapters)));
                    match get(&agent, &format!("{BPHS_INDEX}.{n}")) {
                        Ok(page) => {
                            let text = format!("Brihat Parashara Hora Shastra, chapter {n}\nSource: {BPHS_INDEX}.{n}\nPersonal study only; do not share or package (see README.txt).\n\n{}", html_to_text(&page));
                            write(&dir.join(format!("chapter-{n:03}.txt")), &text, &mut sum);
                        }
                        Err(e) => sum.errors.push(e),
                    }
                    // Be gentle with a small volunteer-run site.
                    std::thread::sleep(Duration::from_millis(400));
                }
            }
            Err(e) => sum.errors.push(e),
        }
    }
    progress(format!("done: {} files, {} KB", sum.files, sum.bytes / 1024));
    sum
}

/// One corpus text (JSON lines of chunks) as Markdown passages with its attribution.
fn chunks_to_markdown(id: &str, title: &str, jsonl: &str) -> String {
    let mut out = format!(
        "# {title}\n\nSource: jyotisha-corpus ({CORPUS_RAW}/chunks/{id}.jsonl), a public-domain edition. \
Corpus data © its authors, CC BY 4.0 (https://creativecommons.org/licenses/by/4.0/). \
Changed by Vedic Astrology: chunks joined into passages; OCR text kept as is.\n\n"
    );
    for line in jsonl.lines().filter(|l| !l.trim().is_empty()) {
        if let Ok(v) = serde_json::from_str::<serde_json::Value>(line) {
            let text = v["text"].as_str().unwrap_or("").trim();
            if !text.is_empty() {
                out.push_str(&format!("## {}\n\n{}\n\n", v["chunk_id"].as_str().unwrap_or(id), text));
            }
        }
    }
    out
}

/// The corpus's rule records wrapped so the knowledge search treats each rule as one passage.
fn rules_file(text: &str) -> Result<String, String> {
    let rules: serde_json::Value = serde_json::from_str(text).map_err(|e| format!("rules.json: {e}"))?;
    let wrapped = serde_json::json!({
        "schema": "jyotisha-corpus-rules",
        "source": {"name": "jyotisha-corpus data/rules.json", "url": format!("{CORPUS_RAW}/data/rules.json"), "license": "CC BY 4.0", "attribution": "maximally0/jyotisha-corpus contributors"},
        "note": "Changed by Vedic Astrology: wrapped in a 'rules' object; records unchanged.",
        "rules": rules,
    });
    serde_json::to_string_pretty(&wrapped).map_err(|e| e.to_string())
}

/// Chapter numbers linked from the BPHS index page.
fn chapter_numbers(index_html: &str) -> Vec<u32> {
    let key = "in.astro.hora.yavana.brhatparasara.";
    let mut n: Vec<u32> = index_html.match_indices(key).filter_map(|(i, _)| index_html[i + key.len()..].split(|c: char| !c.is_ascii_digit()).next()?.parse().ok()).collect();
    n.sort_unstable();
    n.dedup();
    n
}

/// Readable text of an HTML page: scripts and styles dropped, tags turned into line breaks,
/// common entities decoded, blank lines and the site's navigation removed.
pub fn html_to_text(html: &str) -> String {
    let mut s = html.to_string();
    for tag in ["script", "style", "noscript", "svg"] {
        while let Some(a) = s.find(&format!("<{tag}")) {
            let close = format!("</{tag}>");
            match s[a..].find(&close) {
                Some(b) => s.replace_range(a..a + b + close.len(), "\n"),
                None => {
                    s.truncate(a);
                    break;
                }
            }
        }
    }
    let mut out = String::with_capacity(s.len() / 2);
    let mut in_tag = false;
    for ch in s.chars() {
        match ch {
            '<' => in_tag = true,
            '>' if in_tag => {
                in_tag = false;
                out.push('\n');
            }
            _ if !in_tag => out.push(ch),
            _ => {}
        }
    }
    let out = decode_entities(&out);
    const NAV: [&str; 6] = ["Pramāṇa", "·", "Search", "About", "API", "Pramana"];
    out.lines().map(str::trim).filter(|l| !l.is_empty() && !NAV.contains(l)).collect::<Vec<_>>().join("\n")
}

fn decode_entities(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut rest = s;
    while let Some(i) = rest.find('&') {
        out.push_str(&rest[..i]);
        let tail = &rest[i..];
        let end = tail.find(';').filter(|&e| e <= 10);
        let decoded = end.and_then(|e| {
            let ent = &tail[1..e];
            let ch = match ent {
                "amp" => Some('&'),
                "lt" => Some('<'),
                "gt" => Some('>'),
                "quot" => Some('"'),
                "apos" | "#39" => Some('\''),
                "nbsp" => Some(' '),
                _ if ent.starts_with("#x") || ent.starts_with("#X") => u32::from_str_radix(&ent[2..], 16).ok().and_then(char::from_u32),
                _ if ent.starts_with('#') => ent[1..].parse().ok().and_then(char::from_u32),
                _ => None,
            }?;
            Some((ch, e + 1))
        });
        match decoded {
            Some((ch, len)) => {
                out.push(ch);
                rest = &tail[len..];
            }
            None => {
                out.push('&');
                rest = &tail[1..];
            }
        }
    }
    out.push_str(rest);
    out
}

const SOURCES_MD: &str = "# Texts in this folder

* `jyotisha-corpus/`: public-domain editions of classical Jyotisha works (Brihat Jataka, Brihat
  Samhita, Saravali, Laghu Parashari, Surya Siddhanta, Panchasiddhantika, The Hindu Book of
  Astrology), split into passages, and `rules.json`, the corpus's extracted rule records.
  Source: https://github.com/maximally0/jyotisha-corpus. Data licence: Creative Commons
  Attribution 4.0 (https://creativecommons.org/licenses/by/4.0/); code licence MIT; the source
  texts are public domain (pre-1929). Vedic Astrology changed the format only (JSON lines to Markdown;
  rules wrapped in an object).
* `personal/`: present only if you asked for texts for personal study. Do not share or package it;
  see the README.txt inside.

Add your own passages as .md or .txt files anywhere under `texts/`; record their source and rights
in `../sources.json`.
";

const PERSONAL_NOTICE: &str = "Brihat Parashara Hora Shastra (Sanskrit with English), fetched from indike.org
(https://indike.org/pramana/in.astro.hora.yavana.brhatparasara) at your request.

indike.org states that its Sanskrit e-text comes from sanskritdocuments.org, whose terms say the
texts are prepared by volunteers for personal study and research and are not to be copied or
redistributed. Keep this folder for your own study: do not share it, publish it, or build it into
a Docker image or other package. Vedic Astrology's Docker image does not include it; mount your own
knowledge folder if you want the self-hosted version to use it.
";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn html_becomes_clean_text() {
        let page = "<html><head><style>p{}</style><script>var x=1;</script></head><body><nav>Pramāṇa</nav><h1>Chapter&nbsp;3</h1><p>kathita&#7747; &amp; more</p></body></html>";
        assert_eq!(html_to_text(page), "Chapter 3\nkathitaṃ & more");
    }

    #[test]
    fn chapter_links_are_found_in_order() {
        let idx = r#"<a href="/pramana/in.astro.hora.yavana.brhatparasara.10">10</a><a href="/pramana/in.astro.hora.yavana.brhatparasara.2">2</a><a href="/pramana/in.astro.hora.yavana.brhatparasara.2">2</a>"#;
        assert_eq!(chapter_numbers(idx), vec![2, 10]);
    }

    #[test]
    fn corpus_chunks_and_rules_convert() {
        let md = chunks_to_markdown("TXT-x", "X", "{\"chunk_id\":\"TXT-x-ch0001\",\"text\":\"Sun in the 1st\"}\n");
        assert!(md.contains("CC BY 4.0") && md.contains("## TXT-x-ch0001\n\nSun in the 1st"));
        let r = rules_file("[{\"rule_id\":\"JYO-1\"}]").unwrap();
        assert!(r.contains("\"rules\"") && r.contains("JYO-1"));
    }

    /// Needs the network: `cargo test live_import -- --ignored` (writes under target/).
    #[test]
    #[ignore]
    fn live_import() {
        let dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("target/kb-import-test");
        let sum = import(&dir, true, 2, &mut |l| println!("{l}"));
        println!("{sum:?}");
        assert!(sum.errors.is_empty(), "{:?}", sum.errors);
        let hits = crate::knowledge::search(&dir, &["Jupiter".into(), "seventh".into(), "house".into()], 3);
        assert!(!hits.is_empty());
        for h in hits {
            println!("hit: {}", h.file);
        }
    }
}
