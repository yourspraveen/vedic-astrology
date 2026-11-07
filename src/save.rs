//! The report save-file format: a human-readable Markdown body plus a trailing HTML-comment block
//! carrying the birth form, the as-of date and the per-section AI prose, so a report can be rebuilt
//! later from the file alone. Pure string work — no chart maths, no web layer — so it unit-tests on
//! its own. Charts are deliberately dropped from the Markdown (text-faithful): they are recomputed
//! when the file is reopened in the app.

use std::collections::{BTreeMap, HashMap};

const MARKER: &str = "vedic-astrology-save";
const MAX_BYTES: usize = 512 * 1024;

/// Everything needed to rebuild a report from a saved file.
pub struct SavedReport {
    pub form: HashMap<String, String>,
    pub as_of_jd: f64,
    pub ai: HashMap<String, String>,
}

/// Why a file could not be read back.
#[derive(Debug)]
pub enum SaveError {
    NoBlock,
    BadJson,
    Unsupported(u32),
    TooLarge,
}

/// The full `.md` file: the readable report body followed by the embedded reload block.
pub fn to_markdown_file(report_html: &str, form: &HashMap<String, String>, as_of_jd: f64, ai: &BTreeMap<String, String>) -> String {
    let json = serde_json::json!({ "form": form, "as_of_jd": as_of_jd, "ai": ai }).to_string();
    format!("{}\n\n<!-- {MARKER} v1\n{json}\n-->\n", html_to_markdown(report_html))
}

fn decode_entities(s: &str) -> String {
    s.replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&nbsp;", " ")
        .replace("&amp;", "&")
}

/// A text-faithful Markdown rendering of the report HTML. The chart SVGs, styles and scripts are
/// dropped entirely; headings, lists, table rows and paragraphs become their Markdown equivalents.
pub fn html_to_markdown(html: &str) -> String {
    let lower = html.to_ascii_lowercase();
    let mut out = String::with_capacity(html.len());
    let mut i = 0;
    while i < html.len() {
        if html.as_bytes()[i] == b'<' {
            let end = html[i..].find('>').map(|e| i + e + 1).unwrap_or(html.len());
            let tag = &lower[i..end];
            // Drop these elements wholesale, including their contents.
            let skip_to = |name: &str| lower[end..].find(name).map(|p| end + p + name.len()).unwrap_or(html.len());
            if tag.starts_with("<svg") {
                i = skip_to("</svg>");
            } else if tag.starts_with("<style") {
                i = skip_to("</style>");
            } else if tag.starts_with("<script") {
                i = skip_to("</script>");
            } else {
                if tag.starts_with("<h1") {
                    out.push_str("\n\n# ");
                } else if tag.starts_with("<h2") {
                    out.push_str("\n\n## ");
                } else if tag.starts_with("<h3") {
                    out.push_str("\n\n### ");
                } else if tag.starts_with("<li") {
                    out.push_str("\n- ");
                } else if tag.starts_with("<br") || tag.starts_with("<tr") {
                    out.push('\n');
                } else if tag.starts_with("<td") || tag.starts_with("<th") {
                    out.push_str("| ");
                } else if tag.starts_with("</p")
                    || tag.starts_with("</h1")
                    || tag.starts_with("</h2")
                    || tag.starts_with("</h3")
                    || tag.starts_with("</div")
                    || tag.starts_with("</li")
                    || tag.starts_with("</tr")
                    || tag.starts_with("</table")
                    || tag.starts_with("</ul")
                    || tag.starts_with("</ol")
                {
                    out.push('\n');
                }
                i = end;
            }
        } else {
            let next = html[i..].find('<').map(|n| i + n).unwrap_or(html.len());
            out.push_str(&decode_entities(&html[i..next]));
            i = next;
        }
    }
    // Tidy: strip a trailing AI marker from headings, and collapse blank-line runs.
    let mut md: String = out
        .lines()
        .map(|l| if l.starts_with('#') { l.trim_end().trim_end_matches('*').trim_end() } else { l.trim_end() })
        .collect::<Vec<_>>()
        .join("\n");
    while md.contains("\n\n\n") {
        md = md.replace("\n\n\n", "\n\n");
    }
    md.trim().to_string()
}

