//! Vedic (sidereal) astrology rules built on top of `astro`.
//! Every table here is a published classical convention; see MECHANICS.md.

use crate::astro::{self, norm360};
use crate::i18n::{self, tr, trf};
use serde::{Deserialize, Serialize};


/// The nine grahas plus the ascendant (index 9).
#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum Graha {
    Sun = 0,
    Moon,
    Mars,
    Mercury,
    Jupiter,
    Venus,
    Saturn,
    Rahu,
    Ketu,
}

pub const GRAHAS: [Graha; 9] = [
    Graha::Sun, Graha::Moon, Graha::Mars, Graha::Mercury, Graha::Jupiter, Graha::Venus,
    Graha::Saturn, Graha::Rahu, Graha::Ketu,
];

/// Lord of each sign.
pub const SIGN_LORD: [Graha; 12] = [
    Graha::Mars, Graha::Venus, Graha::Mercury, Graha::Moon, Graha::Sun, Graha::Mercury,
    Graha::Venus, Graha::Mars, Graha::Jupiter, Graha::Saturn, Graha::Saturn, Graha::Jupiter,
];

/// Vimshottari order with years; nakshatra n is ruled by DASHA[n % 9].
pub const DASHA: [(Graha, f64); 9] = [
    (Graha::Ketu, 7.0), (Graha::Venus, 20.0), (Graha::Sun, 6.0), (Graha::Moon, 10.0),
    (Graha::Mars, 7.0), (Graha::Rahu, 18.0), (Graha::Jupiter, 16.0), (Graha::Saturn, 19.0),
    (Graha::Mercury, 17.0),
];
pub const DASHA_YEAR_DAYS: f64 = 365.25;

pub const NAK_SPAN: f64 = 360.0 / 27.0;

#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Debug)]
pub enum NodeKind {
    Mean,
    True,
}

/// The ayanamsa choices offered ("ChithraPaksha(Lahiri)", "Raman", "KrishnaMurthy",
/// "User Defined Ayanamsa"). Reference values come from the standard ayanamsa table
/// (see MECHANICS.md).
#[derive(Clone, Copy, PartialEq, Serialize, Deserialize, Debug)]
pub enum Ayanamsa {
    Lahiri,
    Raman,
    Krishnamurti,
    /// Zero at `base_year`, growing by `arcsec_per_year` (reference layout: name, base year, precision).
    User { base_year: f64, arcsec_per_year: f64 },
}

impl Ayanamsa {
    pub fn value(self, jde: f64) -> f64 {
        match self {
            Ayanamsa::Lahiri => astro::lahiri_ayanamsa(jde),
            Ayanamsa::Raman => astro::ayanamsa_from(jde, 2415020.0, 21.01444),
            Ayanamsa::Krishnamurti => astro::ayanamsa_from(jde, 2415020.0, 22.363889),
            Ayanamsa::User { base_year, arcsec_per_year } => {
                let year = 2000.0 + (jde - 2451545.0) / 365.25;
                (year - base_year) * arcsec_per_year / 3600.0
            }
        }
    }
    pub fn label(self) -> &'static str {
        match self {
            Ayanamsa::Lahiri => "Chitrapaksha (Lahiri)",
            Ayanamsa::Raman => "Raman",
            Ayanamsa::Krishnamurti => "Krishnamurti (KP)",
            Ayanamsa::User { .. } => "User defined",
        }
    }
}

/// Calculation settings that change positions. Planet positions come from the Swiss Ephemeris
/// (Moshier mode); these settings pick the lunar node and the ayanamsa.
#[derive(Clone, Copy, PartialEq, Serialize, Deserialize, Debug)]
pub struct Calc {
    pub node: NodeKind,
    pub ayanamsa: Ayanamsa,
}

