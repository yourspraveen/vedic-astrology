//! The full horoscope report: a self-contained, print-ready web page (charts drawn as SVG) in the
//! current language. Opened in the browser, it can be printed or saved as a PDF there.

use crate::astro::{self, norm360};
use crate::style::Style;
use crate::extra::*;
use crate::i18n::{self, tr, trf};
use crate::readings::{self, R};
use crate::texts::{self, Tpl};
use crate::vedic::*;
use crate::knowledge::{self, RuleText};
use std::collections::HashMap;
use std::fmt::Write;

// ---------------------------------------------------------------- shared formatting

pub fn fmt_date(jd_ut: f64, tz: f64) -> String {
    let (y, m, d, _) = astro::calendar(jd_ut + tz / 24.0);
    format!("{:02}-{}-{}", d, i18n::month_short(m), y)
}

pub fn fmt_datetime(jd_ut: f64, tz: f64) -> String {
    let (y, m, d, h) = astro::calendar(jd_ut + tz / 24.0);
    let mins = (h * 60.0).round() as i64;
    format!("{:02}-{}-{} {:02}:{:02}", d, i18n::month_short(m), y, mins / 60 % 24, mins % 60)
}

pub fn fmt_time(jd_ut: Option<f64>, tz: f64) -> String {
    match jd_ut {
        Some(j) => {
            let (_, _, _, h) = astro::calendar(j + tz / 24.0);
            let s = (h * 3600.0).round() as i64;
            format!("{:02}:{:02}:{:02}", s / 3600 % 24, s / 60 % 60, s % 60)
        }
        None => "—".into(),
    }
}

pub fn fmt_tz(tz: f64) -> String {
    let m = (tz.abs() * 60.0).round() as i64;
    format!("UTC{}{:02}:{:02}", if tz < 0.0 { "-" } else { "+" }, m / 60, m % 60)
}

fn esc(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;").replace('"', "&quot;")
}

/// Planet name with (R) when retrograde.
pub fn planet_label(p: &Position) -> String {
    if p.retro() { format!("{} ({})", i18n::planet(p.graha), tr("Retrograde")) } else { i18n::planet(p.graha).to_string() }
}

/// Kuja, Rahu and Ketu dosha results as (heading, lines), following these rules:
/// all three judged from the ascendant in houses 1, 2, 4, 7, 8, 12; Jupiter's aspect or
/// company softens Rahu and Ketu doshas.
pub fn dosha_sections(c: &Chart) -> Vec<(&'static str, Vec<String>)> {
    let mut out = Vec::new();
    let (from_lagna, _) = kuja_dosha(c);
    let moon_house = (c.get(Graha::Mars).sign() + 12 - c.get(Graha::Moon).sign()) % 12 + 1;
    out.push((
        tr("Mars (Kuja) dosha"),
        vec![
            trf("Mars is in house {0} from the Ascendant and house {1} from the Moon.", &[c.house_of(Graha::Mars).to_string(), moon_house.to_string()]),
            s(tr(if from_lagna {
                "Kuja dosha is present with respect to the Ascendant."
            } else {
                "No Kuja dosha with respect to the Ascendant (Mars is not in houses 1, 2, 4, 7, 8 or 12)."
            })),
        ],
    ));
    for (node, title) in [(Graha::Rahu, tr("Rahu dosha")), (Graha::Ketu, tr("Ketu dosha"))] {
        let d = node_dosha(c, node);
        let name = s(i18n::planet(node));
        let mut lines = vec![if d.present {
            trf("{0} is in house {1} from the Ascendant: {0} dosha is present.", &[name.clone(), d.house.to_string()])
        } else {
            trf("No {0} dosha: {0} is in house {1} from the Ascendant (not 1, 2, 4, 7, 8 or 12).", &[name.clone(), d.house.to_string()])
        }];
        if d.present && d.jupiter_with {
            lines.push(trf("Jupiter is together with {0}, which reduces the dosha.", std::slice::from_ref(&name)));
        } else if d.present && d.jupiter_aspects {
            lines.push(trf("Jupiter aspects {0}, which reduces the dosha.", std::slice::from_ref(&name)));
        }
        out.push((title, lines));
    }
    out
}

// ---------------------------------------------------------------- SVG charts

