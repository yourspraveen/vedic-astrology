//! Self-hosted web version (`vedic_astrology --serve 0.0.0.0:8080`): a small HTTP server with the
//! birth-details form, place lookup, the full report, the Tamil / Vedic calendar, a Settings page
//! for the optional AI writer, and chart questions. It is what the Docker image runs. There is no
//! login: run it on a private network or behind a reverse proxy that adds one.
//!
//! Two looks share one layout: a light "Temple Dawn" palette and a dark "Twilight" palette, chosen
//! by the viewer (a top-bar toggle, remembered in the browser) and defaulting to the system theme.
//!
//! AI secrets stay on the server. The base URL and API key come from environment variables
//! (VA_AI_BASE_URL, VA_AI_API_KEY, VA_AI_ORG, VA_AI_USER) and are never sent to browsers. The
//! Settings page lets the viewer pick the model (pulled from the service), set the temperature and
//! passages, edit the AI instructions and choose the default calculation engine; those are kept on
//! the server (VA_AI_MODEL seeds the model). The knowledge folder is set by VA_KNOWLEDGE or
//! `--knowledge <dir>`.

use crate::style::Style;
use crate::i18n::{self, Lang};
use crate::vedic::*;
use crate::{calendar, festivals, geo, knowledge, llm, report, save};
use std::collections::HashMap;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::PathBuf;
use std::sync::Mutex;

/// The interface language is a process-wide setting, so pages are built one at a time.
static RENDER: Mutex<()> = Mutex::new(());

/// Server-side choices made on the Settings page. The AI base URL and key never live here; they
/// stay in the environment. Everything here is safe to change at run time.
struct Overrides {
    /// The report-body (fast) model.
    model: Option<String>,
    /// The Ask-box (reasoning) model.
    ask_model: Option<String>,
    temperature: Option<f32>,
    passages: Option<usize>,
    /// true = show civil (calendar) weekday/dates; false/None = astrological vāra (the default).
    weekday_civil: Option<bool>,
    /// "regional" (default), "western" or "both" — how rashi names are shown.
    sign_style: Option<String>,
}
static OVERRIDES: Mutex<Overrides> = Mutex::new(Overrides { model: None, ask_model: None, temperature: None, passages: None, weekday_civil: None, sign_style: None });

/// Whether the report shows civil weekdays instead of the astrological vāra (Settings choice).
fn weekday_civil() -> bool {
    OVERRIDES.lock().unwrap_or_else(|e| e.into_inner()).weekday_civil.unwrap_or(false)
}

/// The rashi-name style chosen in Settings.
fn sign_style() -> i18n::SignStyle {
    match OVERRIDES.lock().unwrap_or_else(|e| e.into_inner()).sign_style.as_deref() {
        Some("western") => i18n::SignStyle::Western,
        Some("both") => i18n::SignStyle::Both,
        _ => i18n::SignStyle::Regional,
    }
}