/// Recover the reload block from a saved file.
pub fn parse(file: &str) -> Result<SavedReport, SaveError> {
    if file.len() > MAX_BYTES {
        return Err(SaveError::TooLarge);
    }
    // Anchor on the LAST comment opener `<!-- {MARKER}`: the block is at the end of the file, and
    // the readable body (and the JSON prose) may legitimately contain the bare marker literal.
    let open_tag = format!("<!-- {MARKER}");
    let idx = file.rfind(&open_tag).ok_or(SaveError::NoBlock)?;
    let after = file[idx + open_tag.len()..].trim_start();
    let after = after.strip_prefix('v').ok_or(SaveError::NoBlock)?;
    let digits: String = after.chars().take_while(|c| c.is_ascii_digit()).collect();
    let version: u32 = digits.parse().map_err(|_| SaveError::NoBlock)?;
    if version != 1 {
        return Err(SaveError::Unsupported(version));
    }
    let nl = after.find('\n').ok_or(SaveError::BadJson)?;
    // The closing `-->` is the LAST one: AI prose inside the JSON may contain the literal `-->`.
    let json_end = after.rfind("-->").ok_or(SaveError::BadJson)?;
    if json_end < nl {
        return Err(SaveError::BadJson);
    }
    let value: serde_json::Value = serde_json::from_str(after[nl + 1..json_end].trim()).map_err(|_| SaveError::BadJson)?;
    let strings = |key: &str| -> HashMap<String, String> {
        value.get(key)
            .and_then(|v| v.as_object())
            .map(|o| o.iter().filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_string()))).collect())
            .unwrap_or_default()
    };
    Ok(SavedReport {
        form: strings("form"),
        as_of_jd: value.get("as_of_jd").and_then(|v| v.as_f64()).unwrap_or(0.0),
        ai: strings("ai"),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn embed_then_parse_round_trips() {
        let mut form = HashMap::new();
        for (k, v) in [("name", "Soverna"), ("date", "2012-12-04"), ("lat", "40"), ("lon", "-89.25"), ("tz", "-6"), ("lang", "en")] {
            form.insert(k.to_string(), v.to_string());
        }
        let mut ai = BTreeMap::new();
        ai.insert("transit".to_string(), "Transit prose.".to_string());
        let file = to_markdown_file("<h1>Horoscope</h1>", &form, 2451545.0, &ai);
        let got = parse(&file).expect("parses");
        assert_eq!(got.form.get("name").unwrap(), "Soverna");
        assert_eq!(got.as_of_jd, 2451545.0);
        assert_eq!(got.ai.get("transit").unwrap(), "Transit prose.");
    }

    #[test]
    fn markdown_strips_charts_keeps_text_and_decodes() {
        let md = html_to_markdown(r#"<h2>Charts</h2><svg><path d="M0 0"/></svg><p>Jupiter &amp; Saturn</p>"#);
        assert!(md.contains("## Charts"));
        assert!(md.contains("Jupiter & Saturn"));
        assert!(!md.contains("<svg") && !md.contains("path d"));
    }

    #[test]
    fn parse_tolerates_arrow_and_marker_in_prose() {
        // AI prose can contain "-->" or the marker literal; the block is anchored from the end, so a
        // validly-saved file still reopens (no silent round-trip failure).
        let mut ai = BTreeMap::new();
        ai.insert("transit".to_string(), "Caution --> change ahead (vedic-astrology-save).".to_string());
        let mut form = HashMap::new();
        form.insert("name".to_string(), "X".to_string());
        let file = to_markdown_file("<h1>x</h1>", &form, 2451545.0, &ai);
        let got = parse(&file).expect("parses despite --> and marker in prose");
        assert_eq!(got.ai.get("transit").unwrap(), "Caution --> change ahead (vedic-astrology-save).");
    }

    #[test]
    fn parse_rejects_missing_wrong_version_and_oversize() {
        assert!(matches!(parse("# just markdown, no block"), Err(SaveError::NoBlock)));
        assert!(matches!(parse("<!-- vedic-astrology-save v2\n{}\n-->"), Err(SaveError::Unsupported(2))));
        assert!(matches!(parse(&"x".repeat(600 * 1024)), Err(SaveError::TooLarge)));
    }
}