/// One chart as SVG. `cells[sign]` lists (text, highlight) labels.
pub fn svg_chart(style: Style, title: &str, cells: &[Vec<(String, bool)>; 12], asc_sign: usize, size: f32) -> String {
    let mut s = String::new();
    let w = size;
    let _ = write!(s, r##"<svg class="chart" viewBox="0 0 {w} {w}" width="{w}" height="{w}" xmlns="http://www.w3.org/2000/svg"><rect x="1" y="1" width="{a}" height="{a}" fill="#fbf5e6" stroke="#8b1e1e" stroke-width="1.5"/>"##, a = w - 2.0);
    let line = |s: &mut String, x1: f32, y1: f32, x2: f32, y2: f32| {
        let _ = write!(s, r##"<line x1="{x1:.1}" y1="{y1:.1}" x2="{x2:.1}" y2="{y2:.1}" stroke="#8b1e1e" stroke-width="1.2"/>"##);
    };
    let text = |s: &mut String, x: f32, y: f32, t: &str, size: f32, color: &str, cls: &str| {
        let _ = write!(s, r##"<text class="{cls}" x="{x:.1}" y="{y:.1}" font-size="{size:.1}" fill="{color}" text-anchor="middle" dominant-baseline="middle">{}</text>"##, esc(t));
    };
    let labels = |s: &mut String, cx: f32, cy: f32, items: &[(String, bool)], fs: f32| {
        let per_row = 2;
        let rows = items.len().div_ceil(per_row).max(1);
        let lh = fs * 1.3;
        let top = cy - (rows as f32 - 1.0) * lh / 2.0;
        for (i, row) in items.chunks(per_row).enumerate() {
            let n = row.len() as f32;
            let colw = fs * 3.4;
            let left = cx - (n - 1.0) * colw / 2.0;
            for (j, (t, hi)) in row.iter().enumerate() {
                text(s, left + j as f32 * colw, top + i as f32 * lh, t, fs, if *hi { "#b84a00" } else { "#1f2a5a" }, if *hi { "hi" } else { "pl" });
            }
        }
    };
    let fs = w / 24.0;
    match style {
        Style::South => {
            const POS: [(usize, usize); 12] = [(0, 1), (0, 2), (0, 3), (1, 3), (2, 3), (3, 3), (3, 2), (3, 1), (3, 0), (2, 0), (1, 0), (0, 0)];
            let c = w / 4.0;
            for k in 1..4 {
                let v = k as f32 * c;
                if k == 2 {
                    line(&mut s, v, 0.0, v, c);
                    line(&mut s, v, 3.0 * c, v, w);
                    line(&mut s, 0.0, v, c, v);
                    line(&mut s, 3.0 * c, v, w, v);
                } else {
                    line(&mut s, v, 0.0, v, w);
                    line(&mut s, 0.0, v, w, v);
                }
            }
            for (sign, &(r, col)) in POS.iter().enumerate() {
                let (x, y) = (col as f32 * c, r as f32 * c);
                let _ = write!(s, r##"<text class="sgn" x="{:.1}" y="{:.1}" font-size="{:.1}" fill="#9a8a70" text-anchor="end">{}</text>"##, x + c - 3.0, y + c - 4.0, fs * 0.7, esc(i18n::sign_short(sign)));
                if sign == asc_sign {
                    let _ = write!(s, r##"<line class="asc" x1="{:.1}" y1="{:.1}" x2="{:.1}" y2="{:.1}" stroke="#b84a00" stroke-width="1.5"/>"##, x, y + c * 0.28, x + c * 0.28, y);
                }
                labels(&mut s, x + c / 2.0, y + c / 2.0, &cells[sign], fs);
            }
            text(&mut s, w / 2.0, w / 2.0, title, fs * 1.2, "#8b1e1e", "ttl");
        }
        Style::North => {
            line(&mut s, 0.0, 0.0, w, w);
            line(&mut s, w, 0.0, 0.0, w);
            line(&mut s, w / 2.0, 0.0, w, w / 2.0);
            line(&mut s, w, w / 2.0, w / 2.0, w);
            line(&mut s, w / 2.0, w, 0.0, w / 2.0);
            line(&mut s, 0.0, w / 2.0, w / 2.0, 0.0);
            const CENTER: [(f32, f32); 12] = [(0.5, 0.25), (0.25, 0.1), (0.1, 0.25), (0.25, 0.5), (0.1, 0.75), (0.25, 0.9), (0.5, 0.75), (0.75, 0.9), (0.9, 0.75), (0.75, 0.5), (0.9, 0.25), (0.75, 0.1)];
            const NUM: [(f32, f32); 12] = [(0.5, 0.44), (0.25, 0.2), (0.2, 0.25), (0.44, 0.5), (0.2, 0.75), (0.25, 0.8), (0.5, 0.56), (0.75, 0.8), (0.8, 0.75), (0.56, 0.5), (0.8, 0.25), (0.75, 0.2)];
            for house in 0..12 {
                let sign = (asc_sign + house) % 12;
                text(&mut s, NUM[house].0 * w, NUM[house].1 * w, &format!("{}", sign + 1), fs * 0.75, "#9a8a70", "sgn");
                let big = matches!(house, 0 | 3 | 6 | 9);
                labels(&mut s, CENTER[house].0 * w, CENTER[house].1 * w, &cells[sign], if big { fs } else { fs * 0.85 });
            }
        }
        Style::East => {
            let (a, b) = (w / 3.0, 2.0 * w / 3.0);
            line(&mut s, a, 0.0, a, w);
            line(&mut s, b, 0.0, b, w);
            line(&mut s, 0.0, a, w, a);
            line(&mut s, 0.0, b, w, b);
            line(&mut s, 0.0, 0.0, a, a);
            line(&mut s, 0.0, w, a, b);
            line(&mut s, w, w, b, b);
            line(&mut s, w, 0.0, b, a);
            const CENTER: [(f32, f32); 12] = [
                (0.5, 1.0 / 6.0), (2.0 / 9.0, 1.0 / 9.0), (1.0 / 9.0, 2.0 / 9.0), (1.0 / 6.0, 0.5),
                (1.0 / 9.0, 7.0 / 9.0), (2.0 / 9.0, 8.0 / 9.0), (0.5, 5.0 / 6.0), (7.0 / 9.0, 8.0 / 9.0),
                (8.0 / 9.0, 7.0 / 9.0), (5.0 / 6.0, 0.5), (8.0 / 9.0, 2.0 / 9.0), (7.0 / 9.0, 1.0 / 9.0),
            ];
            for sign in 0..12 {
                let full = sign % 3 == 0;
                labels(&mut s, CENTER[sign].0 * w, CENTER[sign].1 * w, &cells[sign], if full { fs } else { fs * 0.8 });
            }
            text(&mut s, w / 2.0, w / 2.0, title, fs * 1.1, "#8b1e1e", "ttl");
        }
    }
    s.push_str("</svg>");
    s
}

/// Chart cells for divisional chart `d`.
pub fn varga_cells(c: &Chart, d: u32) -> [Vec<(String, bool)>; 12] {
    let mut cells: [Vec<(String, bool)>; 12] = Default::default();
    cells[varga_sign(c.asc, d)].push((tr("Asc").to_string(), true));
    for p in &c.planets {
        let mut t = i18n::planet_short(p.graha).to_string();
        if p.retro() {
            t.push_str("(R)");
        }
        cells[varga_sign(p.lon, d)].push((t, false));
    }
    cells
}

// ---------------------------------------------------------------- the report

pub struct Input<'a> {
    pub birth: &'a BirthData,
    pub chart: &'a Chart,
    pub pancha: &'a Panchanga,
    pub dasha: &'a [Period],
    pub calc: Calc,
    pub style: Style,
    pub now: f64,
    /// Show the civil (midnight-to-midnight) weekday everywhere instead of the astrological
    /// vāra (sunrise-to-sunrise). The dedicated row then cross-references the other reckoning.
    pub weekday_civil: bool,
    /// AI section keys whose saved prose was written for an earlier date (set when a saved report is
    /// refreshed to now). Those sections render the prose flagged, with a per-section rewrite button.
    pub stale_keys: &'a [String],
}

/// Does this AI section depend on the as-of date (daśā readings, transits)? Fixed sections — natal
/// chart, yogas, house readings, ashtakavarga, remedies — return false and never flag as stale.
pub fn is_time_based_key(key: &str) -> bool {
    key == "transit" || key.starts_with("dasha-")
}

pub const SECTIONS: [&str; 16] = [
    "Summary",
    "Birth details",
    "Panchanga readings",
    "Planet positions",
    "Charts",
    "House table",
    "Shodasavarga (divisional charts)",
    "Ashtakavarga",
    "Planetary relationships",
    "Benefic and malefic planets",
    "Vimshottari dasha",
    "Yogas and doshas",
    "House readings",
    "Dasha readings",
    "Remedies",
    "Transit forecast",
];

struct Html(String);

impl Html {
    fn h2(&mut self, id: usize, text: &str) {
        let _ = write!(self.0, r#"<h2 id="s{id}">{}</h2>"#, esc(text));
    }
    fn h3(&mut self, text: &str) {
        let _ = write!(self.0, "<h3>{}</h3>", esc(text));
    }
    /// A heading with a subtle trailing asterisk when the section was expanded with AI.
    fn h2a(&mut self, id: usize, text: &str, ai: bool) {
        let _ = write!(self.0, r#"<h2 id="s{id}">{}{}</h2>"#, esc(text), if ai { AIMARK } else { "" });
    }
    fn h3a(&mut self, text: &str, ai: bool) {
        let _ = write!(self.0, "<h3>{}{}</h3>", esc(text), if ai { AIMARK } else { "" });
    }
    fn p(&mut self, text: &str) {
        let _ = write!(self.0, "<p>{}</p>", esc(text));
    }
    fn table(&mut self, head: &[String], rows: &[Vec<String>]) {
        self.0.push_str("<table><thead><tr>");
        for h in head {
            let _ = write!(self.0, "<th>{}</th>", esc(h));
        }
        self.0.push_str("</tr></thead><tbody>");
        for r in rows {
            self.0.push_str("<tr>");
            for c in r {
                let _ = write!(self.0, "<td>{}</td>", esc(c));
            }
            self.0.push_str("</tr>");
        }
        self.0.push_str("</tbody></table>");
    }
    fn kv(&mut self, rows: &[(String, String)]) {
        self.0.push_str(r#"<table class="kv"><tbody>"#);
        for (k, v) in rows {
            let _ = write!(self.0, "<tr><th>{}</th><td>{}</td></tr>", esc(k), esc(v));
        }
        self.0.push_str("</tbody></table>");
    }
    fn raw(&mut self, s: &str) {
        self.0.push_str(s);
    }
}

fn s(x: &str) -> String {
    x.to_string()
}

fn nature_word(n: Nature) -> &'static str {
    readings::t(match n {
        Nature::Benefic => R::Benefic,
        Nature::Malefic => R::Malefic,
        Nature::Neutral => R::Neutral,
    })
}

/// The reading text for a yoga name (several-planet yogas share one text).
fn yoga_group(name: &str) -> &str {
    match name {
        "Trigraha" | "Chaturgraha" | "Panchagraha" => "Graha Malika",
        other => other,
    }
}

/// Section 0: the main points of the horoscope at a glance.
fn summary_section(h: &mut Html, inp: &Input) {
    let c = inp.chart;
    let tz = inp.birth.tz;
    let moon = c.moon();
    let asc_s = c.asc_sign();
    h.h2(0, tr("Summary"));
    h.p(readings::t(R::SummaryIntro));
    let mut rows: Vec<(String, String)> = vec![
        (s(tr("Lagna – lord")), format!("{} – {}", i18n::sign(asc_s), i18n::planet(SIGN_LORD[asc_s]))),
        (s(tr("Birth rasi – lord")), format!("{} – {}", i18n::sign(sign_of(moon)), i18n::planet(SIGN_LORD[sign_of(moon)]))),
        (s(tr("Birth star – pada")), format!("{} – {}", i18n::nak(nakshatra_of(moon)), pada_of(moon))),
    ];
    let now = inp.now.max(c.jd_ut);
    if let Some(md) = inp.dasha.iter().find(|md| md.start <= now && now < md.end) {
        let bk = md.sub.iter().find(|b| b.start <= now && now < b.end);
        let text = match bk {
            Some(bk) => format!("{} – {} ({} {})", i18n::planet(md.lord), i18n::planet(bk.lord), tr("Ends on"), fmt_date(bk.end, tz)),
            None => i18n::planet(md.lord).to_string(),
        };
        rows.push((s(tr("Current dasha – bhukti")), text));
    }
    let list = |n: Nature| -> String {
        let v: Vec<&str> = GRAHAS[..7].iter().filter(|&&g| lordship_nature(asc_s, g).1 == n).map(|&g| i18n::planet(g)).collect();
        if v.is_empty() { "—".into() } else { v.join(", ") }
    };
    rows.push((s(tr("Functional benefics")), list(Nature::Benefic)));
    rows.push((s(tr("Functional malefics")), list(Nature::Malefic)));
    let strong: Vec<&str> = GRAHAS[..7].iter().filter(|&&g| sign_strength(c, g) == 1).map(|&g| i18n::planet(g)).collect();
    let weak: Vec<&str> = GRAHAS[..7].iter().filter(|&&g| sign_strength(c, g) == -1).map(|&g| i18n::planet(g)).collect();
    rows.push((s(tr("Strong planets (exalted or own sign)")), if strong.is_empty() { "—".into() } else { strong.join(", ") }));
    rows.push((s(tr("Weak planets (debilitated or in an enemy's sign)")), if weak.is_empty() { "—".into() } else { weak.join(", ") }));
    let yogas: Vec<&str> = find_yogas(c).iter().map(|y| tr(y.name)).collect();
    rows.push((s(tr("Yogas")), if yogas.is_empty() { "—".into() } else { yogas.join(", ") }));
    let (kuja, _) = kuja_dosha(c);
    let doshas: Vec<String> = [
        (kuja, tr("Mars (Kuja) dosha")),
        (node_dosha(c, Graha::Rahu).present, tr("Rahu dosha")),
        (node_dosha(c, Graha::Ketu).present, tr("Ketu dosha")),
    ]
    .iter()
    .filter(|d| d.0)
    .map(|d| s(d.1))
    .collect();
    rows.push((s(tr("Doshas")), if doshas.is_empty() { s(tr("absent")) } else { doshas.join(", ") }));
    let sav = sarvashtakavarga(&ashtakavarga(c));
    let best = (0..12).max_by_key(|&i| sav[i]).unwrap_or(0);
    let worst = (0..12).min_by_key(|&i| sav[i]).unwrap_or(0);
    rows.push((s(tr("Strongest / weakest sign (Ashtakavarga)")), format!("{} ({}) / {} ({})", i18n::sign(best), sav[best], i18n::sign(worst), sav[worst])));
    let lagna_lord = SIGN_LORD[asc_s] as usize;
    let rem = readings::remedy(lagna_lord);
    rows.push((s(tr("Lucky day, colour and gem (from the Lagna lord)")), format!("{} {} {}", rem.fasting, rem.dress, rem.gem)));
    let adverse = adverse_tara_lords(moon);
    let names: Vec<&str> = adverse.iter().map(|&g| i18n::planet(g)).collect();
    rows.push((s(tr("Less favourable dasas (by birth star)")), names.join(", ")));
    h.kv(&rows);
}

/// Section 9: natural and functional (lordship) benefics and malefics.
fn benefic_section(h: &mut Html, c: &Chart) {
    let asc_s = c.asc_sign();
    h.h2(9, tr("Benefic and malefic planets"));
    h.p(readings::t(R::NaturalIntro));
    h.p(readings::t(R::PakshaRule));
    h.p(readings::t(if moon_has_paksha_bala(c) { R::MoonWith } else { R::MoonWithout }));
    h.p(readings::t(R::MercuryRule));
    h.p(readings::t(if mercury_afflicted(c) { R::MercuryBad } else { R::MercuryGood }));
    let rows: Vec<Vec<String>> = GRAHAS
        .iter()
        .map(|&g| vec![s(i18n::planet(g)), s(nature_word(if is_benefic(c, g) { Nature::Benefic } else { Nature::Malefic }))])
        .collect();
    h.table(&[tr("Planet"), tr("Nature")].map(s), &rows);
    h.h3(tr("Benefic / malefic by lordship of houses"));
    h.p(readings::t(R::LordshipRules));
    let rows: Vec<Vec<String>> = GRAHAS[..7]
        .iter()
        .map(|&g| {
            let (houses, n) = lordship_nature(asc_s, g);
            let hs: Vec<String> = houses.iter().map(|x| x.to_string()).collect();
            vec![s(i18n::planet(g)), hs.join(", "), s(nature_word(n))]
        })
        .collect();
    h.table(&[tr("Planet"), tr("Lordships"), tr("Nature")].map(s), &rows);
}

/// Section 14: star remedies, remedies for less favourable dasas, and dosha remedies.
fn remedies_section(h: &mut Html, inp: &Input) {
    let c = inp.chart;
    let tz = inp.birth.tz;
    let asc_s = c.asc_sign();
    let moon = c.moon();
    let nak = nakshatra_of(moon);
    let star_lord = DASHA[nak % 9].0;
    let rasi = sign_of(moon);
    h.h3(tr("Star remedies"));
    h.p(&i18n::fill(readings::t(R::StarIntro), &[s(i18n::nak(nak)), s(i18n::planet(star_lord)), s(i18n::sign(rasi)), s(i18n::planet(SIGN_LORD[rasi]))]));
    let adverse = adverse_tara_lords(moon);
    let names: Vec<&str> = adverse.iter().map(|&g| i18n::planet(g)).collect();
    h.p(&i18n::fill(readings::t(R::StarAdverse), &[names.join(", ")]));
    h.p(&i18n::fill(readings::t(R::StarAdvice), &[s(i18n::planet(star_lord))]));
    remedy_table(h, star_lord);

    h.h3(tr("Dasa remedies"));
    h.p(readings::t(R::DashaRemedyIntro));
    let start_from = inp.now.max(c.jd_ut);
    let mut any = false;
    for md in inp.dasha.iter().filter(|md| md.end > start_from).take(5) {
        let g = md.lord;
        let mut reasons: Vec<String> = Vec::new();
        if lordship_nature(asc_s, g).1 == Nature::Malefic {
            reasons.push(i18n::fill(readings::t(R::FuncNature), &[s(i18n::planet(g)), s(nature_word(Nature::Malefic)), lordship_nature(asc_s, g).0.iter().map(|x| x.to_string()).collect::<Vec<_>>().join(", ")]));
        }
        if sign_strength(c, g) == -1 {
            reasons.push(format!("{}: {}", i18n::planet(g), tr(dignity(g, c.get(g).sign()))));
        }
        if matches!(c.house_of(g), 6 | 8 | 12) {
            reasons.push(trf("placed in house {0}", &[c.house_of(g).to_string()]));
        }
        if adverse.contains(&g) {
            reasons.push(s(tr("its lord rules an adverse star (tara) from your birth star")));
        }
        if reasons.is_empty() {
            continue;
        }
        any = true;
        h.p(&i18n::fill(readings::t(R::DashaRemedyFor), &[s(i18n::planet(g)), fmt_date(md.start.max(c.jd_ut), tz), fmt_date(md.end, tz), reasons.join("; ")]));
        remedy_table(h, g);
    }
    if !any {
        h.p(readings::t(R::NoDashaRemedy));
    }
    let (kuja, _) = kuja_dosha(c);
    let rahu = node_dosha(c, Graha::Rahu);
    let ketu = node_dosha(c, Graha::Ketu);
    if kuja || rahu.present || ketu.present {
        h.h3(tr("Dosha remedies"));
        if kuja {
            h.p(readings::t(R::KujaRemedy));
        }
        if rahu.present {
            h.p(readings::t(R::RahuRemedy));
        }
        if ketu.present {
            h.p(readings::t(R::KetuRemedy));
        }
    }
}

fn remedy_table(h: &mut Html, g: Graha) {
    let r = readings::remedy(g as usize);
    let l = readings::remedy_labels();
    h.kv(&[
        (s(l[0]), s(r.dress)),
        (s(l[1]), s(r.fasting)),
        (s(l[2]), s(r.alms)),
        (s(l[3]), s(r.flowers)),
        (s(l[4]), s(r.gem)),
        (s(l[5]), s(r.worship)),
        (s(l[6]), s(r.mantra)),
    ]);
}

/// A section the optional AI writer may expand: its key, a title, and the rule-based text it starts from.
pub struct AiTask {
    pub key: String,
    pub title: String,
    pub facts: String,
}

/// English planet names, as the rule files use them.
const PLANET_EN: [&str; 9] = ["Sun", "Moon", "Mars", "Mercury", "Jupiter", "Venus", "Saturn", "Rahu", "Ketu"];

/// A house class from the glossary, in the report's language.
fn house_class(c: &str) -> &'static str {
    match c {
        "kendra" => tr("kendra (angular)"),
        "trikona" => tr("trikona (trine)"),
        "dusthana" => tr("dusthana (difficult)"),
        "upachaya" => tr("upachaya (growing)"),
        "maraka" => tr("maraka"),
        "panaphara" => tr("panaphara (succeeding)"),
        "apoklima" => tr("apoklima (cadent)"),
        _ => "",
    }
}

/// Where a sub-period's results show at each life stage (see `extra::life_stage`).
const LIFE_STAGE_TEXT: [&str; 5] = [
    "At this young age the period shows mostly through health, studies and the family's circumstances.",
    "At this age the period shows mostly through education, friendships and first choices about a career.",
    "At this stage of life the period shows mostly through career, relationships, income and building a home.",
    "At this stage of life the period shows mostly through family responsibilities, wealth, status and the children's progress.",
    "At this stage of life the period shows mostly through health, the family's support, peace of mind and spiritual interests.",
];

/// Traditional Sanskrit names of the planets, for English dasa headings.
const SANSKRIT: [&str; 9] = ["Surya", "Chandra", "Kuja", "Budha", "Guru", "Shukra", "Sani", "Rahu", "Ketu"];

/// Plain text of a piece of the report (tags removed, entities decoded), for the AI writer.
fn plain(html: &str) -> String {
    let mut out = String::new();
    let mut in_tag = false;
    for ch in html.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => {
                in_tag = false;
                out.push('\n');
            }
            _ if !in_tag => out.push(ch),
            _ => {}
        }
    }
    let out = out.replace("&amp;", "&").replace("&lt;", "<").replace("&gt;", ">").replace("&quot;", "\"").replace("&#39;", "'");
    out.lines().map(str::trim).filter(|l| !l.is_empty()).collect::<Vec<_>>().join("\n")
}

/// Where a knowledge-base rule comes from, shown under its text.
fn cite(h: &mut Html, r: &RuleText) {
    let _ = write!(h.0, r#"<p class="cite">{}</p>"#, esc(&trf("Rule {0} — {1}", &[r.id.clone(), r.source.clone()])));
}

/// A subtle asterisk marking a heading whose reading was expanded with AI assistance.
const AIMARK: &str = r#" <span class="aix" title="Expanded with AI assistance from the rule-based reading and the knowledge base">*</span>"#;

/// The AI expansion for a section as non-empty paragraphs, or None (a reasoning model that spent its
/// whole budget thinking can return nothing, which counts as None so the rule-based text stays).
fn ai_paras<'a>(ai: &'a HashMap<String, (String, String)>, key: &str) -> Option<Vec<&'a str>> {
    ai.get(key)
        .map(|(_, t)| t.split("\n\n").map(str::trim).filter(|p| !p.is_empty()).collect::<Vec<_>>())
        .filter(|v| !v.is_empty())
}

/// Houses and dashas: when the AI expanded this section, replace the rule-based prose (everything
/// from `prose_start` to the end of the buffer) with the single AI narrative, so the reading is not
/// stated twice. The compact factual header before `prose_start` is kept.
fn replace_ai(h: &mut Html, ai: &HashMap<String, (String, String)>, key: &str, prose_start: usize, stale: bool) {
    if let Some(paras) = ai_paras(ai, key) {
        h.0.truncate(prose_start);
        emit_paras(h, &paras, stale, key);
    }
}

/// Emit the AI paragraphs, either plainly or — when the saved prose is stale after a refresh —
/// wrapped in a flagged block with a per-section rewrite button. `h.p` escapes each paragraph.
fn emit_paras(h: &mut Html, paras: &[&str], stale: bool, key: &str) {
    if stale {
        h.0.push_str(r#"<div class="stale">"#);
        let _ = write!(h.0, r#"<p class="flag">{}</p>"#, esc(tr("Written for the earlier date — rewrite to update")));
        for p in paras {
            h.p(p);
        }
        let _ = write!(h.0, r#"<button type="button" class="rewrite" data-key="{}">↻ {}</button>"#, esc(key), esc(tr("Rewrite this section with AI")));
        h.0.push_str("</div>");
    } else {
        for p in paras {
            h.p(p);
        }
    }
}

/// Data sections (transit, ashtakavarga, remedies): keep the tables, and add the AI narrative after
/// them when there is one — the prose does not repeat the figures, so nothing is removed.
fn append_ai(h: &mut Html, ai: &HashMap<String, (String, String)>, key: &str, stale: bool) {
    if let Some(paras) = ai_paras(ai, key) {
        emit_paras(h, &paras, stale, key);
    }
}

/// Dashas: replace the repetitive general reading (`prose_start`..`tail_start`) with the AI
/// narrative, but keep the dated sub-period (bhukti) timeline that follows (`tail_start`..end).
fn replace_ai_keep_tail(h: &mut Html, ai: &HashMap<String, (String, String)>, key: &str, prose_start: usize, tail_start: usize, stale: bool) {
    if let Some(paras) = ai_paras(ai, key) {
        let tail = h.0[tail_start..].to_string();
        h.0.truncate(prose_start);
        emit_paras(h, &paras, stale, key);
        h.0.push_str(&tail);
    }
}

pub fn build(inp: &Input) -> String {
    build_with(inp, &HashMap::new()).0
}

/// The report, with AI-written expansions (key → (model, text)) where given, and the sections the AI writer could expand.
pub fn build_with(inp: &Input, ai: &HashMap<String, (String, String)>) -> (String, Vec<AiTask>) {
    let mut tasks: Vec<AiTask> = Vec::new();
    let b = inp.birth;
    let c = inp.chart;
    let p = inp.pancha;
    let tz = b.tz;
    let calc = inp.calc;
    let mut h = Html(String::new());
    let lang_code = match i18n::lang() {
        i18n::Lang::En => "en",
        i18n::Lang::Ta => "ta",
        i18n::Lang::Hi => "hi",
    };
    let _ = write!(
        h.0,
        r#"<!DOCTYPE html><html lang="{lang_code}"><head><meta charset="utf-8"><title>{}</title><style>{CSS}</style></head><body>"#,
        esc(&trf("Horoscope of {0}", std::slice::from_ref(&b.name)))
    );

    // Cover
    h.raw(r#"<div class="cover">"#);
    let _ = write!(h.0, "<h1>{}</h1>", esc(&trf("Horoscope of {0}", std::slice::from_ref(&b.name))));
    h.p(&trf("Prepared with Vedic Astrology on {0}", &[fmt_date(inp.now, tz)]));
    h.raw(r#"<div class="toc"><h3>"#);
    h.raw(&esc(tr("Contents")));
    h.raw("</h3><ol>");
    for (i, sec) in SECTIONS.iter().enumerate() {
        let _ = write!(h.0, r##"<li><a href="#s{i}">{}</a></li>"##, esc(tr(sec)));
    }
    h.raw("</ol></div></div>");

    summary_section(&mut h, inp);

    // 1. Birth details
    let moon = c.moon();
    let msign = sign_of(moon);
    let nak = nakshatra_of(moon);
    let asc_s = c.asc_sign();
    let civil_wd = astro::weekday(b.jd_ut() + tz / 24.0);
    // Which weekday the report shows (vāra by default, civil when the person chose it). The vāra
    // differs from the civil weekday only for a birth before sunrise, so note that when it happens.
    let disp_wd = if inp.weekday_civil { civil_wd } else { p.weekday };
    let wd_note = if !inp.weekday_civil && p.weekday != civil_wd { format!(" ({})", tr("before sunrise")) } else { String::new() };
    let (rise, set) = (p.sunrise, p.sunset);
    let day_len = match (rise, set) {
        (Some(r), Some(st)) => {
            let hrs = (st - r) * 24.0;
            trf("{0} h {1} min  ({2} nazhika)", &[(hrs.floor() as i64).to_string(), ((hrs.fract() * 60.0).round() as i64).to_string(), format!("{:.2}", hrs * 2.5)])
        }
        _ => "—".into(),
    };
    let after_rise = rise
        .map(|r| {
            let mut gh = (c.jd_ut - r) * 60.0; // 1 day = 60 nazhika
            if gh < 0.0 {
                gh += 60.0;
            }
            trf("{0} nazhika {1} vinazhika", &[(gh.floor() as i64).to_string(), ((gh.fract() * 60.0).floor() as i64).to_string()])
        })
        .unwrap_or_else(|| "—".into());
    let lmt = trf("{0} min", &[format!("{:+.1}", b.lon * 4.0 - tz * 60.0)]);
    let sun = c.get(Graha::Sun);
    let western = sign_of(sun.lon + c.ayanamsa);
    let (yogi, _) = yogi_points(c);
    let karakas = chara_karakas(c);
    let ak = karakas[0];
    h.h2(1, tr("Birth details"));
    h.kv(&[
        (s(tr("Name")), b.name.clone()),
        (s(tr("Sex")), s(tr(if b.gender_male { "Male" } else { "Female" }))),
        (s(tr("Date of birth")), format!("{:02}-{}-{}, {}{}", b.day, i18n::month_short(b.month), b.year, i18n::weekday(disp_wd), wd_note)),
        (s(tr("Time of birth")), format!("{:02}:{:02}:{:02}", b.hour, b.minute, b.second)),
        (s(tr("Time zone")), fmt_tz(tz)),
        (s(tr("Place of birth")), b.place.clone()),
        (s(tr("Longitude & latitude")), format!("{:.4}° {} , {:.4}° {}", b.lon.abs(), if b.lon >= 0.0 { "E" } else { "W" }, b.lat.abs(), if b.lat >= 0.0 { "N" } else { "S" })),
        (s(tr("Ayanamsa")), format!("{} = {}", tr(calc.ayanamsa.label()), dms(c.ayanamsa))),
        (s(tr("Birth star – pada")), format!("{} – {}", i18n::nak(nak), pada_of(moon))),
        (s(tr("Birth rasi – lord")), format!("{} – {}", i18n::sign(msign), i18n::planet(SIGN_LORD[msign]))),
        (s(tr("Lagna – lord")), format!("{} – {}", i18n::sign(asc_s), i18n::planet(SIGN_LORD[asc_s]))),
        (s(tr("Tithi")), format!("{}, {}", i18n::tithi(p.tithi), i18n::paksha(p.shukla))),
        (s(tr("Karana")), s(i18n::karana(p.karana))),
        (s(tr("Nithya yoga")), s(i18n::nithya_yoga(p.yoga))),
        (s(tr("Sunrise")), fmt_time(rise, tz)),
        (s(tr("Sunset")), fmt_time(set, tz)),
        (s(tr("Day length (dinamana)")), day_len),
        (s(tr("Birth after sunrise")), after_rise),
        (s(tr("Local mean time correction")), lmt),
        // Cross-reference the other reckoning so both are available without ambiguity.
        if inp.weekday_civil {
            (s(tr("Astrological weekday (vara, from sunrise)")), s(i18n::weekday(p.weekday)))
        } else {
            (s(tr("Civil weekday (calendar)")), s(i18n::weekday(civil_wd)))
        },
        (s(tr("Kali day count (Kalidina sankhya)")), kalidina(c.jd_ut, tz).to_string()),
        (s(tr("Dasa system")), s(tr("Vimshottari, year = 365.25 days"))),
        (s(tr("Star lord")), s(i18n::planet(DASHA[nak % 9].0))),
        (s(tr("Gana, yoni, nadi")), format!("{}, {}, {}", tr(GANA_NAMES[GANA[nak]]), tr(YONI_NAMES[yoni_animal(nak)]), tr(NADI_NAMES[nadi(nak)]))),
        (s(tr("Sun's rasi – star")), format!("{} – {}", i18n::sign(sun.sign()), i18n::nak(nakshatra_of(sun.lon)))),
        (s(tr("Western (tropical) Sun sign")), s(i18n::sign(western))),
        (s(tr("Yogi point – star – planet")), format!("{} – {} – {}", dms(yogi), i18n::nak(nakshatra_of(yogi)), i18n::planet(DASHA[nakshatra_of(yogi) % 9].0))),
        (s(tr("Duplicate yogi")), s(i18n::planet(SIGN_LORD[sign_of(yogi)]))),
        // The avayogi star is the 6th star from the yogi star (reference sample: Punarvasu -> Uthram).
        (s(tr("Avayogi star – planet")), format!("{} – {}", i18n::nak(avayogi_star(yogi)), i18n::planet(DASHA[avayogi_star(yogi) % 9].0))),
        (s(tr("Atma karaka – Karakamsa")), format!("{} – {}", i18n::planet(ak), i18n::sign(varga_sign(c.get(ak).lon, 9)))),
        (s(tr("Amatya karaka")), s(i18n::planet(karakas[1]))),
        (s(tr("Lagna pada (Arudha)")), s(i18n::sign(arudha(c, 1)))),
        (s(tr("Dhana pada (2nd Arudha)")), s(i18n::sign(arudha(c, 2)))),
    ]);

    // 2. Panchanga readings
    h.h2(2, tr("Panchanga readings"));
    h.p(&trf(
        "Born {0}, {1}, in {2} star pada {3}, {4} {5}, {6} karana and {7} yoga, with the Moon in {8} rasi and {9} navamsa, in {10} lagna.",
        &[
            format!("{}{}", i18n::weekday(disp_wd), wd_note),
            format!("{:02}-{}-{}", b.day, i18n::month_short(b.month), b.year),
            s(i18n::nak(nak)),
            pada_of(moon).to_string(),
            s(i18n::paksha(p.shukla)),
            s(i18n::tithi(p.tithi)),
            s(i18n::karana(p.karana)),
            s(i18n::nithya_yoga(p.yoga)),
            s(i18n::sign(msign)),
            s(i18n::sign(varga_sign(moon, 9))),
            s(i18n::sign(asc_s)),
        ],
    ));
    for (label, value, text) in [
        (tr("Weekday"), i18n::weekday(disp_wd), texts::weekday(disp_wd)),
        (tr("Birth star"), i18n::nak(nak), texts::nakshatra(nak)),
        (tr("Tithi"), i18n::tithi(p.tithi), texts::tithi(p.tithi)),
        (tr("Karana"), i18n::karana(p.karana), texts::karana(p.karana)),
        (tr("Nithya yoga"), i18n::nithya_yoga(p.yoga), texts::nithya_yoga(p.yoga)),
        (tr("Lagna"), i18n::sign(asc_s), texts::lagna(asc_s)),
    ] {
        h.h3(&format!("{label} : {value}"));
        h.p(text);
    }

    // 3. Planet positions
    h.h2(3, tr("Planet positions"));
    h.h3(tr("Nirayana (sidereal) longitudes"));
    let mut rows = vec![vec![
        s(tr("Ascendant")),
        dms(c.asc),
        s(i18n::sign(asc_s)),
        in_sign(c.asc),
        s(i18n::nak(nakshatra_of(c.asc))),
        pada_of(c.asc).to_string(),
        s(i18n::planet(DASHA[nakshatra_of(c.asc) % 9].0)),
        "1".into(),
        String::new(),
    ]];
    for pl in &c.planets {
        rows.push(vec![
            planet_label(pl),
            dms(pl.lon),
            s(i18n::sign(pl.sign())),
            in_sign(pl.lon),
            s(i18n::nak(nakshatra_of(pl.lon))),
            pada_of(pl.lon).to_string(),
            s(i18n::planet(DASHA[nakshatra_of(pl.lon) % 9].0)),
            c.house_of(pl.graha).to_string(),
            s(tr(dignity(pl.graha, pl.sign()))),
        ]);
    }
    h.table(
        &[tr("Planet"), tr("Longitude"), tr("Sign"), tr("Degree"), tr("Star (nakshatra)"), tr("Pada"), tr("Star lord"), tr("House"), tr("Strength")].map(s),
        &rows,
    );
    h.h3(tr("Sayana (tropical, western) longitudes"));
    let mut rows = vec![vec![s(tr("Ascendant")), dms(c.asc + c.ayanamsa), s(i18n::sign(sign_of(c.asc + c.ayanamsa))), in_sign(c.asc + c.ayanamsa)]];
    for pl in &c.planets {
        let t = pl.lon + c.ayanamsa;
        rows.push(vec![planet_label(pl), dms(t), s(i18n::sign(sign_of(t))), in_sign(t)]);
    }
    h.table(&[tr("Planet"), tr("Longitude"), tr("Western sign"), tr("Degree")].map(s), &rows);
    h.h3(tr("Star lord / sub-lord / sub-sub-lord (KP)"));
    let kp_row = |name: String, lon: f64| {
        let (a, b2, c2) = kp_lords(lon);
        vec![name, dms(lon), s(i18n::nak(nakshatra_of(lon))), s(i18n::planet(a)), s(i18n::planet(b2)), s(i18n::planet(c2))]
    };
    let mut rows = vec![kp_row(s(tr("Ascendant")), c.asc)];
    for pl in &c.planets {
        rows.push(kp_row(s(i18n::planet(pl.graha)), pl.lon));
    }
    h.table(&[tr("Planet"), tr("Longitude"), tr("Star"), tr("Star lord"), tr("Sub-lord"), tr("Sub-sub-lord")].map(s), &rows);

    // 4. Charts
    h.h2(4, tr("Charts"));
    let bh = sripati_bhavas(c.asc, c.mc);
    let mut bhava_cells: [Vec<(String, bool)>; 12] = Default::default();
    bhava_cells[asc_s].push((tr("Asc").to_string(), true));
    for pl in &c.planets {
        let n = bhava_of(&bh, pl.lon);
        bhava_cells[(asc_s + n - 1) % 12].push((i18n::planet_short(pl.graha).to_string(), false));
    }
    h.raw(r#"<div class="charts">"#);
    for (title, cells, asc) in [
        (tr("Rasi chart (D1)"), varga_cells(c, 1), asc_s),
        (tr("Navamsa chart (D9)"), varga_cells(c, 9), varga_sign(c.asc, 9)),
        (tr("Bhava chart"), bhava_cells, asc_s),
    ] {
        let _ = write!(h.0, r#"<figure>{}<figcaption>{}</figcaption></figure>"#, svg_chart(inp.style, title, &cells, asc, 300.0), esc(title));
    }
    h.raw("</div>");

    // 5. House table
    h.h2(5, tr("House table"));
    h.p(tr("Planets in each house according to the house cusps (Sripati)."));
    let rows: Vec<Vec<String>> = (0..12)
        .map(|i| {
            let names: Vec<&str> = c.planets.iter().filter(|pl| bhava_of(&bh, pl.lon) == i + 1).map(|pl| i18n::planet(pl.graha)).collect();
            vec![
                (i + 1).to_string(),
                format!("{} {}", i18n::sign_short(sign_of(bh[i].1)), in_sign(bh[i].1)),
                format!("{} {}", i18n::sign_short(sign_of(bh[i].0)), in_sign(bh[i].0)),
                format!("{} {}", i18n::sign_short(sign_of(bh[(i + 1) % 12].1)), in_sign(bh[(i + 1) % 12].1)),
                s(i18n::planet(SIGN_LORD[sign_of(bh[i].0)])),
                names.join(", "),
            ]
        })
        .collect();
    h.table(&[tr("House"), tr("Begins (sandhi)"), tr("Middle (madhya)"), tr("Ends"), tr("Lord"), tr("Planets in house")].map(s), &rows);

    // 6. Shodasavarga
    h.h2(6, tr("Shodasavarga (divisional charts)"));
    let mut head = vec![s(tr("Planet"))];
    head.extend(VARGAS.iter().map(|(d, _)| format!("D{d}")));
    let mut rows = Vec::new();
    let mut row_for = |name: String, lon: f64| {
        let mut r = vec![name];
        r.extend(VARGAS.iter().map(|(d, _)| s(i18n::sign_short(varga_sign(lon, *d)))));
        rows.push(r);
    };
    row_for(s(tr("Ascendant")), c.asc);
    for pl in &c.planets {
        row_for(s(i18n::planet(pl.graha)), pl.lon);
    }
    h.table(&head, &rows);
    h.raw(r#"<div class="charts small">"#);
    for (d, _) in VARGAS.iter().filter(|v| v.0 != 1) {
        let label = varga_label(*d);
        let _ = write!(h.0, r#"<figure>{}<figcaption>{}</figcaption></figure>"#, svg_chart(inp.style, &format!("D{d}"), &varga_cells(c, *d), varga_sign(c.asc, *d), 220.0), esc(&label));
    }
    h.raw("</div>");

    // 7. Ashtakavarga
    h.h2a(7, tr("Ashtakavarga"), ai_paras(ai, "ashtakavarga").is_some());
    let bav = ashtakavarga(c);
    let sav = sarvashtakavarga(&bav);
    h.h3(tr("Bhinnashtakavarga (benefic points of each planet by sign)"));
    let mut head = vec![s(tr("Planet"))];
    head.extend((0..12).map(|i| s(i18n::sign_short(i))));
    head.push(s(tr("Total")));
    let mut rows: Vec<Vec<String>> = (0..7)
        .map(|i| {
            let mut r = vec![s(i18n::planet(GRAHAS[i]))];
            r.extend(bav[i].iter().map(|v| v.to_string()));
            r.push(bav[i].iter().map(|&v| v as u32).sum::<u32>().to_string());
            r
        })
        .collect();
    let mut total = vec![s(tr("Total"))];
    total.extend(sav.iter().map(|v| v.to_string()));
    total.push(sav.iter().sum::<u16>().to_string());
    rows.push(total);
    h.table(&head, &rows);
    h.h3(tr("Sarvashtakavarga (total of all planets)"));
    let sav_cells: [Vec<(String, bool)>; 12] = std::array::from_fn(|i| vec![(sav[i].to_string(), sav[i] >= 28)]);
    let _ = write!(h.0, r#"<div class="charts"><figure>{}</figure></div>"#, svg_chart(inp.style, "SAV", &sav_cells, asc_s, 260.0));
    h.p(tr("Signs with 28 or more points are strong; under 25 are weak."));
    // AI narration from clean facts, not the raw number grid.
    let best = (0..12).max_by_key(|&i| sav[i]).unwrap_or(0);
    let worst = (0..12).min_by_key(|&i| sav[i]).unwrap_or(0);
    let strong: Vec<String> = (0..12).filter(|&i| sav[i] >= 28).map(|i| format!("{} ({})", i18n::sign(i), sav[i])).collect();
    let weak: Vec<String> = (0..12).filter(|&i| sav[i] < 25).map(|i| format!("{} ({})", i18n::sign(i), sav[i])).collect();
    let av_facts = trf(
        "Sarvashtakavarga measures the overall benefic strength of each sign (the twelve signs together total 337 points; a sign with 28 or more is strong, under 25 is weak). The strongest sign is {0} with {1} points; the weakest is {2} with {3} points. Strong signs: {4}. Weak signs: {5}. The ascendant is {6}. Houses and planets falling in strong signs tend to give their results more easily; those in weak signs ask for more patience and effort.",
        &[s(i18n::sign(best)), sav[best].to_string(), s(i18n::sign(worst)), sav[worst].to_string(),
          if strong.is_empty() { s(tr("none")) } else { strong.join(", ") },
          if weak.is_empty() { s(tr("none")) } else { weak.join(", ") },
          s(i18n::sign(asc_s))],
    );
    let avkey = "ashtakavarga".to_string();
    tasks.push(AiTask { key: avkey.clone(), title: s(tr("Ashtakavarga")), facts: av_facts });
    append_ai(&mut h, ai, &avkey, inp.stale_keys.iter().any(|k| k == &avkey));

    // 8. Planetary relationships
    h.h2(8, tr("Planetary relationships"));
    h.h3(tr("House lords"));
    let rows: Vec<Vec<String>> = (1..=12)
        .map(|hn| {
            let sg = (asc_s + hn - 1) % 12;
            let lord = SIGN_LORD[sg];
            vec![hn.to_string(), s(i18n::sign(sg)), s(i18n::planet(lord)), c.house_of(lord).to_string(), s(nature_word(lordship_nature(asc_s, lord).1))]
        })
        .collect();
    h.table(&[tr("House"), tr("Sign"), tr("Lord"), tr("Lord placed in house"), tr("Nature")].map(s), &rows);
    h.p(tr("Rows: the planet; columns: how it regards the other planet."));
    let seven = &GRAHAS[..7];
    let mut head = vec![String::new()];
    head.extend(seven.iter().map(|&g| s(i18n::planet_short(g))));
    for (title, f) in [
        (tr("Permanent (natural) friendship"), 0),
        (tr("Temporary friendship"), 1),
        (tr("Five-fold friendship"), 2),
    ] {
        h.h3(title);
        let rows: Vec<Vec<String>> = seven
            .iter()
            .map(|&a| {
                let mut r = vec![s(i18n::planet(a))];
                for &bb in seven {
                    r.push(if a == bb {
                        "—".into()
                    } else {
                        s(tr(match f {
                            0 => match relation(a, bb) {
                                Rel::Friend => "Friend",
                                Rel::Neutral => "Neutral",
                                Rel::Enemy => "Enemy",
                            },
                            1 => {
                                if temporary_friend(c, a, bb) { "Friend" } else { "Enemy" }
                            }
                            _ => fivefold(c, a, bb),
                        }))
                    });
                }
                r
            })
            .collect();
        h.table(&head, &rows);
    }

    benefic_section(&mut h, c);

    // 9. Vimshottari dasha
    h.h2(10, tr("Vimshottari dasha"));
    let (lord, bal) = dasha_balance(moon);
    h.p(&trf("Balance of dasha at birth: {0} {1}", &[s(i18n::planet(lord)), years_to_ymd(bal)]));
    h.h3(tr("Mahadashas"));
    let rows: Vec<Vec<String>> = inp.dasha.iter().map(|md| vec![s(i18n::planet(md.lord)), fmt_date(md.start.max(c.jd_ut), tz), fmt_date(md.end, tz)]).collect();
    h.table(&[tr("Dasha"), tr("Starts"), tr("Ends on")].map(s), &rows);
    let head = [tr("Bhukti"), tr("Starts"), tr("Ends on")].map(s);
    h.raw(r#"<div class="grid3">"#);
    for md in inp.dasha {
        if md.end < c.jd_ut {
            continue;
        }
        h.raw("<div>");
        h.h3(&trf("{0} dasha – bhuktis", &[s(i18n::planet(md.lord))]));
        let rows: Vec<Vec<String>> = md
            .sub
            .iter()
            .filter(|bk| bk.end >= c.jd_ut)
            .map(|bk| vec![format!("{}–{}", i18n::planet_short(md.lord), i18n::planet_short(bk.lord)), fmt_date(bk.start.max(c.jd_ut), tz), fmt_date(bk.end, tz)])
            .collect();
        h.table(&head, &rows);
        h.raw("</div>");
    }
    h.raw("</div>");
    // Antaras for the running mahadasha (or the first one when the person is not yet born).
    let running = inp.dasha.iter().find(|md| md.start <= inp.now && inp.now < md.end).or(inp.dasha.first());
    if let Some(md) = running {
        let head = [tr("Antara"), tr("Starts"), tr("Ends on")].map(s);
        h.raw(r#"<div class="grid3">"#);
        for bk in &md.sub {
            if bk.end < c.jd_ut {
                continue;
            }
            h.raw("<div>");
            h.h3(&trf("{0} dasha, {1} bhukti – antaras", &[s(i18n::planet(md.lord)), s(i18n::planet(bk.lord))]));
            let rows: Vec<Vec<String>> = bk
                .sub
                .iter()
                .filter(|an| an.end >= c.jd_ut)
                .map(|an| vec![format!("{}–{}–{}", i18n::planet_short(md.lord), i18n::planet_short(bk.lord), i18n::planet_short(an.lord)), fmt_date(an.start.max(c.jd_ut), tz), fmt_date(an.end, tz)])
                .collect();
            h.table(&head, &rows);
            h.raw("</div>");
        }
        h.raw("</div>");
    }

    // 10. Yogas and doshas
    h.h2(11, tr("Yogas and doshas"));
    let yogas = find_yogas(c);
    if yogas.is_empty() {
        h.p(tr("None of the checked yogas are present."));
    }
    for y in yogas {
        h.h3(tr(y.name));
        h.p(&format!("{}: {}", readings::t(R::Logic), y.why));
        let result = readings::yoga_result(yoga_group(y.name));
        if !result.is_empty() {
            h.p(result);
        }
    }
    for (title, lines) in dosha_sections(c) {
        h.h3(title);
        for l in lines {
            h.p(&l);
        }
    }

    // 12. House (bhava) readings
    h.h2(12, tr("House readings"));
    for hn in 1..=12 {
        let sg = (asc_s + hn - 1) % 12;
        let lord = SIGN_LORD[sg];
        let placed = c.house_of(lord);
        let title = format!("{} — {} ({})", trf("House {0}", &[hn.to_string()]), readings::house_title(hn), i18n::sign(sg));
        let key = format!("house-{hn}");
        h.h3a(&title, ai_paras(ai, &key).is_some());
        let start = h.0.len();
        // Sanskrit name, significators and kind of house, from the knowledge base's glossary.
        if let Some(bi) = knowledge::bhava(hn) {
            let karakas: Vec<String> = bi.karakas.iter().filter_map(|k| PLANET_EN.iter().position(|p| p == k)).map(|i| s(i18n::planet(GRAHAS[i]))).collect();
            let classes: Vec<String> = bi.classes.iter().map(|c| s(house_class(c))).collect();
            let mut line = format!("{} ({})", bi.sanskrit, bi.devanagari);
            if !karakas.is_empty() {
                line.push_str(" · ");
                line.push_str(&trf("Significator (karaka): {0}", &[karakas.join(", ")]));
            }
            if !classes.is_empty() {
                line.push_str(" · ");
                line.push_str(&classes.join(", "));
            }
            let _ = write!(h.0, r#"<p class="sanskrit">{}</p>"#, esc(&line));
        }
        // The AI narrative, when present, replaces the rule prose from here on (the header above stays).
        let prose_start = h.0.len();
        h.p(&i18n::fill(texts::tpl(Tpl::HouseIntro), &[hn.to_string(), s(texts::house_meaning(hn))]));
        if hn == 1 {
            h.p(texts::lagna(asc_s));
            let deg = norm360(c.asc) % 30.0;
            let (k, rel) = if deg < 10.0 { (R::Drekkana1, 0) } else if deg < 20.0 { (R::Drekkana2, 4) } else { (R::Drekkana3, 8) };
            h.p(&i18n::fill(readings::t(k), &[s(i18n::sign((asc_s + rel) % 12))]));
        }
        let tpl = match placed {
            1 | 4 | 5 | 7 | 9 | 10 | 11 => Tpl::LordGood,
            6 | 8 | 12 => Tpl::LordHard,
            _ => Tpl::LordMid,
        };
        let mut lord_line = i18n::fill(texts::tpl(tpl), &[hn.to_string(), s(i18n::planet(lord)), placed.to_string()]);
        if placed != hn {
            lord_line.push(' ');
            lord_line.push_str(&i18n::fill(readings::t(R::LordLink), &[s(texts::house_meaning(hn)), s(texts::house_meaning(placed))]));
        }
        h.p(&lord_line);
        // The classical result of this house's lord sitting where it does (knowledge base, with its source).
        if let Some(rule) = knowledge::lord_in_house(hn, placed) {
            h.p(&rule.text);
            cite(&mut h, &rule);
        }
        let occupants: Vec<&Position> = c.planets.iter().filter(|pl| c.house_of(pl.graha) == hn).collect();
        if occupants.is_empty() {
            h.p(texts::tpl(Tpl::NoOccupants));
        }
        for pl in occupants {
            h.p(readings::planet_in_house(pl.graha as usize, hn));
        }
        // Special aspects of Jupiter (5, 7, 9), Saturn (3, 7, 10) and Mars (4, 7, 8), counted by sign.
        for (g, houses, key) in [(Graha::Jupiter, [5, 7, 9], R::JupAspect), (Graha::Saturn, [3, 7, 10], R::SatAspect), (Graha::Mars, [4, 7, 8], R::MarsAspect)] {
            let from_planet = (sg + 12 - c.get(g).sign()) % 12 + 1;
            if houses.contains(&from_planet) {
                h.p(readings::t(key));
            }
        }
        tasks.push(AiTask { key: key.clone(), title: title.clone(), facts: plain(&h.0[start..]) });
        replace_ai(&mut h, ai, &key, prose_start, inp.stale_keys.iter().any(|k| k == &key));
    }

    // 13. Dasha readings: the running mahadasha and those still to come, with every bhukti
    h.h2(13, tr("Dasha readings"));
    let start_from = inp.now.max(c.jd_ut);
    let yogas = find_yogas(c);
    for md in inp.dasha.iter().filter(|md| md.end > start_from).take(5) {
        let g = md.lord;
        let pl = c.get(g);
        let house = c.house_of(g);
        let _ = write!(h.0, r#"<p class="period">{}</p>"#, esc(&trf("Starting from {0}", &[fmt_date(md.start.max(c.jd_ut), tz)])));
        // English headings use the traditional names, e.g. "Chandra Dasa (Moon)"; other languages name the planet in their own script.
        let title = if i18n::lang() == i18n::Lang::En {
            match g {
                Graha::Rahu | Graha::Ketu => format!("{} Dasa", PLANET_EN[g as usize]),
                _ => format!("{} Dasa ({})", SANSKRIT[g as usize], PLANET_EN[g as usize]),
            }
        } else {
            trf("{0} Dasa", &[s(i18n::planet(g))])
        };
        let key = format!("dasha-{}", PLANET_EN[g as usize].to_lowercase());
        h.h3a(&title, ai_paras(ai, &key).is_some());
        let start = h.0.len();
        h.p(&trf("{0} mahadasha   {1}  to  {2}", &[s(i18n::planet(g)), fmt_date(md.start.max(c.jd_ut), tz), fmt_date(md.end, tz)]));
        // The AI narrative, when present, replaces the rule prose from here (the dated period line stays).
        let prose_start = h.0.len();
        h.p(texts::dasha(g as usize));
        // Knowledge-base reading for this dasa: the general result, then the strong or weak one.
        if let Some((general, strong, weak)) = knowledge::dasha(PLANET_EN[g as usize]) {
            h.p(&general.text);
            match dasa_strength(c, g) {
                1 => h.p(&strong),
                -1 => h.p(&weak),
                _ => {}
            }
            cite(&mut h, &general);
        }
        for y in yogas.iter().filter(|y| y.planets.contains(&g)) {
            let mut line = trf("This period also brings out the {0} yoga in your chart.", &[s(tr(y.name))]);
            let result = readings::yoga_result(yoga_group(y.name));
            if !result.is_empty() {
                line.push(' ');
                line.push_str(result);
            }
            h.p(&line);
        }
        // Company in the same house: unfriendly planets hold back the dasa lord (the Moon also by malefics).
        let mates: Vec<Graha> = c.planets.iter().map(|p| p.graha).filter(|&o| o != g && c.house_of(o) == house).collect();
        let enemies: Vec<String> = mates.iter().filter(|&&o| relation(g, o) == Rel::Enemy).map(|&o| s(i18n::planet(o))).collect();
        if !enemies.is_empty() && !matches!(g, Graha::Rahu | Graha::Ketu) {
            h.p(&trf("{0} shares a house with {1}, which it does not get along with, so part of the good of this period can be held back by friction.", &[s(i18n::planet(g)), enemies.join(", ")]));
        }
        if g == Graha::Moon {
            let malefics: Vec<String> = mates.iter().filter(|&&o| matches!(o, Graha::Sun | Graha::Mars | Graha::Saturn | Graha::Rahu | Graha::Ketu)).map(|&o| s(i18n::planet(o))).collect();
            if !malefics.is_empty() {
                h.p(&trf("The Moon is joined by {0}; give your peace of mind and emotional balance extra care in this period.", &[malefics.join(", ")]));
            }
        }
        let dig = dignity(g, pl.sign());
        let dig_text = if dig.is_empty() { "—".to_string() } else { tr(dig).to_string() };
        let strength_key = match dasa_strength(c, g) {
            1 => R::DashaStrong,
            -1 => R::DashaWeak,
            _ => R::DashaAverage,
        };
        h.p(&i18n::fill(readings::t(strength_key), &[s(i18n::planet(g)), format!("{}, {}", i18n::sign(pl.sign()), dig_text)]));
        let (owned, nature) = lordship_nature(asc_s, g);
        if !owned.is_empty() {
            let houses: Vec<String> = owned.iter().map(|x| x.to_string()).collect();
            h.p(&i18n::fill(readings::t(R::FuncNature), &[s(i18n::planet(g)), s(nature_word(nature)), houses.join(", ")]));
        }
        h.p(&i18n::fill(texts::tpl(Tpl::DashaPlace), &[s(i18n::planet(g)), s(i18n::sign(pl.sign())), house.to_string(), dig_text, s(texts::house_meaning(house))]));
        // Keep the dated sub-period (bhukti) timeline even when AI replaces the general reading above.
        let bhukti_start = h.0.len();
        for bk in md.sub.iter().filter(|bk| bk.end > start_from) {
            let b_g = bk.lord;
            let _ = write!(h.0, r#"<p class="period">{} – {}</p>"#, esc(&fmt_date(bk.start.max(c.jd_ut), tz)), esc(&fmt_date(bk.end, tz)));
            let mut text = i18n::fill(readings::t(R::BhuktiIntro), &[s(i18n::planet(b_g)), s(i18n::planet(g)), s(readings::bhukti_topics(b_g as usize))]);
            if b_g != g {
                let rel = match (relation(g, b_g), relation(b_g, g)) {
                    (Rel::Friend, Rel::Friend) | (Rel::Friend, Rel::Neutral) | (Rel::Neutral, Rel::Friend) => R::RelFriend,
                    (Rel::Enemy, _) | (_, Rel::Enemy) => R::RelEnemy,
                    _ => R::RelNeutral,
                };
                text.push(' ');
                text.push_str(&i18n::fill(readings::t(rel), &[s(i18n::planet(g)), s(i18n::planet(b_g))]));
            }
            let bh = c.house_of(b_g);
            text.push(' ');
            text.push_str(&i18n::fill(readings::t(R::BhuktiPlace), &[s(i18n::planet(b_g)), bh.to_string(), s(texts::house_meaning(bh))]));
            // Life stage at the end of the sub-period decides where its results show (the reference age bands).
            text.push(' ');
            text.push_str(tr(LIFE_STAGE_TEXT[life_stage((bk.end - c.jd_ut) / 365.25)]));
            h.p(&text);
        }
        tasks.push(AiTask { key: key.clone(), title: format!("{} ({} – {})", title, fmt_date(md.start.max(c.jd_ut), tz), fmt_date(md.end, tz)), facts: plain(&h.0[start..]) });
        replace_ai_keep_tail(&mut h, ai, &key, prose_start, bhukti_start, inp.stale_keys.iter().any(|k| k == &key));
    }

    // 14. Remedies
    let rkey = "remedies".to_string();
    h.h2a(14, tr("Remedies"), ai_paras(ai, &rkey).is_some());
    let rem_start = h.0.len();
    remedies_section(&mut h, inp);
    tasks.push(AiTask { key: rkey.clone(), title: s(tr("Remedies")), facts: plain(&h.0[rem_start..]) });
    append_ai(&mut h, ai, &rkey, inp.stale_keys.iter().any(|k| k == &rkey));

    // 15. Transit forecast: the Sun for the next year, Jupiter and Saturn for the next 12 years
    h.h2a(15, tr("Transit forecast"), ai_paras(ai, "transit").is_some());
    let transit_start = h.0.len();
    let from = inp.now.max(c.jd_ut);
    // Transits are counted from the birth Moon sign, and the whole section is read
    // against the person's life stage on the day the report is made (the reference transit bands by age).
    h.p(&trf("Transits are counted from your Moon sign, {0}.", &[s(i18n::sign(msign))]));
    h.p(tr(LIFE_STAGE_TEXT[life_stage((inp.now - c.jd_ut).max(0.0) / 365.25)]));
    for (g, title, years) in [(Graha::Sun, tr("Sun transits"), 1.0), (Graha::Jupiter, tr("Jupiter transits"), 12.0), (Graha::Saturn, tr("Saturn transits"), 12.0)] {
        h.h3(title);
        let rows: Vec<Vec<String>> = sign_periods(g, from, from + years * 365.25, calc)
            .into_iter()
            .filter(|(a, b2, _)| b2 - a > 2.0)
            .map(|(a, b2, sg)| {
                let house = (sg + 12 - msign) % 12 + 1;
                let reading = if g == Graha::Sun {
                    // Classical gochara: the Sun does well in the 3rd, 6th, 10th and 11th from the Moon.
                    if matches!(house, 3 | 6 | 10 | 11) {
                        trf("Sun in house {0} from the Moon: a good month for energy, recognition and getting ahead of rivals.", &[house.to_string()])
                    } else {
                        trf("Sun in house {0} from the Moon: a month to go steadily; look after your health and avoid clashes with people in authority.", &[house.to_string()])
                    }
                } else if g == Graha::Jupiter {
                    let t = if matches!(house, 2 | 5 | 7 | 9 | 11) { Tpl::JupGood } else { Tpl::JupHard };
                    i18n::fill(texts::tpl(t), &[house.to_string()])
                } else {
                    let t = if matches!(house, 3 | 6 | 11) { Tpl::SatGood } else { Tpl::SatHard };
                    let mut r = i18n::fill(texts::tpl(t), &[house.to_string()]);
                    if matches!(house, 12 | 1 | 2) {
                        r.push(' ');
                        r.push_str(texts::tpl(Tpl::SadeSati));
                    } else if house == 8 {
                        r.push(' ');
                        r.push_str(texts::tpl(Tpl::Ashtama));
                    }
                    r
                };
                vec![format!("{} – {}", fmt_date(a, tz), fmt_date(b2, tz)), s(i18n::sign(sg)), house.to_string(), reading]
            })
            .collect();
        h.table(&[tr("Period"), tr("Sign"), tr("From Moon"), tr("Reading")].map(s), &rows);
    }
    let tkey = "transit".to_string();
    tasks.push(AiTask { key: tkey.clone(), title: s(tr("Transit forecast")), facts: plain(&h.0[transit_start..]) });
    append_ai(&mut h, ai, &tkey, inp.stale_keys.iter().any(|k| k == &tkey));

    h.h3(tr("Note"));
    h.p(tr("This report is calculated from the birth details given, using standard Vedic astrology rules. Readings describe tendencies, not certainties; please use them as general guidance."));
    let _ = norm360(0.0);
    h.raw("</body></html>");
    (h.0, tasks)
}

const CSS: &str = r#"
@page { size: A4; margin: 14mm; }
body { font-family: "Noto Sans", "Segoe UI", "Helvetica Neue", Arial, "Tamil Sangam MN", "Noto Sans Tamil", "Latha", "Nirmala UI", "Kohinoor Devanagari", "Devanagari Sangam MN", "Noto Sans Devanagari", "Mangal", sans-serif;
       color: #222; background: #fffdf7; max-width: 960px; margin: 0 auto; padding: 18px; line-height: 1.45; font-size: 14px; }
h1 { color: #8b1e1e; font-size: 30px; margin: 40px 0 6px; text-align: center; }
h2 { color: #fff; background: #8b1e1e; padding: 6px 10px; border-radius: 4px; margin-top: 34px; page-break-before: always; font-size: 19px; }
h3 { color: #8b1e1e; margin: 18px 0 6px; font-size: 16px; }
p { margin: 6px 0; text-align: justify; }
.cover { text-align: center; }
.cover p { text-align: center; color: #6b5a40; }
.toc { display: inline-block; text-align: left; margin-top: 24px; border: 1px solid #d9c8a5; border-radius: 6px; padding: 8px 24px; background: #fbf5e6; }
.toc a { color: #1f2a5a; text-decoration: none; }
table { border-collapse: collapse; width: 100%; margin: 6px 0 12px; font-size: 13px; }
th, td { border: 1px solid #d9c8a5; padding: 4px 6px; text-align: left; vertical-align: top; }
thead th { background: #f3e6c8; }
tbody tr:nth-child(even) td { background: #fbf6ea; }
table.kv th { width: 38%; background: #f7eedb; font-weight: 600; }
.charts { display: flex; flex-wrap: wrap; gap: 16px; justify-content: center; }
.charts figure { margin: 4px; text-align: center; }
.charts figcaption { color: #8b1e1e; font-weight: 600; margin-top: 4px; }
.charts.small figure svg { width: 200px; height: 200px; }
p.period { font-weight: 600; color: #8b1e1e; margin: 10px 0 2px; }
p.sanskrit { color: #8a4b12; font-size: 12.5px; margin: 0 0 6px; }
p.cite { font-size: 11px; color: #7a6a55; font-style: italic; margin: 0 0 8px; }
div.ai { border-left: 3px solid #c97b2e; background: #fff8ec; padding: 4px 12px; margin: 8px 0 12px; }
p.ai-label { font-weight: 600; color: #8a4b12; margin: 4px 0; }
.grid3 { display: grid; grid-template-columns: repeat(auto-fill, minmax(280px, 1fr)); gap: 0 16px; }
svg.chart text { font-family: inherit; }
div.stale { border-left: 3px solid #e6a23c; background: #fdf3e0; padding: 4px 12px; margin: 8px 0 12px; }
p.flag { font-weight: 600; color: #9a5b12; font-size: 12.5px; margin: 4px 0; }
button.rewrite { border: 1px solid #7c2b32; background: #7c2b32; color: #fff; border-radius: 7px; padding: 6px 12px; font-size: 13px; cursor: pointer; margin: 6px 0 2px; }
.ai-off button.rewrite { display: none; }
@media print { body { background: #fff; padding: 0; } h2 { -webkit-print-color-adjust: exact; print-color-adjust: exact; } button.rewrite { display: none; } }
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stale_keys_flag_time_based_ai_only() {
        assert!(is_time_based_key("transit") && is_time_based_key("dasha-jupiter"));
        assert!(!is_time_based_key("ashtakavarga") && !is_time_based_key("house-5"));
        let b = BirthData::default();
        let chart = Chart::compute(b.jd_ut(), b.lat, b.lon, b.tz, Calc::default());
        let pancha = panchanga(&chart);
        let dasha = vimshottari(chart.moon(), chart.jd_ut, 3);
        let stale = ["transit".to_string()];
        let inp = Input { birth: &b, chart: &chart, pancha: &pancha, dasha: &dasha, calc: Calc::default(), style: Style::South, now: chart.jd_ut, weekday_civil: false, stale_keys: &stale };
        let mut ai = HashMap::new();
        ai.insert("transit".to_string(), (String::new(), "<script>x</script>old transit".to_string()));
        let html = build_with(&inp, &ai).0;
        assert!(html.contains(r#"class="stale""#) && html.contains(r#"data-key="transit""#));
        assert!(!html.contains("<script>x</script>"), "saved AI prose is escaped");
    }
}