pub struct Response {
    pub status: u16,
    pub content_type: &'static str,
    pub body: Vec<u8>,
    /// Extra response headers (e.g. Content-Disposition for a download); emitted after the fixed ones.
    pub headers: Vec<(&'static str, String)>,
}

fn html(body: String) -> Response {
    // Every full page carries the AGPL §13 source-code offer (see `footer`). The page's language is
    // read from its own `<html lang="…">` so the footer matches it without touching global state.
    let lc = body
        .split_once("<html lang=\"")
        .and_then(|(_, r)| r.split_once('"'))
        .map(|(c, _)| c.to_string())
        .unwrap_or_else(|| "en".into());
    // Report and calendar pages close with `</body>`; the index and settings templates leave the
    // body open, so fall back to appending the footer at the end.
    let foot = footer(&lc);
    let body = if body.contains("</body>") {
        body.replacen("</body>", &format!("{foot}</body>"), 1)
    } else {
        format!("{body}{foot}")
    };
    Response { status: 200, content_type: "text/html; charset=utf-8", body: body.into_bytes(), headers: Vec::new() }
}
fn json(v: serde_json::Value) -> Response {
    Response { status: 200, content_type: "application/json; charset=utf-8", body: v.to_string().into_bytes(), headers: Vec::new() }
}
fn icon() -> Response {
    Response {
        status: 200,
        content_type: "image/x-icon",
        body: FAVICON.to_vec(),
        headers: vec![("Cache-Control", "public, max-age=604800".into())],
    }
}
fn error(status: u16, msg: &str) -> Response {
    Response { status, content_type: "text/plain; charset=utf-8", body: msg.as_bytes().to_vec(), headers: Vec::new() }
}

/// A file download: sets `Content-Disposition: attachment` with a sanitised filename.
fn download(body: String, filename: &str, content_type: &'static str) -> Response {
    let safe: String = filename.chars().filter(|&c| c != '"' && c != '\r' && c != '\n').collect();
    Response {
        status: 200,
        content_type,
        body: body.into_bytes(),
        headers: vec![("Content-Disposition", format!("attachment; filename=\"{safe}\""))],
    }
}

fn env(k: &str) -> Option<String> {
    std::env::var(k).ok().filter(|v| !v.trim().is_empty())
}

/// Is an AI service configured on this server (base URL present)?
fn ai_base_url() -> Option<String> {
    env("VA_AI_BASE_URL")
}

/// The report-body model (fast tier): a Settings override, else VA_AI_MODEL. The body fans out into
/// ~20 parallel calls, so a fast, non-reasoning model keeps every section complete and the run quick.
fn body_model() -> String {
    OVERRIDES.lock().unwrap_or_else(|e| e.into_inner()).model.clone().or_else(|| env("VA_AI_MODEL")).unwrap_or_default()
}

/// The Ask-box model (reasoning tier): a Settings override, else VA_AI_ASK_MODEL, else whatever the
/// body uses. One deliberate question is worth a slower, deeper model.
fn ask_model() -> String {
    OVERRIDES.lock().unwrap_or_else(|e| e.into_inner()).ask_model.clone()
        .or_else(|| env("VA_AI_ASK_MODEL"))
        .filter(|m| !m.trim().is_empty())
        .unwrap_or_else(body_model)
}

/// A service config for the given model: base URL and key from the environment. None when no base URL.
fn ai_probe_with(model: String) -> Option<llm::LlmConfig> {
    let base_url = ai_base_url()?;
    Some(llm::LlmConfig {
        enabled: true,
        base_url,
        model,
        api_key: env("VA_AI_API_KEY").unwrap_or_default(),
        user: env("VA_AI_USER").unwrap_or_default(),
        organization: env("VA_AI_ORG").unwrap_or_default(),
        knowledge_dir: knowledge::dir().map(|d| d.display().to_string()).unwrap_or_default(),
        // Reasoning models burn part of this on hidden thinking; VA_AI_MAX_TOKENS tunes it without a rebuild.
        max_tokens: env("VA_AI_MAX_TOKENS").and_then(|v| v.trim().parse().ok()).unwrap_or(2000),
        ..Default::default()
    })
}

/// A config for touching the service (test, model list), using the body model. None when no base URL.
fn ai_probe() -> Option<llm::LlmConfig> {
    ai_probe_with(body_model())
}

/// Apply the shared Settings overrides (temperature, passages) and drop a config with no model.
fn finish_config(mut cfg: llm::LlmConfig) -> Option<llm::LlmConfig> {
    if cfg.model.trim().is_empty() {
        return None;
    }
    let ov = OVERRIDES.lock().unwrap_or_else(|e| e.into_inner());
    if let Some(t) = ov.temperature {
        cfg.temperature = t;
    }
    if let Some(p) = ov.passages {
        cfg.passages = p;
    }
    Some(cfg)
}

/// The full AI config for writing a report (fast tier): needs both a base URL and a model.
pub fn ai_config() -> Option<llm::LlmConfig> {
    finish_config(ai_probe_with(body_model())?)
}

/// The AI config for the Ask box (reasoning tier).
pub fn ask_config() -> Option<llm::LlmConfig> {
    finish_config(ai_probe_with(ask_model())?)
}

/// Run the server until the process is stopped.
pub fn serve(addr: &str, knowledge_dir: PathBuf) -> std::io::Result<()> {
    let _ = knowledge::seed(&knowledge_dir);
    knowledge::set_dir(Some(knowledge_dir.clone()));
    let listener = TcpListener::bind(addr)?;
    eprintln!("Vedic Astrology web: http://{addr}/  (knowledge folder {}; AI writer {})", knowledge_dir.display(), if ai_config().is_some() { "on" } else { "off" });
    for stream in listener.incoming().flatten() {
        std::thread::spawn(move || {
            let _ = connection(stream);
        });
    }
    Ok(())
}

fn connection(mut stream: TcpStream) -> std::io::Result<()> {
    stream.set_read_timeout(Some(std::time::Duration::from_secs(30)))?;
    let mut reader = BufReader::new(stream.try_clone()?);
    let mut line = String::new();
    reader.read_line(&mut line)?;
    let mut parts = line.split_whitespace();
    let (method, target) = (parts.next().unwrap_or("").to_string(), parts.next().unwrap_or("/").to_string());
    let mut length = 0usize;
    loop {
        let mut h = String::new();
        if reader.read_line(&mut h)? == 0 || h == "\r\n" || h == "\n" {
            break;
        }
        if let Some((k, v)) = h.split_once(':')
            && k.trim().eq_ignore_ascii_case("content-length") {
                length = v.trim().parse().unwrap_or(0);
            }
    }
    let mut body = vec![0u8; length.min(1 << 20)];
    reader.read_exact(&mut body)?;
    let (path, query) = target.split_once('?').unwrap_or((&target, ""));
    let resp = handle(&method, path, query, &String::from_utf8_lossy(&body));
    let reason = match resp.status {
        200 => "OK",
        400 => "Bad Request",
        404 => "Not Found",
        _ => "Error",
    };
    write!(stream, "HTTP/1.1 {} {reason}\r\nContent-Type: {}\r\nContent-Length: {}\r\nConnection: close\r\nX-Content-Type-Options: nosniff\r\n", resp.status, resp.content_type, resp.body.len())?;
    for (name, value) in &resp.headers {
        write!(stream, "{name}: {value}\r\n")?;
    }
    write!(stream, "\r\n")?;
    stream.write_all(&resp.body)?;
    stream.flush()
}

/// Route one request.
pub fn handle(method: &str, path: &str, query: &str, body: &str) -> Response {
    let q = form(query);
    match (method, path) {
        ("GET", "/") => html(index_page(&q)),
        ("GET", "/settings") => html(settings_page(&q)),
        ("GET", "/health") => error(200, "ok"),
        ("GET", "/favicon.ico") => icon(),
        ("GET", "/api/geocode") => geocode(&q),
        ("GET", "/api/offset") => offset(&q),
        ("GET", "/api/reverse") => reverse(&q),
        // The report is produced by POST; a stray GET (e.g. an old link) falls back to the form.
        ("GET", "/report") => html(index_page(&q)),
        ("GET", "/api/ai/models") => ai_models(),
        ("POST", "/api/ai/test") => ai_test(),
        ("POST", "/api/settings") => save_settings(&form(body)),
        ("POST", "/report") => report_page(&form(body)),
        ("POST", "/report.md") => report_markdown(&form(body)),
        ("POST", "/report/open") => report_open(&form(body)),
        ("POST", "/report/rewrite-section") => rewrite_section(&form(body)),
        ("GET", "/calendar") => calendar_page(&q),
        ("POST", "/api/ask") => ask(&form(body)),
        _ => error(404, "not found"),
    }
}

// ---------------------------------------------------------------- form handling

/// Decode application/x-www-form-urlencoded text.
pub fn form(s: &str) -> HashMap<String, String> {
    s.split('&').filter(|p| !p.is_empty()).map(|p| {
        let (k, v) = p.split_once('=').unwrap_or((p, ""));
        (decode(k), decode(v))
    }).collect()
}

fn decode(s: &str) -> String {
    let b = s.as_bytes();
    let mut out = Vec::with_capacity(b.len());
    let mut i = 0;
    while i < b.len() {
        match b[i] {
            b'+' => out.push(b' '),
            b'%' if i + 2 < b.len() => {
                match u8::from_str_radix(std::str::from_utf8(&b[i + 1..i + 3]).unwrap_or(""), 16) {
                    Ok(v) => {
                        out.push(v);
                        i += 2;
                    }
                    Err(_) => out.push(b'%'),
                }
            }
            c => out.push(c),
        }
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

fn esc(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;").replace('"', "&quot;")
}

fn lang_of(q: &HashMap<String, String>) -> Lang {
    match q.get("lang").map(String::as_str) {
        Some("ta") => Lang::Ta,
        Some("hi") => Lang::Hi,
        _ => Lang::En,
    }
}

fn num<T: std::str::FromStr>(q: &HashMap<String, String>, k: &str) -> Option<T> {
    q.get(k).and_then(|v| v.trim().parse().ok())
}

/// Birth details and calculation choices from the form.
fn birth_from(q: &HashMap<String, String>) -> Result<(BirthData, Calc, Style), String> {
    let date = q.get("date").ok_or("date is missing")?;
    let mut d = date.split('-').map(|x| x.parse::<i64>());
    let (year, month, day) = match (d.next(), d.next(), d.next()) {
        (Some(Ok(y)), Some(Ok(m)), Some(Ok(dd))) => (y as i32, m as u32, dd as u32),
        _ => return Err("date must be YYYY-MM-DD".into()),
    };
    let mut t = q.get("time").map(String::as_str).unwrap_or("12:00").split(':').map(|x| x.parse::<u32>().unwrap_or(0));
    let (hour, minute, second) = (t.next().unwrap_or(12), t.next().unwrap_or(0), t.next().unwrap_or(0));
    let mut b = BirthData {
        name: q.get("name").cloned().filter(|n| !n.trim().is_empty()).unwrap_or_else(|| "—".into()),
        gender_male: q.get("gender").map(String::as_str) != Some("f"),
        year,
        month,
        day,
        hour: hour.min(23),
        minute: minute.min(59),
        second: second.min(59),
        tz: num(q, "tz").ok_or("time zone is missing")?,
        place: q.get("place").cloned().unwrap_or_default(),
        lat: num(q, "lat").ok_or("latitude is missing")?,
        lon: num(q, "lon").ok_or("longitude is missing")?,
        tz_name: q.get("tz_name").cloned().unwrap_or_default(),
    };
    if !(1..=12).contains(&b.month) || !(1..=31).contains(&b.day) || !(-90.0..=90.0).contains(&b.lat) || !(-180.0..=180.0).contains(&b.lon) {
        return Err("date or place is out of range".into());
    }
    if let Some(off) = geo::utc_offset(&b.tz_name, b.year, b.month, b.day, b.hour, b.minute) {
        b.tz = off;
    }
    let calc = Calc {
        node: if q.get("node").map(String::as_str) == Some("true") { NodeKind::True } else { NodeKind::Mean },
        ayanamsa: match q.get("ayanamsa").map(String::as_str) {
            Some("raman") => Ayanamsa::Raman,
            Some("kp") => Ayanamsa::Krishnamurti,
            _ => Ayanamsa::Lahiri,
        },
    };
    let style = match q.get("style").map(String::as_str) {
        Some("north") => Style::North,
        Some("east") => Style::East,
        _ => Style::South,
    };
    Ok((b, calc, style))
}

fn now_jd() -> f64 {
    let secs = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs_f64()).unwrap_or(0.0);
    2440587.5 + secs / 86400.0
}

// ---------------------------------------------------------------- shared chrome

/// The two palettes, the fonts, the theme toggle and the components every page shares.
const CSS: &str = include_str!("web_assets/style.css");
/// The star mark as a multi-size .ico, served at the well-known /favicon.ico path: browsers and
/// bookmarking tools ask for it there whether or not a page links to it.
const FAVICON: &[u8] = include_bytes!("web_assets/favicon.ico");
/// Theme bootstrap (set before paint, no flash) and the toggle + glyph-tooltip helpers.
const SCRIPT: &str = include_str!("web_assets/app.js");

/// An auspicious Lord Ganesha line emblem and the traditional invocation, shown at the top of a
/// report (Ganesha is invoked at the start of any new undertaking, including a horoscope).
const GANESHA: &str = r##"<div class="ganesha"><svg viewBox="0 0 100 100" role="img" aria-label="Lord Ganesha" fill="none" stroke="currentColor" stroke-width="2.4" stroke-linecap="round" stroke-linejoin="round"><circle cx="50" cy="50" r="46" stroke-width="1.3" opacity=".4"/><path d="M34 40 C 18 38 14 50 18 60 C 22 68 32 64 36 60"/><path d="M66 40 C 82 38 86 50 82 60 C 78 68 68 64 64 60"/><ellipse cx="50" cy="52" rx="19" ry="17"/><path d="M38 36 Q50 16 62 36"/><circle cx="50" cy="19" r="2.6" fill="currentColor" stroke="none"/><path d="M50 22 L50 30"/><path d="M41 31 L39 26"/><path d="M59 31 L61 26"/><path d="M50 38 L50 43"/><circle cx="42" cy="50" r="1.9" fill="currentColor" stroke="none"/><circle cx="58" cy="50" r="1.9" fill="currentColor" stroke="none"/><path d="M50 45 C 49 56 46 64 43 70 C 40 75 44 80 49 78 C 53 76.5 51 72 48 71.5"/><path d="M44 62 L41 68"/><path d="M56 62 L59 68"/><path d="M34 85 Q50 79 66 85"/><path d="M41 83 Q50 89 59 83"/></svg><p class="invocation">॥ श्री गणेशाय नमः ॥</p></div>"##;

/// `<head>` + opening `<body>`, with the theme set before first paint.
fn head(title: &str, lang_code: &str) -> String {
    format!(
        "<!DOCTYPE html><html lang=\"{lang_code}\"><head><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width, initial-scale=1\"><title>{}</title>\
         <link rel=\"icon\" href=\"/favicon.ico\" sizes=\"16x16 32x32 48x48 64x64\">\
         <link rel=\"icon\" type=\"image/svg+xml\" href=\"data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 32 32'%3E%3Cpath fill='%23e6a23c' d='M16 3l3.4 8.6 9.2.5-7.1 5.9 2.3 8.9L16 25.9 8.2 30.9l2.3-8.9-7.1-5.9 9.2-.5z'/%3E%3C/svg%3E\">\
         <link rel=\"preconnect\" href=\"https://fonts.gstatic.com\" crossorigin><link rel=\"stylesheet\" href=\"https://fonts.googleapis.com/css2?family=Marcellus&family=Noto+Sans:wght@400;500;600;700&display=swap\">\
         <style>{CSS}</style><script>{}</script></head><body>",
        esc(title),
        THEME_INIT
    )
}

/// A generic, overridable default for the source-code link. Operators point `VA_SOURCE_URL` at
/// their own published source — which is what the AGPL actually requires, since §13 obliges each
/// network operator to offer *their* Corresponding Source, not some upstream copy. No personal host
/// is baked in here.
const SOURCE_URL_DEFAULT: &str = "https://codeberg.org/vedic-astrology/vedic-astrology";

/// Where the footer's "Source code" link points. `VA_SOURCE_URL` overrides the default above.
fn source_url() -> String {
    env("VA_SOURCE_URL").unwrap_or_else(|| SOURCE_URL_DEFAULT.into())
}

/// The site footer, served on every full HTML page. It satisfies the AGPL's network clause (§13):
/// a user interacting with the app over the network is offered its Corresponding Source. It also
/// credits the Swiss Ephemeris, whose AGPL use sets the project's licence. `lc` is the page's
/// language code so the labels match; the three strings are inlined to avoid depending on the
/// global current-language state after a page has finished rendering.
fn footer(lc: &str) -> String {
    let (agpl, src, pos, places) = match lc {
        "ta" => ("கட்டற்ற மென்பொருள் (AGPL-3.0)", "மூலக் குறியீடு", "கோள் நிலைகள்: Swiss Ephemeris", "இடத் தரவு: Open-Meteo, © OpenStreetMap பங்களிப்பாளர்கள்"),
        "hi" => ("मुक्त सॉफ़्टवेयर (AGPL-3.0)", "सोर्स कोड", "ग्रह स्थिति: Swiss Ephemeris", "स्थान डेटा: Open-Meteo, © OpenStreetMap योगदानकर्ता"),
        _ => ("Free software (AGPL-3.0)", "Source code", "Positions by the Swiss Ephemeris", "Place data: Open-Meteo, © OpenStreetMap contributors"),
    };
    format!(
        "<footer class=\"site-foot\"><span>✨ Vedic&nbsp;Astrology · {agpl} · \
         <a href=\"{url}\" rel=\"license noopener\" target=\"_blank\">{src}</a></span>\
         <span class=\"foot-sub\">{pos} · {places}</span></footer>",
        url = esc(&source_url())
    )
}

const THEME_INIT: &str = "(function(){try{var d=document.documentElement;var t=localStorage.getItem('fj-theme');if(t==='dark'||t==='light')d.setAttribute('data-theme',t);var z=parseFloat(localStorage.getItem('fj-fontscale'));if(z>=0.8&&z<=1.6)d.style.setProperty('--zoom',z);}catch(e){}})();";

/// Replace `{{English}}` markers in a template with the current language's text.
fn i18n_pass(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut rest = s;
    while let Some(i) = rest.find("{{") {
        out.push_str(&rest[..i]);
        let after = &rest[i + 2..];
        match after.find("}}") {
            Some(j) => {
                out.push_str(&i18n::tr_dyn(&after[..j]));
                rest = &after[j + 2..];
            }
            None => {
                out.push_str("{{");
                rest = after;
            }
        }
    }
    out.push_str(rest);
    out
}

/// The sticky top bar: brand, page navigation, a language switcher and the light/dark toggle.
/// `lang` is the current language code; nav links and the switcher carry it through the site.
fn topbar(active: &str, lang: &str) -> String {
    let item = |href: &str, key: &str, label: &str| {
        let cur = if key == active { " aria-current=\"page\"" } else { "" };
        format!("<a href=\"{href}\"{cur}>{}</a>", esc(&i18n::tr_dyn(label)))
    };
    let options: String = i18n::LANGS.iter().enumerate().map(|(i, (_, name))| {
        let code = ["en", "ta", "hi"][i];
        format!("<option value=\"{code}\"{}>{}</option>", if code == lang { " selected" } else { "" }, esc(name))
    }).collect();
    format!(
        "<header class=\"bar\"><a class=\"brand\" href=\"/?lang={lang}\">✨ Vedic&nbsp;Astrology</a>\
         <nav class=\"nav\">{}{}{}</nav>\
         <select class=\"langsel\" aria-label=\"{}\" onchange=\"switchLang(this.value)\">{options}</select>\
         <button class=\"icon\" type=\"button\" onclick=\"bumpFont(-1)\" title=\"{sm}\" aria-label=\"{sm}\">A−</button>\
         <button class=\"icon\" type=\"button\" onclick=\"bumpFont(1)\" title=\"{lg}\" aria-label=\"{lg}\">A+</button>\
         <button id=\"themebtn\" class=\"icon\" type=\"button\" onclick=\"toggleTheme()\" title=\"{}\" aria-label=\"{}\">☾</button></header>",
        item(&format!("/?lang={lang}"), "report", "Report"),
        item(&format!("/calendar?lang={lang}"), "calendar", "Calendar"),
        item(&format!("/settings?lang={lang}"), "settings", "Settings"),
        esc(&i18n::tr_dyn("Language")),
        esc(&i18n::tr_dyn("Light / dark")),
        esc(&i18n::tr_dyn("Light / dark")),
        sm = esc(&i18n::tr_dyn("Smaller text")),
        lg = esc(&i18n::tr_dyn("Larger text")),
    )
}

// ---------------------------------------------------------------- pages

fn report_page(q: &HashMap<String, String>) -> Response {
    let (b, calc, style) = match birth_from(q) {
        Ok(v) => v,
        Err(e) => return error(400, &e),
    };
    let chart = Chart::compute(b.jd_ut(), b.lat, b.lon, b.tz, calc);
    let pancha = panchanga(&chart);
    let dasha = vimshottari(chart.moon(), chart.jd_ut, 3);
    let as_of_jd = now_jd();
    let inp = report::Input { birth: &b, chart: &chart, pancha: &pancha, dasha: &dasha, calc, style, now: as_of_jd, weekday_civil: weekday_civil(), stale_keys: &[] };
    let lang = lang_of(q);
    // Render the rule-based report under the process-wide language lock (brief).
    let (page, tasks) = {
        let _g = RENDER.lock().unwrap_or_else(|e| e.into_inner());
        i18n::set_lang(lang);
        i18n::set_sign_style(sign_style());
        report::build_with(&inp, &HashMap::new())
    };
    // Expand with the AI WITHOUT holding the lock: it does network calls, not rendering, so the rest
    // of the site stays responsive while a (possibly slow) report is being enriched.
    let results = if q.get("ai").is_some() {
        match ai_config() {
            Some(cfg) => {
                let job = Mutex::new(llm::Job { total: tasks.len(), ..Default::default() });
                llm::expand(&cfg, &tasks, ["English", "Tamil", "Hindi"][lang as usize], &job);
                std::mem::take(&mut job.lock().unwrap_or_else(|e| e.into_inner()).results)
            }
            None => HashMap::new(),
        }
    } else {
        HashMap::new()
    };
    // Final render (with AI texts if any) and decorate, under the lock again.
    let out = {
        let _g = RENDER.lock().unwrap_or_else(|e| e.into_inner());
        i18n::set_lang(lang);
        i18n::set_sign_style(sign_style());
        let p = if results.is_empty() { page } else { report::build_with(&inp, &results).0 };
        let ai_json = {
            let m: std::collections::BTreeMap<&str, &str> = results.iter().map(|(k, (_, t))| (k.as_str(), t.as_str())).collect();
            serde_json::to_string(&m).unwrap_or_else(|_| "{}".into())
        };
        decorate_report(p, q, &b, lang, as_of_jd, &ai_json, "")
    };
    html(out)
}

/// The birth-form fields stored in a saved report, so reopening reuses the exact same `birth_from`
/// parse (and its range validation). Everything needed to rebuild the chart, plus the language.
const SAVE_FORM_FIELDS: [&str; 13] = ["name", "gender", "date", "time", "place", "lat", "lon", "tz", "tz_name", "ayanamsa", "style", "node", "lang"];

/// A filesystem-safe slug for a name, for the download filename.
fn slug(name: &str) -> String {
    let s: String = name.trim().chars().map(|c| if c.is_alphanumeric() { c.to_ascii_lowercase() } else { '-' }).collect();
    let s = s.trim_matches('-').to_string();
    if s.is_empty() { "report".into() } else { s }
}

/// Build the current report and return it as a downloadable `.md` save file. The AI prose is taken
/// from the posted `ai` field (the page's current texts, including any rewrites) — this never calls
/// the AI service, so a download costs nothing.
fn report_markdown(q: &HashMap<String, String>) -> Response {
    let (b, calc, style) = match birth_from(q) {
        Ok(v) => v,
        Err(e) => return error(400, &e),
    };
    let chart = Chart::compute(b.jd_ut(), b.lat, b.lon, b.tz, calc);
    let pancha = panchanga(&chart);
    let dasha = vimshottari(chart.moon(), chart.jd_ut, 3);
    let as_of_jd = q.get("as_of_jd").and_then(|s| s.parse::<f64>().ok()).unwrap_or_else(now_jd);
    let inp = report::Input { birth: &b, chart: &chart, pancha: &pancha, dasha: &dasha, calc, style, now: as_of_jd, weekday_civil: weekday_civil(), stale_keys: &[] };
    let lang = lang_of(q);
    let ai_text: std::collections::BTreeMap<String, String> = q.get("ai").and_then(|s| serde_json::from_str(s).ok()).unwrap_or_default();
    let ai_map: HashMap<String, (String, String)> = ai_text.iter().map(|(k, v)| (k.clone(), (String::new(), v.clone()))).collect();
    let html_body = {
        let _g = RENDER.lock().unwrap_or_else(|e| e.into_inner());
        i18n::set_lang(lang);
        i18n::set_sign_style(sign_style());
        report::build_with(&inp, &ai_map).0
    };
    let form: HashMap<String, String> = SAVE_FORM_FIELDS.iter().filter_map(|k| q.get(*k).map(|v| (k.to_string(), v.clone()))).collect();
    let md = save::to_markdown_file(&html_body, &form, as_of_jd, &ai_text);
    let (y, m, d, _) = crate::astro::calendar(as_of_jd + b.tz / 24.0);
    download(md, &format!("{}-{y}-{m:02}-{d:02}.md", slug(&b.name)), "text/markdown; charset=utf-8")
}

/// Reopen a saved `.md` report. `file` is the uploaded file's text; `refresh=1` re-anchors the
/// time-based sections to today (free). The chart is rebuilt from the embedded birth form through the
/// same `birth_from` parse as a fresh report, so a hand-edited, out-of-range file is rejected here.
fn report_open(q: &HashMap<String, String>) -> Response {
    let file = match q.get("file") {
        Some(f) if !f.is_empty() => f,
        _ => return error(400, "no file provided"),
    };
    let saved = match save::parse(file) {
        Ok(s) => s,
        Err(_) => return error(400, "this does not look like a saved Vedic Astrology report"),
    };
    let (b, calc, style) = match birth_from(&saved.form) {
        Ok(v) => v,
        Err(e) => return error(400, &e),
    };
    let lang = lang_of(&saved.form);
    let refresh = q.get("refresh").map(String::as_str) == Some("1");
    let now = if refresh { now_jd() } else { saved.as_of_jd };
    let chart = Chart::compute(b.jd_ut(), b.lat, b.lon, b.tz, calc);
    let pancha = panchanga(&chart);
    let dasha = vimshottari(chart.moon(), chart.jd_ut, 3);
    // On refresh, the saved prose for time-based sections is stale; on plain view, nothing is stale.
    let stale: Vec<String> = if refresh {
        saved.ai.keys().filter(|k| report::is_time_based_key(k)).cloned().collect()
    } else {
        Vec::new()
    };
    let inp = report::Input { birth: &b, chart: &chart, pancha: &pancha, dasha: &dasha, calc, style, now, weekday_civil: weekday_civil(), stale_keys: &stale };
    let ai_map: HashMap<String, (String, String)> = saved.ai.iter().map(|(k, v)| (k.clone(), (String::new(), v.clone()))).collect();
    let out = {
        let _g = RENDER.lock().unwrap_or_else(|e| e.into_inner());
        i18n::set_lang(lang);
        i18n::set_sign_style(sign_style());
        let p = report::build_with(&inp, &ai_map).0;
        let ai_json = serde_json::to_string(&saved.ai).unwrap_or_else(|_| "{}".into());
        decorate_report(p, &saved.form, &b, lang, now, &ai_json, "")
    };
    html(out)
}

/// Rewrite one refreshed section with the AI and re-render the whole report page. Stateless: the
/// page posts its current section texts (`ai`) and the keys already rewritten (`fresh`); this adds
/// the one new rewrite and returns the updated page, carrying the new `ai`/`fresh` forward. Costs
/// exactly one section's tokens. Refuses when no AI service is configured.
fn rewrite_section(q: &HashMap<String, String>) -> Response {
    let cfg = match ai_config() {
        Some(c) => c,
        None => return error(400, "the AI writer is not configured on this server"),
    };
    let section = match q.get("section") {
        Some(s) if !s.is_empty() => s.clone(),
        _ => return error(400, "no section given"),
    };
    let (b, calc, style) = match birth_from(q) {
        Ok(v) => v,
        Err(e) => return error(400, &e),
    };
    let lang = lang_of(q);
    let now = now_jd(); // a rewrite only happens on a refreshed report, i.e. anchored to today
    let chart = Chart::compute(b.jd_ut(), b.lat, b.lon, b.tz, calc);
    let pancha = panchanga(&chart);
    let dasha = vimshottari(chart.moon(), chart.jd_ut, 3);
    let mut ai_text: std::collections::BTreeMap<String, String> = q.get("ai").and_then(|s| serde_json::from_str(s).ok()).unwrap_or_default();
    let ai_map: HashMap<String, (String, String)> = ai_text.iter().map(|(k, v)| (k.clone(), (String::new(), v.clone()))).collect();
    // Render once to collect the AI tasks, find this section's, and expand just that one.
    let task = {
        let _g = RENDER.lock().unwrap_or_else(|e| e.into_inner());
        i18n::set_lang(lang);
        i18n::set_sign_style(sign_style());
        let inp = report::Input { birth: &b, chart: &chart, pancha: &pancha, dasha: &dasha, calc, style, now, weekday_civil: weekday_civil(), stale_keys: &[] };
        report::build_with(&inp, &ai_map).1.into_iter().find(|t| t.key == section)
    };
    let task = match task {
        Some(t) => t,
        None => return error(400, "unknown section"),
    };
    let job = Mutex::new(llm::Job { total: 1, ..Default::default() });
    llm::expand(&cfg, &[task], ["English", "Tamil", "Hindi"][lang as usize], &job);
    let results = std::mem::take(&mut job.lock().unwrap_or_else(|e| e.into_inner()).results);
    if let Some((_, text)) = results.get(&section) {
        ai_text.insert(section.clone(), text.clone());
    }
    // This section is no longer stale; others that are time-based and not yet rewritten still are.
    let mut fresh: Vec<String> = q.get("fresh").map(|s| s.split(',').filter(|x| !x.is_empty()).map(str::to_string).collect()).unwrap_or_default();
    if !fresh.contains(&section) {
        fresh.push(section.clone());
    }
    let stale: Vec<String> = ai_text.keys().filter(|k| report::is_time_based_key(k) && !fresh.contains(k)).cloned().collect();
    let ai_map2: HashMap<String, (String, String)> = ai_text.iter().map(|(k, v)| (k.clone(), (String::new(), v.clone()))).collect();
    let out = {
        let _g = RENDER.lock().unwrap_or_else(|e| e.into_inner());
        i18n::set_lang(lang);
        i18n::set_sign_style(sign_style());
        let inp = report::Input { birth: &b, chart: &chart, pancha: &pancha, dasha: &dasha, calc, style, now, weekday_civil: weekday_civil(), stale_keys: &stale };
        let p = report::build_with(&inp, &ai_map2).0;
        let ai_json = serde_json::to_string(&ai_text).unwrap_or_else(|_| "{}".into());
        decorate_report(p, q, &b, lang, now, &ai_json, &fresh.join(","))
    };
    html(out)
}

/// Wrap the report's own HTML in the site chrome: the top bar, a theme-aware override so it matches
/// the dark palette, a "new chart" link, and the Ask-a-question panel (moved here from the home
/// page, since a question is about this chart). The report body is left untouched.
fn decorate_report(page: String, q: &HashMap<String, String>, b: &BirthData, lang: Lang, as_of_jd: f64, ai_json: &str, fresh: &str) -> String {
    let lc = ["en", "ta", "hi"][lang as usize];
    let f = |k: &str| esc(q.get(k).map(String::as_str).unwrap_or(""));
    let ai_on = ai_config().is_some();
    let ai_field = if q.get("ai").is_some() { "<input type=hidden name=ai value=1>" } else { "" };
    // All the chart's inputs, so a form can recompute it (for the question box and for regenerating
    // the report in another language). The `lang` input is added per form.
    let birth_fields = format!(
        "<input type=hidden name=name value=\"{}\"><input type=hidden name=gender value=\"{}\"><input type=hidden name=date value=\"{}\"><input type=hidden name=time value=\"{}\"><input type=hidden name=lat value=\"{}\"><input type=hidden name=lon value=\"{}\"><input type=hidden name=tz value=\"{}\"><input type=hidden name=tz_name value=\"{}\"><input type=hidden name=place value=\"{}\"><input type=hidden name=ayanamsa value=\"{}\"><input type=hidden name=node value=\"{}\"><input type=hidden name=style value=\"{}\">",
        f("name"), f("gender"), f("date"), f("time"), f("lat"), f("lon"), f("tz"), f("tz_name"), f("place"), f("ayanamsa"), f("node"), f("style"),
    );
    let base_fields = format!("{birth_fields}{ai_field}");
    // A download form posting the chart plus the page's current AI texts to /report.md. The `ai`
    // value is the live section texts (JSON); it is distinct from the `ai=1` enable flag above.
    let md_form = format!(
        "<form method=post action=\"/report.md\" style=\"display:inline\">{birth_fields}<input type=hidden name=lang value=\"{lc}\"><input type=hidden name=as_of_jd value=\"{as_of_jd}\"><input type=hidden name=ai value=\"{ai}\"><button type=submit class=\"btn\">⬇ {md}</button></form>",
        ai = esc(ai_json),
        md = esc(&i18n::tr_dyn("Download as Markdown")),
    );
    // Hidden form the per-section "Rewrite with AI" buttons submit (app.js fills in `section`). It
    // carries the page's current AI texts and the already-rewritten keys so the server stays stateless.
    let rewrite_form = format!(
        "<form id=\"rewritef\" method=post action=\"/report/rewrite-section\" hidden>{birth_fields}<input type=hidden name=lang value=\"{lc}\"><input type=hidden name=as_of_jd value=\"{as_of_jd}\"><input type=hidden name=ai value=\"{ai}\"><input type=hidden name=fresh value=\"{fresh}\"><input type=hidden name=section id=\"rewrite_section\" value=\"\"></form>",
        ai = esc(ai_json),
        fresh = esc(fresh),
    );
    // Hidden form that re-posts the chart in whatever language the top-bar switcher picks.
    let regen = format!("<form id=\"regen\" method=\"post\" action=\"/report\" hidden>{base_fields}<input type=hidden name=lang id=\"regen_lang\" value=\"{lc}\"></form>");
    let ask = format!(
        "<section class=\"ask card\"><h2 class=\"ask-h\">🔮 {}</h2>\
         <form id=\"askf\" onsubmit=\"return ask(event)\">{base_fields}<input type=hidden name=lang value=\"{lc}\">\
         <div class=\"ask-row\"><input id=\"question\" name=\"question\" placeholder=\"{ph}\" {dis}>\
         <button class=\"btn\" {dis}>{ask_btn}</button></div></form>\
         <p class=\"note\">{note}</p><div id=\"answer\" class=\"answer\" hidden></div>\
         <div class=\"qahead\" id=\"qahead\" hidden><h3>{qa_title}</h3><button type=\"button\" class=\"btn ghost sm\" onclick=\"exportQa()\">⤓ {qa_export}</button></div>\
         <div id=\"qalog\" class=\"qalog\"></div></section>",
        esc(&i18n::trf("Ask about {0}'s chart", &[esc(&b.name)])),
        ph = esc(&i18n::tr_dyn("e.g. When is a good period for my career?")),
        ask_btn = esc(&i18n::tr_dyn("Ask")),
        qa_title = esc(&i18n::tr_dyn("Previous questions")),
        qa_export = esc(&i18n::tr_dyn("Export Q&A (.md)")),
        dis = if ai_on { "" } else { "disabled" },
        note = if ai_on { esc(&i18n::tr_dyn("The AI answers from this chart and the knowledge base only.")) } else { esc(&i18n::tr_dyn("Turn on the AI writer in Settings to ask questions.")) },
    );
    // Dark-theme override for the report's own tables, plus a layout that gives the site a full-width
    // top bar with the report centered in a readable column (instead of a left-aligned narrow block).
    let dark_css = "@media(prefers-color-scheme:dark){:root:not([data-theme=light]) body{background:#14122b;color:#efe9d8}:root:not([data-theme=light]) table,:root:not([data-theme=light]) td,:root:not([data-theme=light]) th{border-color:#39336a}:root:not([data-theme=light]) tbody tr:nth-child(even) td{background:#1f1c3d}:root:not([data-theme=light]) thead th,:root:not([data-theme=light]) table.kv th{background:#241f45;color:#f4c66a}:root:not([data-theme=light]) .toc{background:#1f1c3d;border-color:#39336a}:root:not([data-theme=light]) div.ai{background:#241f45;border-color:#e6a23c}:root:not([data-theme=light]) div.stale{background:#2d2410;border-color:#e6a23c}:root:not([data-theme=light]) p.flag{color:#f0b86a}:root:not([data-theme=light]) h1,:root:not([data-theme=light]) h3,:root:not([data-theme=light]) .charts figcaption,:root:not([data-theme=light]) p.period{color:#f4c66a}:root:not([data-theme=light]) .toc a{color:#f4c66a}:root:not([data-theme=light]) .cover p,:root:not([data-theme=light]) p.cite{color:#a79fd0}:root:not([data-theme=light]) p.sanskrit{color:#e8a65c}:root:not([data-theme=light]) h2{background:#2a2550;color:#f4c66a;border-left:4px solid #e6a23c}:root:not([data-theme=light]) p.ai-label{color:#e8a65c}:root:not([data-theme=light]) svg.chart rect{fill:#1f1c3d;stroke:#e6a23c}:root:not([data-theme=light]) svg.chart line{stroke:#4f4789}:root:not([data-theme=light]) svg.chart line.asc{stroke:#e8a65c}:root:not([data-theme=light]) svg.chart text.pl{fill:#efe9d8}:root:not([data-theme=light]) svg.chart text.hi{fill:#f0a35a}:root:not([data-theme=light]) svg.chart text.sgn{fill:#8c84b5}:root:not([data-theme=light]) svg.chart text.ttl{fill:#f4c66a}}\
        :root[data-theme=dark] body{background:#14122b;color:#efe9d8}:root[data-theme=dark] table,:root[data-theme=dark] td,:root[data-theme=dark] th{border-color:#39336a}:root[data-theme=dark] tbody tr:nth-child(even) td{background:#1f1c3d}:root[data-theme=dark] thead th,:root[data-theme=dark] table.kv th{background:#241f45;color:#f4c66a}:root[data-theme=dark] .toc{background:#1f1c3d;border-color:#39336a}:root[data-theme=dark] div.ai{background:#241f45;border-color:#e6a23c}:root[data-theme=dark] div.stale{background:#2d2410;border-color:#e6a23c}:root[data-theme=dark] p.flag{color:#f0b86a}:root[data-theme=dark] h1,:root[data-theme=dark] h3,:root[data-theme=dark] .charts figcaption,:root[data-theme=dark] p.period{color:#f4c66a}:root[data-theme=dark] .toc a{color:#f4c66a}:root[data-theme=dark] .cover p,:root[data-theme=dark] p.cite{color:#a79fd0}:root[data-theme=dark] p.sanskrit{color:#e8a65c}:root[data-theme=dark] h2{background:#2a2550;color:#f4c66a;border-left:4px solid #e6a23c}:root[data-theme=dark] p.ai-label{color:#e8a65c}:root[data-theme=dark] svg.chart rect{fill:#1f1c3d;stroke:#e6a23c}:root[data-theme=dark] svg.chart line{stroke:#4f4789}:root[data-theme=dark] svg.chart line.asc{stroke:#e8a65c}:root[data-theme=dark] svg.chart text.pl{fill:#efe9d8}:root[data-theme=dark] svg.chart text.hi{fill:#f0a35a}:root[data-theme=dark] svg.chart text.sgn{fill:#8c84b5}:root[data-theme=dark] svg.chart text.ttl{fill:#f4c66a}";
    let layout_css = "body{max-width:none;margin:0;padding:0}.rbody{max-width:1040px;margin-inline:auto;padding:18px max(16px,env(safe-area-inset-left,0px))}.report-actions,.ask{max-width:none;margin-left:0;margin-right:0;padding-inline:0}";
    let mut out = page;
    out = out.replacen("<head>", &format!("<head><script>{THEME_INIT}</script><link rel=\"stylesheet\" href=\"https://fonts.googleapis.com/css2?family=Marcellus&family=Noto+Sans:wght@400;500;600;700&display=swap\">"), 1);
    out = out.replacen("</style>", &format!("{dark_css}{CSS}{layout_css}</style>"), 1);
    // Full-width bar + hidden regen form, then open the centered content column with the actions,
    // the auspicious Ganesha banner, and (when AI expanded sections) a subtle legend for the * mark.
    let legend = if q.get("ai").is_some() && ai_on {
        format!("<p class=\"ailegend\">{}</p>", esc(&i18n::tr_dyn("Sections marked with * were expanded with AI assistance from the rule-based reading and the knowledge base.")))
    } else {
        String::new()
    };
    let top = format!(
        "{bar}{regen}{rewrite_form}<div class=\"rbody\"><div class=\"report-actions\"><a class=\"btn ghost\" href=\"/\">← {back}</a><button type=\"button\" class=\"btn pdfbtn\" onclick=\"window.print()\">⬇ {pdf}</button>{md_form}</div>{GANESHA}{legend}",
        bar = topbar("report", lc),
        back = esc(&i18n::tr_dyn("New chart")),
        pdf = esc(&i18n::tr_dyn("Download as PDF")),
    );
    // Mark the body when no AI is configured, so the per-section rewrite buttons are hidden (CSS):
    // a click would otherwise POST and replace the whole report with a plain-text "AI not configured".
    let body_tag = if ai_on { "<body>" } else { "<body class=\"ai-off\">" };
    out = out.replacen("<body>", &format!("{body_tag}{top}"), 1);
    out = out.replacen("</body>", &format!("{ask}</div><script>{SCRIPT}</script></body>"), 1);
    out
}

fn geocode(q: &HashMap<String, String>) -> Response {
    let query = q.get("q").cloned().unwrap_or_default();
    match geo::search(&query) {
        Ok(places) => {
            let (y, m, d) = q.get("date").and_then(|s| {
                let v: Vec<i64> = s.split('-').filter_map(|x| x.parse().ok()).collect();
                (v.len() == 3).then(|| (v[0] as i32, v[1] as u32, v[2] as u32))
            }).unwrap_or((2000, 1, 1));
            let mut tt = q.get("time").map(String::as_str).unwrap_or("12:00").split(':').map(|x| x.parse::<u32>().unwrap_or(0));
            let (h, mi) = (tt.next().unwrap_or(12), tt.next().unwrap_or(0));
            let list: Vec<serde_json::Value> = places
                .iter()
                .take(10)
                .map(|p| {
                    let off = geo::utc_offset(&p.tz_name, y, m, d, h.min(23), mi.min(59));
                    serde_json::json!({"label": p.label(), "lat": p.lat, "lon": p.lon, "tz_name": p.tz_name, "tz": off, "utc": off.map(report::fmt_tz)})
                })
                .collect();
            json(serde_json::json!({"places": list}))
        }
        Err(e) => json(serde_json::json!({"error": e})),
    }
}

/// GET /api/reverse — coordinates → the nearest city, named in `lang`. The calendar's "Use my
/// location" calls it so the page says where it put you rather than an anonymous "My location".
fn reverse(q: &HashMap<String, String>) -> Response {
    let (Some(lat), Some(lon)) = (num::<f64>(q, "lat"), num::<f64>(q, "lon")) else {
        return json(serde_json::json!({"error": "coordinates are required"}));
    };
    let lc = ["en", "ta", "hi"][lang_of(q) as usize];
    match geo::reverse(lat, lon, lc) {
        Ok(place) => json(serde_json::json!({"place": place})),
        Err(e) => json(serde_json::json!({"error": e})),
    }
}

/// GET /api/offset — the UTC offset for a zone at a given date/time (so the form can refresh it
/// when the date changes, applying daylight-saving and historical rules).
fn offset(q: &HashMap<String, String>) -> Response {
    let tz_name = q.get("tz_name").cloned().unwrap_or_default();
    if tz_name.trim().is_empty() {
        return json(serde_json::json!({"tz": serde_json::Value::Null}));
    }
    let (y, m, d) = q.get("date").and_then(|s| {
        let v: Vec<i64> = s.split('-').filter_map(|x| x.parse().ok()).collect();
        (v.len() == 3).then(|| (v[0] as i32, v[1] as u32, v[2] as u32))
    }).unwrap_or((2000, 1, 1));
    let mut tt = q.get("time").map(String::as_str).unwrap_or("12:00").split(':').map(|x| x.parse::<u32>().unwrap_or(0));
    let (h, mi) = (tt.next().unwrap_or(12), tt.next().unwrap_or(0));
    let off = geo::utc_offset(&tz_name, y, m, d, h.min(23), mi.min(59));
    json(serde_json::json!({"tz": off, "utc": off.map(report::fmt_tz)}))
}

fn ask(q: &HashMap<String, String>) -> Response {
    // One deliberate question goes to the reasoning tier, not the fast report-body model.
    let Some(cfg) = ask_config() else { return json(serde_json::json!({"error": "The AI writer is not set up on this server. Open Settings to configure it."})) };
    let (b, calc, _) = match birth_from(q) {
        Ok(v) => v,
        Err(e) => return json(serde_json::json!({"error": e})),
    };
    let question = q.get("question").cloned().unwrap_or_default();
    if question.trim().is_empty() {
        return json(serde_json::json!({"error": "Type a question first."}));
    }
    let chart = Chart::compute(b.jd_ut(), b.lat, b.lon, b.tz, calc);
    let lang = lang_of(q);
    let facts = {
        let _guard = RENDER.lock().unwrap_or_else(|e| e.into_inner());
        i18n::set_lang(lang);
        i18n::set_sign_style(sign_style());
        chart_facts(&b, &chart, calc)
    };
    match llm::ask(&cfg, &facts, &question, ["English", "Tamil", "Hindi"][lang as usize]) {
        Ok((answer, files)) => json(serde_json::json!({"answer": answer, "files": files})),
        Err(e) => json(serde_json::json!({"error": e})),
    }
}

/// A plain-text chart summary for the AI's questions. The report computes far more than static
/// positions, so the summary now also carries: the current date, the running dasa→bhukti
/// (antardasa) windows, the slow-planet transits counted from the Moon (gochara), and the
/// birth-star attributes (yoni/gana/nadi, rashi varna/vashya). All of these are already computed
/// elsewhere — the Ask box simply was not being given them — so questions about timing and
/// classical matching/remedies now have something to reason from.
fn chart_facts(b: &BirthData, c: &Chart, calc: Calc) -> String {
    let now = now_jd();
    let mut s = format!(
        "Today is {}.\nBorn {:04}-{:02}-{:02} {:02}:{:02} ({}) at {} ({:.4}, {:.4}).\nAscendant: {}.\n",
        report::fmt_date(now, b.tz), b.year, b.month, b.day, b.hour, b.minute, report::fmt_tz(b.tz), b.place, b.lat, b.lon, i18n::sign(sign_of(c.asc)),
    );
    for p in &c.planets {
        s.push_str(&format!("{}: {} in house {}, star {}.\n", report::planet_label(p), i18n::sign(p.sign()), c.house_of(p.graha), i18n::nak(nakshatra_of(p.lon))));
    }
    // Birth-star (Moon nakshatra) and Moon-sign attributes used in classical matching and temperament.
    let mnak = nakshatra_of(c.moon());
    let msign = sign_of(c.moon());
    s.push_str(&format!(
        "Birth star (Moon) attributes: yoni (animal) {}, gana {}, nadi {}. Moon-sign varna {}, vashya {}.\n",
        YONI_NAMES[yoni_animal(mnak)], GANA_NAMES[GANA[mnak]], NADI_NAMES[nadi(mnak)], VARNA_NAMES[varna(msign)], VASHYA_NAMES[vashya(msign)],
    ));
    // Mahadasha overview (next three) plus the running dasa's antardasa (bhukti) windows.
    let dashas = vimshottari(c.moon(), c.jd_ut, 2);
    for md in dashas.iter().filter(|md| md.end > now).take(3) {
        s.push_str(&format!("Dasa of {}: {} to {}.\n", i18n::planet(md.lord), report::fmt_date(md.start.max(c.jd_ut), b.tz), report::fmt_date(md.end, b.tz)));
    }
    if let Some(md) = dashas.iter().find(|md| md.start <= now && now < md.end) {
        for bk in md.sub.iter().filter(|bk| bk.end > now).take(5) {
            let mark = if bk.start <= now && now < bk.end { " (running now)" } else { "" };
            s.push_str(&format!("{} dasa, {} bhukti (antardasa): {} to {}{}.\n", i18n::planet(md.lord), i18n::planet(bk.lord), report::fmt_date(bk.start.max(c.jd_ut), b.tz), report::fmt_date(bk.end, b.tz), mark));
        }
    }
    // Current transits (gochara) of the slow planets, counted as houses from the birth Moon sign.
    let tc = Chart::compute(now, b.lat, b.lon, b.tz, calc);
    s.push_str(&format!("Current transits (counted from your Moon sign, {}):\n", i18n::sign(msign)));
    for tp in &tc.planets {
        if matches!(tp.graha, Graha::Saturn | Graha::Jupiter | Graha::Rahu | Graha::Ketu) {
            let house = (tp.sign() + 12 - msign) % 12 + 1;
            s.push_str(&format!("  {} now in {} — house {} from the Moon.\n", report::planet_label(tp), i18n::sign(tp.sign()), house));
        }
    }
    let yogas: Vec<&str> = find_yogas(c).iter().map(|y| y.name).collect();
    if !yogas.is_empty() {
        s.push_str(&format!("Yogas: {}.\n", yogas.join(", ")));
    }
    s
}

// ---------------------------------------------------------------- AI settings endpoints

/// GET /api/ai/models — the service's model list, for the Settings picker.
fn ai_models() -> Response {
    let Some(cfg) = ai_probe() else { return json(serde_json::json!({"error": "No AI service is set on the server (VA_AI_BASE_URL)."})) };
    match llm::list_models(&cfg) {
        Ok(models) => json(serde_json::json!({"models": models, "current": cfg.model})),
        Err(e) => json(serde_json::json!({"error": e})),
    }
}

/// POST /api/ai/test — check the server can reach the service with its key.
fn ai_test() -> Response {
    let Some(cfg) = ai_probe() else { return json(serde_json::json!({"error": "No AI service is set on the server (VA_AI_BASE_URL)."})) };
    match llm::list_models(&cfg) {
        Ok(models) => json(serde_json::json!({"ok": format!("Connected — {} model(s) available.", models.len())})),
        // Some services have no model list; fall back to a one-line chat if a model is chosen.
        Err(e) => {
            if cfg.model.trim().is_empty() {
                return json(serde_json::json!({"error": e}));
            }
            match llm::test(&cfg) {
                Ok(_) => json(serde_json::json!({"ok": "Connected."})),
                Err(e2) => json(serde_json::json!({"error": e2})),
            }
        }
    }
}

/// POST /api/settings — save the model, temperature, passages, engine and AI instructions.
fn save_settings(q: &HashMap<String, String>) -> Response {
    {
        let mut ov = OVERRIDES.lock().unwrap_or_else(|e| e.into_inner());
        ov.model = q.get("model").cloned().filter(|m| !m.trim().is_empty());
        ov.ask_model = q.get("ask_model").cloned().filter(|m| !m.trim().is_empty());
        ov.temperature = num(q, "temperature");
        ov.passages = num(q, "passages");
        ov.weekday_civil = q.get("weekday").map(|w| w == "civil");
        ov.sign_style = q.get("signs").cloned().filter(|s| s == "regional" || s == "western" || s == "both");
    }
    // AI instructions are stored in the knowledge folder so the AI writer picks them up.
    if let Some(text) = q.get("instructions").filter(|t| !t.trim().is_empty())
        && let Some(dir) = knowledge::dir() {
            let ai_dir = dir.join("ai");
            let _ = std::fs::create_dir_all(&ai_dir);
            let _ = std::fs::write(ai_dir.join("instructions.md"), text);
        }
    json(serde_json::json!({"ok": "Saved."}))
}

// ---------------------------------------------------------------- calendar

fn calendar_page(q: &HashMap<String, String>) -> Response {
    let lang = lang_of(q);
    let lat = num(q, "lat").unwrap_or(13.0827);
    let lon = num(q, "lon").unwrap_or(80.2707);
    let tz = num(q, "tz").unwrap_or(5.5);
    let tz_name = q.get("tz_name").cloned().unwrap_or_else(|| if q.contains_key("lat") { String::new() } else { "Asia/Kolkata".into() });
    let place = q.get("place").cloned().unwrap_or_else(|| "Chennai, India".into());
    // Festival traditions to overlay (comma-separated `trad=`); unknown names are ignored.
    let trad_param = q.get("trad").cloned().unwrap_or_default();
    let trads: Vec<festivals::Tradition> = trad_param.split(',').filter_map(festivals::Tradition::parse).collect();
    let (ny, nm, _, _) = crate::astro::calendar(now_jd() + tz / 24.0);
    let y: i32 = num(q, "y").unwrap_or(ny);
    let m: u32 = num::<u32>(q, "m").unwrap_or(nm).clamp(1, 12);
    let days = calendar::month(y, m, lat, lon, &tz_name, tz, Calc::default());
    let lc = ["en", "ta", "hi"][lang as usize];
    let _guard = RENDER.lock().unwrap_or_else(|e| e.into_inner());
    i18n::set_lang(lang);
    let t = |s: &'static str| esc(i18n::tr(s));
    let link = |yy: i32, mm: u32| format!("/calendar?y={yy}&m={mm}&lat={lat}&lon={lon}&tz={tz}&tz_name={}&place={}&trad={}&lang={lc}", url(&tz_name), url(&place), url(&trad_param));
    let (py, pm) = if m == 1 { (y - 1, 12) } else { (y, m - 1) };
    let (xy, xm) = if m == 12 { (y + 1, 1) } else { (y, m + 1) };
    let year_name = i18n::trf("{0} year", &[calendar::year_name(days.first().map(|d| d.tamil_year).unwrap_or(0)).into()]);
    let mut out = head(&format!("{} — {} {}", i18n::tr("Tamil / Vedic calendar"), i18n::month_short(m), y), lc);
    out.push_str(&topbar("calendar", lc));
    out.push_str(&format!(
        "<main class=\"wrap\"><div class=\"cal-head\"><div><h1 class=\"page-title\">{} {}</h1><p class=\"note\">{} · {} · {}</p></div>\
         <div class=\"cal-nav\"><a class=\"btn ghost\" href=\"{}\" aria-label=\"{}\">◀</a>\
         <input type=\"month\" class=\"monthpick\" value=\"{}-{:02}\" onchange=\"goMonth(this.value)\" aria-label=\"{}\">\
         <a class=\"btn ghost\" href=\"{}\" aria-label=\"{}\">▶</a></div></div>",
        esc(i18n::month_short(m)), y,
        t("Tamil / Vedic calendar"), esc(&place), esc(&year_name),
        link(py, pm), t("Prev"), y, m, t("Go to month"), link(xy, xm), t("Next"),
    ));
    // Location + festival-traditions controls. The JS (app.js) auto-detects the timezone, handles the
    // geolocation button, the city picker (via /api/geocode) and the multiselect, remembering choices.
    let tradboxes: String = festivals::Tradition::all()
        .iter()
        .map(|tr| {
            let v = tr.key().to_ascii_lowercase();
            let chk = if trads.contains(tr) { " checked" } else { "" };
            format!("<label class=\"tradopt\"><input type=\"checkbox\" class=\"tradbox\" value=\"{v}\"{chk} onchange=\"applyTrad()\"> {}</label>", esc(&i18n::tr_dyn(tr.key())))
        })
        .collect();
    out.push_str(&format!(
        "<div class=\"cal-controls\">\
         <button type=\"button\" class=\"btn ghost\" onclick=\"useMyLocation()\">📍 {useloc}</button>\
         <form class=\"cityf\" onsubmit=\"return changeCity(event)\">\
         <input id=\"cityq\" class=\"cityq\" type=\"search\" name=\"cityq\" placeholder=\"{cityph}\" aria-label=\"{cityph}\" \
         autocapitalize=\"off\" autocorrect=\"off\" spellcheck=\"false\" enterkeyhint=\"search\">\
         <button type=\"submit\" class=\"btn ghost\">{changecity}</button></form>\
         <details class=\"tradsel\"><summary class=\"btn ghost\">{fest} ▾</summary>\
         <div class=\"tradmenu\">{tradboxes}<button type=\"button\" class=\"btn ghost sm\" onclick=\"clearTrad()\">{clear}</button></div></details>\
         </div><p class=\"cal-msg\" id=\"calmsg\" role=\"status\" aria-live=\"polite\"></p>",
        useloc = t("Use my location"),
        cityph = t("City"),
        changecity = t("Change city"),
        fest = t("Festivals"),
        clear = t("Clear"),
    ));
    out.push_str("<div class=\"cal-scroll\"><table class=\"cal\"><thead><tr>");
    for (i, h) in ["Date", "Tamil date", "Lunar day (tithi)", "Star (nakshatra)", "Observance", "Sunrise / sunset", "Rahu kalam (avoid)", "Yamagandam (avoid)", "Gulika kalam (avoid new starts)", "Abhijit muhurta (auspicious)", "Brahma muhurta (auspicious)"].iter().enumerate() {
        let cls = if i == 0 { " class=\"sticky\"" } else { "" };
        out.push_str(&format!("<th{cls}>{}</th>", t(h)));
    }
    if !trads.is_empty() {
        out.push_str(&format!("<th>{}</th>", t("Festivals")));
    }
    out.push_str("</tr></thead><tbody>");
    for d in &days {
        let dz = d.tz;
        let tm = |j: Option<f64>| report::fmt_time(j, dz).chars().take(5).collect::<String>();
        let span = |p: Option<(f64, f64)>| p.map(|(a, b)| format!("{}–{}", tm(Some(a)), tm(Some(b)))).unwrap_or_else(|| "—".into());
        out.push_str(&format!(
            "<tr{}><td class=\"sticky\"><b>{}</b> {}</td><td>{} {}</td><td>{}</td><td>{}</td><td>{}</td><td>{} / {}</td><td class=\"bad\">{}</td><td class=\"bad\">{}</td><td class=\"bad\">{}</td><td class=\"good\">{}</td><td class=\"good\">{}</td>",
            if d.weekday == 0 { " class=\"sun\"" } else { "" },
            d.day, esc(i18n::weekday(d.weekday)),
            esc(calendar::tamil_month_name(d.tamil_month)), d.tamil_day,
            esc(i18n::tithi(d.tithi)), esc(i18n::nak(d.nakshatra)),
            calendar::observance(d).map(|o| esc(i18n::tr(o))).unwrap_or_default(),
            tm(d.sunrise), tm(d.sunset),
            span(d.rahu), span(d.yama), span(d.gulika), span(d.abhijit), span(d.brahma)
        ));
        if !trads.is_empty() {
            let names = festivals::festivals_for(d, &trads).iter().map(|&k| esc(&i18n::tr_dyn(k))).collect::<Vec<_>>().join("<br>");
            out.push_str(&format!("<td class=\"fest\">{names}</td>"));
        }
        out.push_str("</tr>");
    }
    let fest_note = if trads.is_empty() {
        String::new()
    } else {
        format!("<p class=\"note\">{}</p>", t("Festival dates are computed estimates and may differ by a day from local observance; Islamic dates are tabular."))
    };
    out.push_str(&format!(
        "</tbody></table></div><p class=\"note\">{} ({})</p>{fest_note}</main><script>{SCRIPT}</script></body></html>",
        t("Each day shows the Tamil date, and the lunar day (tithi) and star (nakshatra) at sunrise. Pick a day for its timings."),
        report::fmt_tz(days.first().map(|d| d.tz).unwrap_or(tz))
    ));
    html(out)
}

fn url(s: &str) -> String {
    s.bytes().map(|b| if b.is_ascii_alphanumeric() || b"-_.~".contains(&b) { (b as char).to_string() } else { format!("%{b:02X}") }).collect()
}

// ---------------------------------------------------------------- home & settings

fn index_page(q: &HashMap<String, String>) -> String {
    let presets: String = crate::geo::PLACES
        .iter()
        .map(|(name, lat, lon, tz, zone)| format!("<option data-lat='{lat}' data-lon='{lon}' data-tz='{tz}' data-zone='{zone}'>{}</option>", esc(name)))
        .collect();
    let lang = lang_of(q);
    let lc = ["en", "ta", "hi"][lang as usize];
    let _guard = RENDER.lock().unwrap_or_else(|e| e.into_inner());
    i18n::set_lang(lang);
    let ai_on = ai_config().is_some();
    let mut page = head(&i18n::tr_dyn("Vedic Astrology — your Vedic birth chart"), lc);
    page.push_str(&topbar("report", lc));
    page.push_str(
        &i18n_pass(INDEX)
            .replace("{PRESETS}", &presets)
            .replace("{LANG}", lc)
            .replace("{L_EN_SEL}", if lc == "en" { "selected" } else { "" })
            .replace("{L_TA_SEL}", if lc == "ta" { "selected" } else { "" })
            .replace("{L_HI_SEL}", if lc == "hi" { "selected" } else { "" })
            .replace("{AI_DIS}", if ai_on { "" } else { "disabled" })
            .replace("{AI_NOTE}", &esc(&i18n::tr_dyn(if ai_on { "Adds an AI-written reading to each section." } else { "Turn on the AI writer in Settings first." })))
            .replace("{SCRIPT}", SCRIPT),
    );
    page
}

fn settings_page(q: &HashMap<String, String>) -> String {
    let lang = lang_of(q);
    let lc = ["en", "ta", "hi"][lang as usize];
    let base = ai_base_url();
    let _guard = RENDER.lock().unwrap_or_else(|e| e.into_inner());
    i18n::set_lang(lang);
    let ov = OVERRIDES.lock().unwrap_or_else(|e| e.into_inner());
    let model = ov.model.clone().or_else(|| env("VA_AI_MODEL")).unwrap_or_default();
    let ask_model_val = ov.ask_model.clone().or_else(|| env("VA_AI_ASK_MODEL")).unwrap_or_default();
    let temp = ov.temperature.unwrap_or(0.4);
    let passages = ov.passages.unwrap_or(4);
    let wd_civil = ov.weekday_civil.unwrap_or(false);
    let signs = ov.sign_style.clone().unwrap_or_else(|| "regional".into());
    drop(ov);
    // What the AI writer will consult as extra context (RAG): rule records, glossary, passages.
    let (kb_rules, kb_glossary, kb_passages) = knowledge::summary();
    let instructions = knowledge::ai_instructions("Report sections").unwrap_or_default();
    let status = match &base {
        Some(u) => {
            // Show the host only, never the key.
            let host = u.split('/').nth(2).unwrap_or(u);
            format!("<span class=\"pill ok\">● {}</span> <span class=\"note\">{}</span>", esc(&i18n::tr_dyn("Service set")), esc(host))
        }
        None => format!("<span class=\"pill off\">● {}</span> <span class=\"note\">{}</span>", esc(&i18n::tr_dyn("Not set")), esc(&i18n::tr_dyn("Set VA_AI_BASE_URL and VA_AI_API_KEY on the server to enable the AI writer."))),
    };
    let disabled = if base.is_some() { "" } else { "disabled" };
    let mut page = head(&i18n::tr_dyn("Settings — Vedic Astrology"), lc);
    page.push_str(&topbar("settings", lc));
    page.push_str(
        &i18n_pass(SETTINGS)
            .replace("{STATUS}", &status)
            .replace("{MODEL}", &esc(&model))
            .replace("{ASK_MODEL}", &esc(&ask_model_val))
            .replace("{TEMP}", &format!("{temp}"))
            .replace("{PASSAGES}", &format!("{passages}"))
            .replace("{WD_ASTRO_SEL}", if wd_civil { "" } else { "selected" })
            .replace("{WD_CIVIL_SEL}", if wd_civil { "selected" } else { "" })
            .replace("{SIGN_REG}", if signs == "regional" { "selected" } else { "" })
            .replace("{SIGN_WEST}", if signs == "western" { "selected" } else { "" })
            .replace("{SIGN_BOTH}", if signs == "both" { "selected" } else { "" })
            .replace("{KB_RULES}", &kb_rules.to_string())
            .replace("{KB_GLOSSARY}", &kb_glossary.to_string())
            .replace("{KB_PASSAGES}", &kb_passages.to_string())
            .replace("{INSTRUCTIONS}", &esc(&instructions))
            .replace("{DIS}", disabled)
            .replace("{SCRIPT}", SCRIPT),
    );
    page
}

const INDEX: &str = include_str!("web_assets/index.html");
const SETTINGS: &str = include_str!("web_assets/settings.html");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn form_decoding() {
        let f = form("place=Madurai%2C+Tamil+Nadu&name=%E0%AE%95&x=50%25");
        assert_eq!(f["place"], "Madurai, Tamil Nadu");
        assert_eq!(f["name"], "க");
        assert_eq!(f["x"], "50%");
    }

    fn urlencode(s: &str) -> String {
        s.bytes().map(|b| if b.is_ascii_alphanumeric() || b"-_.~".contains(&b) { (b as char).to_string() } else { format!("%{b:02X}") }).collect()
    }

    fn bad_form_lat_999() -> HashMap<String, String> {
        let mut f = HashMap::new();
        for (k, v) in [("name", "X"), ("gender", "m"), ("date", "1990-08-15"), ("time", "06:30"), ("place", "Nowhere"), ("lat", "999"), ("lon", "80"), ("tz", "5.5"), ("tz_name", "Asia/Kolkata"), ("ayanamsa", "lahiri"), ("style", "south"), ("node", "mean"), ("lang", "en")] {
            f.insert(k.to_string(), v.to_string());
        }
        f
    }

    #[test]
    fn stale_block_has_a_dark_mode_style() {
        // The flagged stale prose must stay readable in dark mode (light-on-light bug otherwise).
        let body = "name=Soverna&gender=f&date=2012-12-04&time=20%3A06&place=Illinois&lat=40&lon=-89.25&tz=-6&tz_name=America%2FChicago&lang=en&style=south&ayanamsa=lahiri&node=mean";
        let page = String::from_utf8_lossy(&handle("POST", "/report", "", body).body).into_owned();
        assert!(page.contains("[data-theme=dark] div.stale"), "no dark-theme override for the stale block");
    }

    #[test]
    fn rewrite_button_hidden_when_ai_off() {
        // Tests run with no AI service, so a reopened+refreshed report must mark the body ai-off and
        // hide the rewrite buttons by CSS — clicking one would otherwise replace the report with a 400.
        let genq = "name=Soverna&gender=f&date=2012-12-04&time=20%3A06&place=Illinois&lat=40&lon=-89.25&tz=-6&tz_name=America%2FChicago&lang=en&style=south&ayanamsa=lahiri&node=mean&as_of_jd=2456266.0&ai=%7B%22transit%22%3A%22old%20prose%22%7D";
        let md = String::from_utf8_lossy(&handle("POST", "/report.md", "", genq).body).into_owned();
        let p = String::from_utf8_lossy(&handle("POST", "/report/open", "", &format!("file={}&refresh=1", urlencode(&md))).body).into_owned();
        assert!(p.contains(r#"class="ai-off""#), "body not marked ai-off when AI is unconfigured");
        assert!(p.contains(".ai-off button.rewrite"), "no CSS rule hiding the rewrite button when AI is off");
    }

    #[test]
    fn rewrite_section_requires_ai_and_targets_one_key() {
        // With no AI service configured, the endpoint refuses rather than silently doing nothing.
        let body = "name=Soverna&gender=f&date=2012-12-04&time=20%3A06&place=Illinois&lat=40&lon=-89.25&tz=-6&tz_name=America%2FChicago&lang=en&style=south&ayanamsa=lahiri&node=mean&as_of_jd=2456266.0&ai=%7B%7D&fresh=&section=transit";
        assert_eq!(handle("POST", "/report/rewrite-section", "", body).status, 400);
    }

    #[test]
    fn reopen_round_trips_and_refreshes() {
        // Generate a Hindi report, download its .md, reopen it (view), then refresh it.
        let genq = "name=Soverna&gender=f&date=2012-12-04&time=20%3A06&place=Illinois&lat=40&lon=-89.25&tz=-6&tz_name=America%2FChicago&lang=hi&style=south&ayanamsa=lahiri&node=mean&as_of_jd=2456266.0&ai=%7B%22transit%22%3A%22old%20transit%20prose%22%7D";
        let md = String::from_utf8_lossy(&handle("POST", "/report.md", "", genq).body).into_owned();
        let field = format!("file={}", urlencode(&md));
        let view = handle("POST", "/report/open", "", &field);
        assert_eq!(view.status, 200);
        let vp = String::from_utf8_lossy(&view.body);
        assert!(vp.contains("Soverna"), "chart rebuilt from the file");
        assert!(vp.contains("old transit prose"), "saved AI prose restored");
        assert!(vp.chars().any(|c| ('\u{0900}'..='\u{097F}').contains(&c)), "Hindi (Devanagari) text restored");
        let refreshed = handle("POST", "/report/open", "", &format!("{field}&refresh=1"));
        assert_eq!(refreshed.status, 200);
        assert!(String::from_utf8_lossy(&refreshed.body).contains(r#"class="stale""#), "refresh flags the stale transit prose");
    }

    #[test]
    fn reopen_rejects_bad_file_and_out_of_range() {
        assert_eq!(handle("POST", "/report/open", "", "file=not%20a%20save%20file").status, 400);
        let bad = save::to_markdown_file("<h1>x</h1>", &bad_form_lat_999(), 2456266.0, &std::collections::BTreeMap::new());
        assert_eq!(handle("POST", "/report/open", "", &format!("file={}", urlencode(&bad))).status, 400);
    }

    #[test]
    fn refresh_of_rule_only_report_does_not_crash() {
        let genq = "name=Ravi&gender=m&date=1990-08-15&time=06%3A30&place=Chennai&lat=13.08&lon=80.27&tz=5.5&tz_name=Asia%2FKolkata&lang=en&style=south&ayanamsa=lahiri&node=mean&as_of_jd=2447000.0&ai=%7B%7D";
        let md = String::from_utf8_lossy(&handle("POST", "/report.md", "", genq).body).into_owned();
        let r = handle("POST", "/report/open", "", &format!("file={}&refresh=1", urlencode(&md)));
        assert_eq!(r.status, 200);
        assert!(!String::from_utf8_lossy(&r.body).contains(r#"class="stale""#), "no stale flag without saved AI");
    }

    #[test]
    fn report_markdown_embeds_block_and_current_ai() {
        // `ai` carries the page's current section texts (including any rewrites); the sentinel must
        // land in the downloaded file, proving the download reflects page state, not a recompute.
        let ai = "%7B%22transit%22%3A%22SENTINEL%20transit%20text%22%7D"; // {"transit":"SENTINEL transit text"}
        let body = format!("name=Soverna&gender=f&date=2012-12-04&time=20%3A06&place=Illinois&lat=40&lon=-89.25&tz=-6&tz_name=America%2FChicago&lang=en&style=south&ayanamsa=lahiri&node=mean&as_of_jd=2456266.0&ai={ai}");
        let r = handle("POST", "/report.md", "", &body);
        assert_eq!(r.status, 200);
        let md = String::from_utf8_lossy(&r.body);
        assert!(md.contains("vedic-astrology-save v1"), "embeds the reload block");
        assert!(md.contains("SENTINEL transit text"), "download carries the page's current AI text");
        assert!(r.headers.iter().any(|(k, _)| *k == "Content-Disposition"));
    }

    #[test]
    fn calendar_header_has_location_and_traditions_controls() {
        let page = String::from_utf8_lossy(&handle("GET", "/calendar", "y=2026&m=12", "").body).into_owned();
        assert!(page.contains("useMyLocation") || page.contains("Use my location"), "location control missing");
        assert!(page.contains("trad") && page.contains("type=\"checkbox\""), "traditions multiselect missing");
    }

    #[test]
    fn calendar_city_picker_is_submittable_and_has_a_status_line() {
        // iOS swallows the tap that dismisses the on-screen keyboard, so the city input must also be
        // submittable from the keyboard's Go key; and every failure path needs somewhere to speak.
        let page = String::from_utf8_lossy(&handle("GET", "/calendar", "y=2026&m=12", "").body).into_owned();
        assert!(page.contains("onsubmit=\"return changeCity(event)\""), "city input is not a submittable form");
        assert!(page.contains("id=\"calmsg\"") && page.contains("aria-live=\"polite\""), "status line missing");
        assert!(page.contains("autocapitalize=\"off\"") && page.contains("enterkeyhint=\"search\""), "mobile keyboard hints missing");
    }

    #[test]
    fn ai_report_is_posted_without_leaving_the_page() {
        // WebKit suspends a page that is navigating away, which froze the elapsed timer at 0:00 while
        // the compositor-driven spinner kept turning. The overlay only ticks if we never navigate.
        assert!(SCRIPT.contains("postPage"), "no in-page post helper");
        assert_eq!(SCRIPT.matches("postPage(").count(), 4, "all three overlay paths must post in place (plus the definition)");
        assert!(!SCRIPT.contains("regen.submit()"), "the language switch still navigates");
    }

    #[test]
    fn favicon_is_served_at_the_well_known_path() {
        let r = handle("GET", "/favicon.ico", "", "");
        assert_eq!(r.status, 200);
        assert_eq!(r.content_type, "image/x-icon");
        // ICO magic: reserved=0, type=1 (icon), then the image count.
        assert_eq!(&r.body[..4], &[0, 0, 1, 0], "not an .ico");
        assert!(u16::from_le_bytes([r.body[4], r.body[5]]) >= 2, "only one size: no hi-dpi entry");
        for path in ["/", "/settings", "/calendar"] {
            let page = String::from_utf8_lossy(&handle("GET", path, "", "").body).into_owned();
            assert!(page.contains("href=\"/favicon.ico\""), "{path} does not link the .ico");
        }
    }

    #[test]
    fn reverse_lookup_needs_coordinates_and_follows_the_page_language() {
        let r = String::from_utf8_lossy(&handle("GET", "/api/reverse", "lang=ta", "").body).into_owned();
        assert!(r.contains("error"), "missing coordinates must be reported, not guessed at");
        assert!(SCRIPT.contains("/api/reverse?lat="), "the location button does not ask for a city name");
        assert!(!SCRIPT.contains("place: 'My location'"), "the place label is still hardcoded in English");
    }

    #[test]
    fn calendar_location_failures_are_reported() {
        assert!(SCRIPT.contains("isSecureContext"), "insecure-origin geolocation refusal is not explained");
        assert!(SCRIPT.contains("timeout:15000"), "getCurrentPosition has no timeout, so a hang shows nothing");
        assert!(SCRIPT.contains("calSay"), "no way to report a failed lookup");
    }

    #[test]
    fn calendar_shows_selected_festivals_only() {
        let dec = String::from_utf8_lossy(&handle("GET", "/calendar", "y=2026&m=12&trad=christian", "").body).into_owned();
        assert!(dec.contains("Christmas"), "Dec 25 should be labeled Christmas when Christian is selected");
        let none = String::from_utf8_lossy(&handle("GET", "/calendar", "y=2026&m=12", "").body).into_owned();
        assert!(!none.contains("Christmas"), "no festival labels without a trad selection");
        assert_eq!(handle("GET", "/calendar", "y=2026&m=12&trad=bogus", "").status, 200); // garbage ignored
    }

    #[test]
    fn download_sets_attachment_header() {
        let r = download("body".into(), "soverna-2026-11-06.md", "text/markdown; charset=utf-8");
        assert!(r.headers.iter().any(|(k, v)| *k == "Content-Disposition" && v.contains("attachment") && v.contains("soverna-2026-11-06.md")));
        assert_eq!(r.content_type, "text/markdown; charset=utf-8");
    }

    #[test]
    fn pages_render() {
        let r = handle("GET", "/", "", "");
        assert_eq!(r.status, 200);
        assert!(String::from_utf8_lossy(&r.body).contains("Fetch"));
        assert_eq!(handle("GET", "/health", "", "").status, 200);
        assert_eq!(handle("GET", "/settings", "", "").status, 200);
        assert_eq!(handle("GET", "/nope", "", "").status, 404);
        let body = "name=Soverna&gender=f&date=2012-12-04&time=20%3A06&place=Illinois&lat=40&lon=-89.25&tz=-6&tz_name=America%2FChicago&lang=en&style=south&ayanamsa=lahiri&node=mean";
        let r = handle("POST", "/report", "", body);
        let page = String::from_utf8_lossy(&r.body);
        assert_eq!(r.status, 200);
        assert!(page.contains("Horoscope of Soverna") && page.contains("Chandra Dasa (Moon)"));
        assert!(page.contains("Ask about"), "the ask box moved onto the report page");
        let r = handle("GET", "/calendar", "y=2024&m=1&lang=ta", "");
        let page = String::from_utf8_lossy(&r.body);
        assert!(page.contains("தை"), "Tamil month names in the Tamil calendar");
        assert_eq!(handle("POST", "/report", "", "date=bad").status, 400);
    }

    #[test]
    fn footer_offers_source_on_every_page() {
        // AGPL §13: a network user must be offered the Corresponding Source. The footer carries that
        // offer, in the page's own language, on every full HTML page.
        for path in ["/", "/settings", "/calendar"] {
            let page = String::from_utf8_lossy(&handle("GET", path, "", "").body).into_owned();
            assert!(page.contains("</footer>"), "{path} has no footer");
            assert!(page.contains("AGPL-3.0"), "{path} footer omits the licence");
            assert!(page.contains(SOURCE_URL_DEFAULT), "{path} footer omits the source link");
            // ODbL: the reverse lookup that names "Use my location" displays OpenStreetMap data.
            assert!(page.contains("OpenStreetMap"), "{path} footer omits the place-data credit");
        }
        // A report (POST) carries it too, and the label follows the chosen language.
        let body = "name=Soverna&gender=f&date=2012-12-04&time=20%3A06&place=Illinois&lat=40&lon=-89.25&tz=-6&tz_name=America%2FChicago&lang=hi&style=south&ayanamsa=lahiri&node=mean";
        let page = String::from_utf8_lossy(&handle("POST", "/report", "", body).body).into_owned();
        assert!(page.contains("सोर्स कोड"), "Hindi report footer should use the Hindi source label");

        // `VA_SOURCE_URL` overrides the default link (done here, not in a separate test, because the
        // env var is process-global and parallel tests would otherwise race on it).
        // SAFETY: single-threaded within this test; `source_url` only reads the var.
        unsafe { std::env::set_var("VA_SOURCE_URL", "https://example.org/my/fork") };
        let page = String::from_utf8_lossy(&handle("GET", "/", "", "").body).into_owned();
        assert!(page.contains("https://example.org/my/fork"), "VA_SOURCE_URL should override the default");
        unsafe { std::env::remove_var("VA_SOURCE_URL") };
    }
}