impl Default for Calc {
    fn default() -> Self {
        Calc { node: NodeKind::Mean, ayanamsa: Ayanamsa::Lahiri }
    }
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct BirthData {
    pub name: String,
    pub gender_male: bool,
    pub year: i32,
    pub month: u32,
    pub day: u32,
    pub hour: u32,
    pub minute: u32,
    pub second: u32,
    /// Hours east of Greenwich (India = 5.5).
    pub tz: f64,
    pub place: String,
    pub lat: f64,
    pub lon: f64,
    /// IANA time zone of the place ("Asia/Kolkata"); when set, `tz` follows it for the birth date.
    #[serde(default)]
    pub tz_name: String,
}

impl Default for BirthData {
    fn default() -> Self {
        Self {
            name: "Sample Person".into(),
            gender_male: true,
            year: 1990,
            month: 8,
            day: 15,
            hour: 6,
            minute: 30,
            second: 0,
            tz: 5.5,
            place: "Chennai, India".into(),
            lat: 13.0827,
            lon: 80.2707,
            tz_name: "Asia/Kolkata".into(),
        }
    }
}

impl BirthData {
    pub fn jd_ut(&self) -> f64 {
        let h = self.hour as f64 + self.minute as f64 / 60.0 + self.second as f64 / 3600.0;
        astro::julian_day(self.year, self.month, self.day, h - self.tz)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Position {
    pub graha: Graha,
    /// Sidereal longitude, degrees.
    pub lon: f64,
    /// Degrees per day.
    pub speed: f64,
}

impl Position {
    pub fn sign(&self) -> usize {
        sign_of(self.lon)
    }
    pub fn retro(&self) -> bool {
        self.speed < 0.0 && !matches!(self.graha, Graha::Rahu | Graha::Ketu)
    }
}

pub fn sign_of(lon: f64) -> usize {
    ((norm360(lon) / 30.0).floor() as usize) % 12
}
pub fn nakshatra_of(lon: f64) -> usize {
    ((norm360(lon) / NAK_SPAN).floor() as usize) % 27
}
pub fn pada_of(lon: f64) -> usize {
    ((norm360(lon) % NAK_SPAN) / (NAK_SPAN / 4.0)).floor() as usize + 1
}

pub fn dms(x: f64) -> String {
    let x = norm360(x);
    let total = (x * 3600.0).round() as i64;
    format!("{:02}°{:02}'{:02}\"", total / 3600, (total / 60) % 60, total % 60)
}

/// Degrees within the sign as D°M'S".
pub fn in_sign(lon: f64) -> String {
    dms(norm360(lon) % 30.0)
}

/// The ayanamsa subtracted from tropical positions. Swiss Ephemeris positions are apparent (with
/// nutation), so the table's mean ayanamsa gets the same nutation added, as the reference reports print it.
pub fn ayanamsa_used(jde: f64, calc: Calc) -> f64 {
    // Swiss positions are apparent (with nutation), so the mean ayanamsa gets the same nutation added.
    calc.ayanamsa.value(jde) + crate::swe::nutation(jde)
}

/// Tropical longitude and daily speed from the Swiss Ephemeris (Moshier mode).
pub fn tropical_with_speed(g: Graha, jde: f64, calc: Calc) -> (f64, f64) {
    use crate::swe;
    let node = if calc.node == NodeKind::Mean { swe::MEAN_NODE } else { swe::TRUE_NODE };
    let body = match g {
        Graha::Sun => swe::SUN,
        Graha::Moon => swe::MOON,
        Graha::Mars => swe::MARS,
        Graha::Mercury => swe::MERCURY,
        Graha::Jupiter => swe::JUPITER,
        Graha::Venus => swe::VENUS,
        Graha::Saturn => swe::SATURN,
        Graha::Rahu | Graha::Ketu => node,
    };
    let (lon, speed) = swe::calc(jde, body);
    if g == Graha::Ketu { (norm360(lon + 180.0), speed) } else { (lon, speed) }
}

pub fn sidereal(g: Graha, jd_ut: f64, calc: Calc) -> Position {
    let t = astro::jde(jd_ut);
    let ay = ayanamsa_used(t, calc);
    let (trop, speed) = tropical_with_speed(g, t, calc);
    Position { graha: g, lon: norm360(trop - ay), speed }
}

#[derive(Clone, Debug)]
pub struct Chart {
    pub jd_ut: f64,
    pub ayanamsa: f64,
    pub asc: f64,
    pub mc: f64,
    pub planets: Vec<Position>,
    pub lat: f64,
    pub lon: f64,
    pub tz: f64,
}

impl Chart {
    pub fn compute(jd_ut: f64, lat: f64, lon: f64, tz: f64, calc: Calc) -> Chart {
        let t = astro::jde(jd_ut);
        let ay = ayanamsa_used(t, calc);
        let (asc, mc) = crate::swe::asc_mc(jd_ut, lat, lon);
        let planets = GRAHAS.iter().map(|&g| sidereal(g, jd_ut, calc)).collect();
        Chart {
            jd_ut,
            ayanamsa: ay,
            asc: norm360(asc - ay),
            mc: norm360(mc - ay),
            planets,
            lat,
            lon,
            tz,
        }
    }
    pub fn get(&self, g: Graha) -> Position {
        self.planets[g as usize]
    }
    pub fn moon(&self) -> f64 {
        self.get(Graha::Moon).lon
    }
    pub fn asc_sign(&self) -> usize {
        sign_of(self.asc)
    }
    /// House (1..12, whole sign from the ascendant) of a planet.
    pub fn house_of(&self, g: Graha) -> usize {
        (self.get(g).sign() + 12 - self.asc_sign()) % 12 + 1
    }
}

// ---------------------------------------------------------------- dignity

pub fn dignity(g: Graha, sign: usize) -> &'static str {
    let exalt = [0usize, 1, 9, 5, 3, 11, 6];
    let own: [&[usize]; 7] = [&[4], &[3], &[0, 7], &[2, 5], &[8, 11], &[1, 6], &[9, 10]];
    let i = g as usize;
    if i > 6 {
        return "";
    }
    if exalt[i] == sign {
        "Exalted"
    } else if (exalt[i] + 6) % 12 == sign {
        "Debilitated"
    } else if own[i].contains(&sign) {
        "Own sign"
    } else {
        match relation(g, SIGN_LORD[sign]) {
            Rel::Friend => "Friendly",
            Rel::Neutral => "Neutral",
            Rel::Enemy => "Enemy",
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Rel {
    Friend,
    Neutral,
    Enemy,
}

/// Natural (naisargika) relationship of `a` towards `b`.
pub fn relation(a: Graha, b: Graha) -> Rel {
    use Graha::*;
    use Rel::*;
    if a == b {
        return Friend;
    }
    let (f, e): (&[Graha], &[Graha]) = match a {
        Sun => (&[Moon, Mars, Jupiter], &[Venus, Saturn]),
        Moon => (&[Sun, Mercury], &[]),
        Mars => (&[Sun, Moon, Jupiter], &[Mercury]),
        Mercury => (&[Sun, Venus], &[Moon]),
        Jupiter => (&[Sun, Moon, Mars], &[Mercury, Venus]),
        Venus => (&[Mercury, Saturn], &[Sun, Moon]),
        Saturn => (&[Mercury, Venus], &[Sun, Moon, Mars]),
        _ => (&[], &[]),
    };
    if f.contains(&b) {
        Friend
    } else if e.contains(&b) {
        Enemy
    } else {
        Neutral
    }
}

// ---------------------------------------------------------------- divisional charts

pub const VARGAS: [(u32, &str); 11] = [
    (1, "Rasi"),
    (2, "Hora"),
    (3, "Drekkana"),
    (4, "Chaturthamsa"),
    (7, "Saptamsa"),
    (9, "Navamsa"),
    (10, "Dasamsa"),
    (12, "Dwadasamsa"),
    (16, "Shodasamsa"),
    (30, "Trimsamsa"),
    (60, "Shashtiamsa"),
];

/// "D9 Navamsa" in the current language.
pub fn varga_label(d: u32) -> String {
    let name = VARGAS.iter().find(|v| v.0 == d).map(|v| v.1).unwrap_or("");
    format!("D{} {}", d, i18n::tr(name))
}

/// Sign occupied in divisional chart `d` by sidereal longitude `lon`.
pub fn varga_sign(lon: f64, d: u32) -> usize {
    let lon = norm360(lon);
    let s = sign_of(lon);
    let deg = lon % 30.0;
    let odd = s.is_multiple_of(2); // Aries (index 0) is an odd sign
    let part = |n: u32| ((deg / (30.0 / n as f64)).floor() as usize).min(n as usize - 1);
    match d {
        1 => s,
        2 => {
            let first = deg < 15.0;
            if odd == first { 4 } else { 3 }
        }
        3 => (s + 4 * part(3)) % 12,
        4 => (s + 3 * part(4)) % 12,
        7 => (if odd { s } else { s + 6 } + part(7)) % 12,
        9 => ([0, 9, 6, 3][s % 4] + part(9)) % 12,
        10 => (if odd { s } else { s + 8 } + part(10)) % 12,
        12 => (s + part(12)) % 12,
        16 => ([0, 4, 8][s % 3] + part(16)) % 12,
        30 => {
            if odd {
                match deg {
                    x if x < 5.0 => 0,
                    x if x < 10.0 => 10,
                    x if x < 18.0 => 8,
                    x if x < 25.0 => 2,
                    _ => 6,
                }
            } else {
                match deg {
                    x if x < 5.0 => 1,
                    x if x < 12.0 => 5,
                    x if x < 20.0 => 11,
                    x if x < 25.0 => 9,
                    _ => 7,
                }
            }
        }
        60 => (s + part(60)) % 12,
        _ => s,
    }
}

// ---------------------------------------------------------------- bhava (houses)

/// Sripati bhavas: each quadrant between the angles trisected (Porphyry). Returns
/// (bhava madhya, bhava sandhi starting that bhava) for houses 1..12.
pub fn sripati_bhavas(asc: f64, mc: f64) -> Vec<(f64, f64)> {
    let ic = norm360(mc + 180.0);
    let dsc = norm360(asc + 180.0);
    let mut cusp = [0.0f64; 12];
    let quads = [(asc, ic, 0usize), (ic, dsc, 3), (dsc, mc, 6), (mc, asc, 9)];
    for (from, to, start) in quads {
        let span = norm360(to - from);
        for k in 0..3 {
            cusp[start + k] = norm360(from + span * k as f64 / 3.0);
        }
    }
    (0..12)
        .map(|i| {
            let prev = cusp[(i + 11) % 12];
            let sandhi = norm360(prev + norm360(cusp[i] - prev) / 2.0);
            (cusp[i], sandhi)
        })
        .collect()
}

// ---------------------------------------------------------------- panchanga

#[derive(Clone, Debug)]
pub struct Panchanga {
    pub tithi: usize, // 0..29
    pub shukla: bool,
    pub nakshatra: usize,
    pub pada: usize,
    pub yoga: usize,
    /// 0..6 Bava..Vishti, 7 Shakuni, 8 Chatushpada, 9 Naga, 10 Kimstughna.
    pub karana: usize,
    pub weekday: usize,
    pub sunrise: Option<f64>,
    pub sunset: Option<f64>,
}

pub fn karana_index(diff: f64) -> usize {
    let k = (norm360(diff) / 6.0).floor() as usize % 60;
    match k {
        0 => 10,
        57 => 7,
        58 => 8,
        59 => 9,
        _ => (k - 1) % 7,
    }
}

/// UT Julian day of local midnight that starts the civil day containing `jd_ut`.
pub fn local_midnight(jd_ut: f64, tz: f64) -> f64 {
    let local = jd_ut + tz / 24.0;
    (local - 0.5).floor() + 0.5 - tz / 24.0
}

pub fn panchanga(chart: &Chart) -> Panchanga {
    let sun = chart.get(Graha::Sun).lon;
    let moon = chart.moon();
    let diff = norm360(moon - sun);
    let tithi = (diff / 12.0).floor() as usize % 30;
    let midnight = local_midnight(chart.jd_ut, chart.tz);
    let (rise, set) = astro::sunrise_sunset(midnight, chart.lat, chart.lon);
    // The Vedic day runs from sunrise to sunrise.
    let mut wd = astro::weekday(midnight + chart.tz / 24.0 + 0.5);
    if let Some(r) = rise
        && chart.jd_ut < r {
            wd = (wd + 6) % 7;
        }
    Panchanga {
        tithi,
        shukla: tithi < 15,
        nakshatra: nakshatra_of(moon),
        pada: pada_of(moon),
        yoga: (norm360(sun + moon) / NAK_SPAN).floor() as usize % 27,
        karana: karana_index(diff),
        weekday: wd,
        sunrise: rise,
        sunset: set,
    }
}

/// Next UT time after `jd` at which `f(jd)` (an angle growing with time) reaches a
/// multiple of `span`. Used for tithi / nakshatra / yoga ending times.
pub fn next_boundary(jd: f64, span: f64, f: impl Fn(f64) -> f64) -> f64 {
    let start = f(jd);
    let target = ((start / span).floor() + 1.0) * span;
    let gap = |t: f64| {
        let v = f(t);
        // progress from start, unwrapped
        norm360(v - start) - norm360(target - start)
    };
    let mut lo = jd;
    let mut hi = jd + 1.0 / 24.0;
    while gap(hi) < 0.0 && hi < jd + 3.0 {
        lo = hi;
        hi += 1.0 / 24.0;
    }
    for _ in 0..40 {
        let mid = 0.5 * (lo + hi);
        if gap(mid) < 0.0 {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    0.5 * (lo + hi)
}

pub fn moon_sid(jd: f64, calc: Calc) -> f64 {
    sidereal(Graha::Moon, jd, calc).lon
}
pub fn sun_sid(jd: f64, calc: Calc) -> f64 {
    sidereal(Graha::Sun, jd, calc).lon
}

// ---------------------------------------------------------------- Vimshottari dasha

#[derive(Clone, Debug)]
pub struct Period {
    pub lord: Graha,
    pub start: f64,
    pub end: f64,
    pub sub: Vec<Period>,
}

/// Mahadashas (with bhuktis and antaras) from the birth Moon.
pub fn vimshottari(moon_lon: f64, birth_jd: f64, levels: usize) -> Vec<Period> {
    let nak = nakshatra_of(moon_lon);
    let frac = (norm360(moon_lon) % NAK_SPAN) / NAK_SPAN;
    let first = nak % 9;
    let first_years = DASHA[first].1;
    // Start of the running mahadasha (before birth).
    let start = birth_jd - frac * first_years * DASHA_YEAR_DAYS;
    let mut out = Vec::new();
    let mut t = start;
    for k in 0..9 {
        let idx = (first + k) % 9;
        let len = DASHA[idx].1 * DASHA_YEAR_DAYS;
        out.push(Period {
            lord: DASHA[idx].0,
            start: t,
            end: t + len,
            sub: subperiods(idx, t, len, levels.saturating_sub(1)),
        });
        t += len;
    }
    out
}

fn subperiods(lord_idx: usize, start: f64, len: f64, levels: usize) -> Vec<Period> {
    if levels == 0 {
        return Vec::new();
    }
    let mut out = Vec::new();
    let mut t = start;
    for k in 0..9 {
        let idx = (lord_idx + k) % 9;
        let l = len * DASHA[idx].1 / 120.0;
        out.push(Period {
            lord: DASHA[idx].0,
            start: t,
            end: t + l,
            sub: subperiods(idx, t, l, levels - 1),
        });
        t += l;
    }
    out
}

/// Balance of the first mahadasha at birth, in years.
pub fn dasha_balance(moon_lon: f64) -> (Graha, f64) {
    let nak = nakshatra_of(moon_lon);
    let frac = (norm360(moon_lon) % NAK_SPAN) / NAK_SPAN;
    let (g, y) = DASHA[nak % 9];
    (g, (1.0 - frac) * y)
}

pub fn years_to_ymd(y: f64) -> String {
    let years = y.floor();
    let months_f = (y - years) * 12.0;
    let months = months_f.floor();
    let days = ((months_f - months) * 30.0).floor();
    trf("{0} y {1} m {2} d", &[(years as i64).to_string(), (months as i64).to_string(), (days as i64).to_string()])
}

// ---------------------------------------------------------------- marriage matching

#[derive(Clone, Debug)]
pub struct Koota {
    pub name: &'static str,
    pub max: f64,
    pub got: f64,
    pub note: String,
}

const YONI_ANIMAL: [usize; 27] = [
    0, 1, 2, 3, 3, 4, 5, 2, 5, 6, 6, 7, 8, 9, 8, 9, 10, 10, 4, 11, 12, 11, 13, 0, 13, 7, 1,
];
pub fn yoni_animal(nak: usize) -> usize {
    YONI_ANIMAL[nak % 27]
}
pub const YONI_NAMES: [&str; 14] = [
    "Horse", "Elephant", "Sheep", "Serpent", "Dog", "Cat", "Rat", "Cow", "Buffalo", "Tiger",
    "Deer", "Monkey", "Mongoose", "Lion",
];
const YONI_SCORE: [[u8; 14]; 14] = [
    [4, 2, 2, 3, 2, 2, 2, 1, 0, 1, 3, 3, 2, 1],
    [2, 4, 3, 3, 2, 2, 2, 2, 3, 1, 2, 3, 2, 0],
    [2, 3, 4, 2, 1, 2, 1, 3, 3, 1, 2, 0, 3, 1],
    [3, 3, 2, 4, 2, 1, 1, 1, 1, 2, 2, 2, 0, 2],
    [2, 2, 1, 2, 4, 2, 1, 2, 2, 1, 0, 2, 1, 1],
    [2, 2, 2, 1, 2, 4, 0, 2, 2, 1, 3, 3, 2, 1],
    [2, 2, 1, 1, 1, 0, 4, 2, 2, 2, 2, 2, 1, 2],
    [1, 2, 3, 1, 2, 2, 2, 4, 3, 0, 3, 2, 2, 1],
    [0, 3, 3, 1, 2, 2, 2, 3, 4, 1, 2, 2, 2, 1],
    [1, 1, 1, 2, 1, 1, 2, 0, 1, 4, 1, 1, 2, 1],
    [3, 2, 2, 2, 0, 3, 2, 3, 2, 1, 4, 2, 2, 1],
    [3, 3, 0, 2, 2, 3, 2, 2, 2, 1, 2, 4, 3, 2],
    [2, 2, 3, 0, 1, 2, 1, 2, 2, 2, 2, 3, 4, 2],
    [1, 0, 1, 2, 1, 1, 2, 1, 1, 1, 1, 2, 2, 4],
];
/// 0 Deva, 1 Manushya, 2 Rakshasa
pub const GANA: [usize; 27] = [
    0, 1, 2, 1, 0, 1, 0, 0, 2, 2, 1, 1, 0, 2, 0, 2, 0, 2, 2, 1, 1, 0, 2, 2, 1, 1, 0,
];
pub const GANA_NAMES: [&str; 3] = ["Deva", "Manushya", "Rakshasa"];
/// 0 Adi, 1 Madhya, 2 Antya
pub fn nadi(nak: usize) -> usize {
    [0, 1, 2, 2, 1, 0][nak % 6]
}
pub const NADI_NAMES: [&str; 3] = ["Adi (Vata)", "Madhya (Pitta)", "Antya (Kapha)"];
/// 4 Brahmin, 3 Kshatriya, 2 Vaishya, 1 Shudra
pub fn varna(sign: usize) -> usize {
    [3, 2, 1, 4][sign % 4]
}
pub const VARNA_NAMES: [&str; 5] = ["", "Shudra", "Vaishya", "Kshatriya", "Brahmin"];
/// 0 Chatushpada, 1 Manava, 2 Jalachara, 3 Vanachara, 4 Keeta (sign level)
pub fn vashya(sign: usize) -> usize {
    [0, 0, 1, 2, 3, 1, 1, 4, 1, 2, 1, 2][sign]
}
pub const VASHYA_NAMES: [&str; 5] = ["Chatushpada", "Manava", "Jalachara", "Vanachara", "Keeta"];
const VASHYA_SCORE: [[f64; 5]; 5] = [
    [2.0, 1.0, 1.0, 0.5, 1.0],
    [1.0, 2.0, 0.5, 0.0, 1.0],
    [1.0, 0.5, 2.0, 1.0, 1.0],
    [0.5, 0.0, 1.0, 2.0, 0.0],
    [1.0, 1.0, 1.0, 0.0, 2.0],
];

fn maitri_score(a: Graha, b: Graha) -> f64 {
    use Rel::*;
    match (relation(a, b), relation(b, a)) {
        (Friend, Friend) => 5.0,
        (Friend, Neutral) | (Neutral, Friend) => 4.0,
        (Neutral, Neutral) => 3.0,
        (Friend, Enemy) | (Enemy, Friend) => 1.0,
        (Neutral, Enemy) | (Enemy, Neutral) => 0.5,
        (Enemy, Enemy) => 0.0,
    }
}

/// Ashtakoota (36-point) matching. Inputs are the Moon's sidereal longitudes.
pub fn ashtakoota(boy_moon: f64, girl_moon: f64) -> Vec<Koota> {
    let (bn, gn) = (nakshatra_of(boy_moon), nakshatra_of(girl_moon));
    let (bs, gs) = (sign_of(boy_moon), sign_of(girl_moon));
    let mut v = Vec::new();

    let (bv, gv) = (varna(bs), varna(gs));
    v.push(Koota {
        name: "Varna",
        max: 1.0,
        got: if bv >= gv { 1.0 } else { 0.0 },
        note: format!("{} / {}", tr(VARNA_NAMES[bv]), tr(VARNA_NAMES[gv])),
    });

    let (bw, gw) = (vashya(bs), vashya(gs));
    v.push(Koota {
        name: "Vashya",
        max: 2.0,
        got: VASHYA_SCORE[bw][gw],
        note: format!("{} / {}", tr(VASHYA_NAMES[bw]), tr(VASHYA_NAMES[gw])),
    });

    let tara_ok = |from: usize, to: usize| {
        let n = (to + 27 - from) % 27 + 1;
        !matches!(n % 9, 3 | 5 | 7)
    };
    let tara = 1.5 * tara_ok(gn, bn) as u8 as f64 + 1.5 * tara_ok(bn, gn) as u8 as f64;
    v.push(Koota {
        name: "Tara (Dina)",
        max: 3.0,
        got: tara,
        note: trf("counted both ways from {0} and {1}", &[i18n::nak(bn).into(), i18n::nak(gn).into()]),
    });

    let (by, gy) = (YONI_ANIMAL[bn], YONI_ANIMAL[gn]);
    v.push(Koota {
        name: "Yoni",
        max: 4.0,
        got: YONI_SCORE[by][gy] as f64,
        note: format!("{} / {}", tr(YONI_NAMES[by]), tr(YONI_NAMES[gy])),
    });

    let (bl, gl) = (SIGN_LORD[bs], SIGN_LORD[gs]);
    v.push(Koota {
        name: "Graha Maitri",
        max: 5.0,
        got: maitri_score(bl, gl),
        note: format!("{} / {}", i18n::planet(bl), i18n::planet(gl)),
    });

    let (bg, gg) = (GANA[bn], GANA[gn]);
    let gana = match (bg, gg) {
        (a, b) if a == b => 6.0,
        (0, 1) => 6.0,
        (1, 0) => 5.0,
        (0, 2) | (2, 0) => 1.0,
        _ => 0.0,
    };
    v.push(Koota {
        name: "Gana",
        max: 6.0,
        got: gana,
        note: format!("{} / {}", tr(GANA_NAMES[bg]), tr(GANA_NAMES[gg])),
    });

    let d = (bs + 12 - gs) % 12 + 1;
    let r = (gs + 12 - bs) % 12 + 1;
    let bad = matches!(d, 2 | 12 | 5 | 9 | 6 | 8);
    v.push(Koota {
        name: "Bhakoot (Rasi)",
        max: 7.0,
        got: if bad { 0.0 } else { 7.0 },
        note: trf("{0} / {1} ({2}/{3} apart)", &[i18n::sign(bs).into(), i18n::sign(gs).into(), d.to_string(), r.to_string()]),
    });

    let (bnd, gnd) = (nadi(bn), nadi(gn));
    v.push(Koota {
        name: "Nadi",
        max: 8.0,
        got: if bnd == gnd { 0.0 } else { 8.0 },
        note: format!("{} / {}", tr(NADI_NAMES[bnd]), tr(NADI_NAMES[gnd])),
    });
    v
}

/// Mars (Kuja) dosha houses, houses 1, 2, 4, 7, 8 and 12.
pub const DOSHA_HOUSES: [usize; 6] = [1, 2, 4, 7, 8, 12];

/// Mars (Kuja) dosha counted from the ascendant and, for reference, from the Moon.
pub fn kuja_dosha(chart: &Chart) -> (bool, bool) {
    let mars = chart.get(Graha::Mars).sign();
    let from = |s: usize| (mars + 12 - s) % 12 + 1;
    let bad = |h: usize| DOSHA_HOUSES.contains(&h);
    (bad(from(chart.asc_sign())), bad(from(chart.get(Graha::Moon).sign())))
}

// ---------------------------------------------------------------- yogas

pub struct YogaFound {
    pub name: &'static str,
    pub why: String,
    /// The planets that form the yoga (its dasas bring it out).
    pub planets: Vec<Graha>,
}

pub fn find_yogas(c: &Chart) -> Vec<YogaFound> {
    use Graha::*;
    let mut out = Vec::new();
    let sign = |g: Graha| c.get(g).sign();
    let kendra = |from: usize, s: usize| matches!((s + 12 - from) % 12, 0 | 3 | 6 | 9);
    let moon = sign(Moon);

    if kendra(moon, sign(Jupiter)) {
        out.push(YogaFound { planets: vec![],
            name: "Gaja Kesari",
            why: tr("Jupiter is in a kendra (1, 4, 7, 10) from the Moon.").into(),
        });
    }
    if sign(Sun) == sign(Mercury) {
        out.push(YogaFound { planets: vec![],
            name: "Budha Aditya",
            why: trf("Sun and Mercury together in {0}.", &[i18n::sign(sign(Sun)).into()]),
        });
    }
    if sign(Moon) == sign(Mars) {
        out.push(YogaFound { planets: vec![],
            name: "Chandra Mangala",
            why: trf("Moon and Mars together in {0}.", &[i18n::sign(moon).into()]),
        });
    }
    let maha = [
        (Mars, "Ruchaka"),
        (Mercury, "Bhadra"),
        (Jupiter, "Hamsa"),
        (Venus, "Malavya"),
        (Saturn, "Sasa"),
    ];
    for (g, name) in maha {
        let d = dignity(g, sign(g));
        if (d == "Exalted" || d == "Own sign") && kendra(c.asc_sign(), sign(g)) {
            out.push(YogaFound { planets: vec![g],
                name,
                why: trf("{0} is {1} in a kendra from the ascendant (Pancha Mahapurusha).", &[i18n::planet(g).into(), tr(d).to_lowercase()]),
            });
        }
    }
    let others = [Mars, Mercury, Jupiter, Venus, Saturn];
    let second = others.iter().any(|&g| sign(g) == (moon + 1) % 12);
    let twelfth = others.iter().any(|&g| sign(g) == (moon + 11) % 12);
    match (second, twelfth) {
        (true, true) => out.push(YogaFound { planets: vec![], name: "Durudhara", why: tr("Planets on both sides of the Moon (2nd and 12th).").into() }),
        (true, false) => out.push(YogaFound { planets: vec![], name: "Sunapha", why: tr("A planet in the 2nd from the Moon.").into() }),
        (false, true) => out.push(YogaFound { planets: vec![], name: "Anapha", why: tr("A planet in the 12th from the Moon.").into() }),
        (false, false) => {
            let in_kendra = others.iter().any(|&g| kendra(c.asc_sign(), sign(g)));
            if !in_kendra {
                out.push(YogaFound { planets: vec![], name: "Kemadruma", why: tr("No planets beside the Moon and none in kendras.").into() });
            }
        }
    }
    let vesi = others.iter().any(|&g| sign(g) == (sign(Sun) + 1) % 12);
    if vesi {
        out.push(YogaFound { planets: vec![], name: "Vesi", why: tr("A planet (not the Moon) in the 2nd from the Sun.").into() });
    }
    // Kala Sarpa: all seven planets on one side of the Rahu-Ketu axis
    let rahu = c.get(Rahu).lon;
    let sides: Vec<bool> = [Sun, Moon, Mars, Mercury, Jupiter, Venus, Saturn]
        .iter()
        .map(|&g| norm360(c.get(g).lon - rahu) < 180.0)
        .collect();
    if sides.iter().all(|&s| s) || sides.iter().all(|&s| !s) {
        out.push(YogaFound { planets: vec![], name: "Kala Sarpa", why: tr("All seven planets lie on one side of the Rahu-Ketu axis.").into() });
    }

    let lagna = c.asc_sign();
    let from = |base: usize, s: usize| (s + 12 - base) % 12 + 1;
    let upachaya = |h: usize| matches!(h, 3 | 6 | 10 | 11);
    // Vasumathi: Jupiter, Venus and Mercury each in an upachaya (3, 6, 10, 11) from the Lagna or the Moon.
    if [Jupiter, Venus, Mercury].iter().all(|&g| upachaya(from(lagna, sign(g))) || upachaya(from(moon, sign(g)))) {
        out.push(YogaFound { planets: vec![], name: "Vasumathi", why: tr("Jupiter, Venus and Mercury are in upachaya houses (3, 6, 10, 11) from the Lagna or the Moon.").into() });
    }
    // Amala: a natural benefic in the 10th from the Lagna or the Moon.
    let benefic_in = |s: usize| [Jupiter, Venus, Mercury].iter().any(|&g| sign(g) == s);
    if benefic_in((lagna + 9) % 12) || benefic_in((moon + 9) % 12) {
        out.push(YogaFound { planets: vec![], name: "Amala", why: tr("A natural benefic occupies the 10th house from the Lagna or the Moon.").into() });
    }
    // Adhi: Jupiter, Venus and Mercury all in the 6th, 7th or 8th from the Moon.
    if [Jupiter, Venus, Mercury].iter().all(|&g| matches!(from(moon, sign(g)), 6..=8)) {
        out.push(YogaFound { planets: vec![], name: "Adhi", why: tr("Jupiter, Venus and Mercury are in the 6th, 7th and 8th houses from the Moon.").into() });
    }
    // Sakata: the Moon in the 6th, 8th or 12th from Jupiter.
    let mj = from(sign(Jupiter), moon);
    if matches!(mj, 6 | 8 | 12) {
        out.push(YogaFound { planets: vec![], name: "Sakata", why: trf("The Moon is in house {0} from Jupiter.", &[mj.to_string()]) });
    }
    // Parivartana: two planets in each other's signs.
    let seven = [Sun, Moon, Mars, Mercury, Jupiter, Venus, Saturn];
    for (i, &a) in seven.iter().enumerate() {
        for &b in &seven[i + 1..] {
            if SIGN_LORD[sign(a)] == b && SIGN_LORD[sign(b)] == a {
                out.push(YogaFound { planets: vec![a, b], name: "Parivartana", why: trf("{0} and {1} are in each other's signs.", &[i18n::planet(a).into(), i18n::planet(b).into()]) });
            }
        }
    }
    // Raja: a kendra lord and a different trikona lord in the same sign.
    let lord_of = |h: usize| SIGN_LORD[(lagna + h - 1) % 12];
    let mut seen: Vec<(Graha, Graha)> = Vec::new();
    for k in [1, 4, 7, 10] {
        for t in [5, 9] {
            let (a, b) = (lord_of(k), lord_of(t));
            if a != b && sign(a) == sign(b) && !seen.contains(&(a, b)) && !seen.contains(&(b, a)) {
                seen.push((a, b));
                out.push(YogaFound { planets: vec![a, b],
                    name: "Raja",
                    why: trf("{0} (lord of house {1}) and {2} (lord of house {3}) are together in {4}.", &[i18n::planet(a).into(), k.to_string(), i18n::planet(b).into(), t.to_string(), i18n::sign(sign(a)).into()]),
                });
            }
        }
    }
    // Three or more planets together in one sign.
    for s in 0..12 {
        let here: Vec<Graha> = seven.iter().copied().filter(|&g| sign(g) == s).collect();
        if here.len() >= 3 {
            let name = match here.len() {
                3 => "Trigraha",
                4 => "Chaturgraha",
                _ => "Panchagraha",
            };
            let names: Vec<&str> = here.iter().map(|&g| i18n::planet(g)).collect();
            out.push(YogaFound { planets: here.clone(), name, why: trf("{0} are together in house {1} ({2}).", &[names.join(", "), from(lagna, s).to_string(), i18n::sign(s).into()]) });
        }
    }
    for y in &mut out {
        if y.planets.is_empty() {
            y.planets = match y.name {
                "Gaja Kesari" | "Sakata" => vec![Moon, Jupiter],
                "Budha Aditya" => vec![Sun, Mercury],
                "Chandra Mangala" => vec![Moon, Mars],
                "Durudhara" | "Sunapha" | "Anapha" | "Kemadruma" => vec![Moon],
                "Vesi" => vec![Sun],
                "Kala Sarpa" => vec![Rahu, Ketu],
                "Vasumathi" | "Adhi" | "Amala" => vec![Jupiter, Venus, Mercury],
                _ => vec![],
            };
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn navamsa_rules() {
        assert_eq!(varga_sign(0.5, 9), 0); // Aries 0.5 -> Aries
        assert_eq!(varga_sign(30.5, 9), 9); // Taurus -> starts Capricorn
        assert_eq!(varga_sign(359.9, 9), 11); // end of Pisces -> Pisces
        assert_eq!(varga_sign(10.0, 2), 4);
        assert_eq!(varga_sign(40.0, 2), 3);
    }

    #[test]
    fn dasha_total() {
        let d = vimshottari(10.0, 2451545.0, 2);
        let total: f64 = d.iter().map(|p| p.end - p.start).sum();
        assert!((total / DASHA_YEAR_DAYS - 120.0).abs() < 1e-6);
        let sub: f64 = d[1].sub.iter().map(|p| p.end - p.start).sum();
        assert!((sub - (d[1].end - d[1].start)).abs() < 1e-6);
    }

    #[test]
    fn matching_bounds() {
        let k = ashtakoota(100.0, 200.0);
        let total: f64 = k.iter().map(|x| x.max).sum();
        assert_eq!(total, 36.0);
        let k = ashtakoota(100.0, 100.0);
        assert_eq!(k[7].got, 0.0); // same nadi
    }
}
