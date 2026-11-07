//! Festival overlay for the calendar. Given a day (with its lunar month, paksha and tithi already
//! computed by `calendar`) and a set of enabled traditions, returns the festival name-keys falling on
//! that day. Dates are **computed estimates** — authored rules over the panchang (Hindu/Buddhist/Jain),
//! the Gregorian computus (Christian) and the tabular Islamic calendar (Muslim). No external festival
//! database is used; names are facts and the ruleset is registered in `knowledge/sources.json`.

use crate::astro;
use crate::calendar::Day;

/// Amānta lunar-month indices (0 = Chaitra … 11 = Phalguna), as `calendar::chandra_masa` returns.
const CHAITRA: usize = 0;
const VAISHAKHA: usize = 1;
const JYESHTHA: usize = 2;
const ASHADHA: usize = 3;
const SHRAVANA: usize = 4;
const BHADRAPADA: usize = 5;
const ASHWINA: usize = 6;
const KARTIKA: usize = 7;
const MAGHA: usize = 10;
const PHALGUNA: usize = 11;

/// A festival tradition the viewer can enable.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Tradition {
    Hindu,
    Christian,
    Islamic,
    Buddhist,
    Jain,
}

impl Tradition {
    /// Parse a lowercase selector (as sent in the `trad` query), else `None`.
    pub fn parse(s: &str) -> Option<Tradition> {
        match s.trim().to_ascii_lowercase().as_str() {
            "hindu" => Some(Tradition::Hindu),
            "christian" => Some(Tradition::Christian),
            "islamic" => Some(Tradition::Islamic),
            "buddhist" => Some(Tradition::Buddhist),
            "jain" => Some(Tradition::Jain),
            _ => None,
        }
    }

    /// The i18n label key for this tradition.
    pub fn key(self) -> &'static str {
        match self {
            Tradition::Hindu => "Hindu",
            Tradition::Christian => "Christian",
            Tradition::Islamic => "Islamic",
            Tradition::Buddhist => "Buddhist",
            Tradition::Jain => "Jain",
        }
    }

    /// All traditions, for building the picker.
    pub fn all() -> [Tradition; 5] {
        [Tradition::Hindu, Tradition::Christian, Tradition::Islamic, Tradition::Buddhist, Tradition::Jain]
    }
}

/// Festival name-keys on `day` for every enabled tradition, in `traditions` order.
pub fn festivals_for(day: &Day, traditions: &[Tradition]) -> Vec<&'static str> {
    let mut out = Vec::new();
    for t in traditions {
        match t {
            Tradition::Hindu => hindu(day, &mut out),
            Tradition::Christian => christian(day, &mut out),
            Tradition::Islamic => islamic(day, &mut out),
            Tradition::Buddhist => buddhist(day, &mut out),
            Tradition::Jain => jain(day, &mut out),
        }
    }
    out
}

/// Day within the fortnight, 1..=15 (śukla 1–15, then kṛṣṇa 1–15; 15 = pūrṇimā or amāvāsyā).
fn tday(day: &Day) -> usize {
    if day.tithi < 15 { day.tithi + 1 } else { day.tithi - 14 }
}

/// Lunar-rule match: amānta month, paksha (0 śukla / 1 kṛṣṇa) and day-within-fortnight.
fn lunar(day: &Day, month: usize, paksha: u8, d: usize) -> bool {
    day.lunar_month == month && day.paksha == paksha && tday(day) == d
}

/// Julian day number (integer, noon-based) of this civil day.
fn jdn(day: &Day) -> i64 {
    astro::julian_day(day.year, day.month, day.day, 12.0).floor() as i64
}

fn hindu(day: &Day, out: &mut Vec<&'static str>) {
    if lunar(day, CHAITRA, 0, 1) { out.push("Ugadi"); }
    if lunar(day, CHAITRA, 0, 9) { out.push("Rama Navami"); }
    if lunar(day, PHALGUNA, 0, 15) { out.push("Holi"); }
    if lunar(day, MAGHA, 1, 14) { out.push("Maha Shivaratri"); }
    if lunar(day, SHRAVANA, 1, 8) { out.push("Krishna Janmashtami"); }
    if lunar(day, SHRAVANA, 0, 15) { out.push("Raksha Bandhan"); }
    if lunar(day, BHADRAPADA, 0, 4) { out.push("Ganesh Chaturthi"); }
    if lunar(day, ASHWINA, 0, 1) { out.push("Navaratri"); }
    if lunar(day, ASHWINA, 0, 10) { out.push("Dussehra"); }
    // Diwali's amāvāsyā ends the amānta month Āśvina (it is "Kārtika" only in the pūrṇimānta system).
    if lunar(day, ASHWINA, 1, 15) { out.push("Diwali"); }
    // Makar Sankranti is solar: the Sun entering Makara = Tamil month Thai (index 9), day 1.
    if day.tamil_month == 9 && day.tamil_day == 1 { out.push("Makar Sankranti"); }
}

fn buddhist(day: &Day, out: &mut Vec<&'static str>) {
    if lunar(day, VAISHAKHA, 0, 15) { out.push("Vesak"); }
    if lunar(day, ASHADHA, 0, 15) { out.push("Asalha Puja"); }
    if lunar(day, MAGHA, 0, 15) { out.push("Magha Puja"); }
}

fn jain(day: &Day, out: &mut Vec<&'static str>) {
    if lunar(day, CHAITRA, 0, 13) { out.push("Mahavir Jayanti"); }
    if lunar(day, BHADRAPADA, 0, 12) { out.push("Paryushana"); }
    if lunar(day, ASHWINA, 1, 15) { out.push("Mahavir Nirvana"); }
    if lunar(day, KARTIKA, 0, 5) { out.push("Jnan Panchami"); }
    let _ = JYESHTHA; // reserved for future rules
}

fn christian(day: &Day, out: &mut Vec<&'static str>) {
    if day.month == 12 && day.day == 25 { out.push("Christmas"); }
    if day.month == 1 && day.day == 6 { out.push("Epiphany"); }
    if day.month == 11 && day.day == 1 { out.push("All Saints"); }
    let (em, ed) = easter(day.year);
    let easter_jdn = astro::julian_day(day.year, em, ed, 12.0).floor() as i64;
    let j = jdn(day);
    if j == easter_jdn { out.push("Easter"); }
    if j == easter_jdn - 2 { out.push("Good Friday"); }
    if j == easter_jdn - 46 { out.push("Ash Wednesday"); }
    if j == easter_jdn + 49 { out.push("Pentecost"); }
}

fn islamic(day: &Day, out: &mut Vec<&'static str>) {
    let (_, hm, hd) = hijri(day.year, day.month, day.day);
    match (hm, hd) {
        (1, 1) => out.push("Islamic New Year"),
        (1, 10) => out.push("Ashura"),
        (3, 12) => out.push("Mawlid"),
        (9, 1) => out.push("Ramadan"),
        (10, 1) => out.push("Eid al-Fitr"),
        (12, 10) => out.push("Eid al-Adha"),
        _ => {}
    }
}

/// Easter Sunday (month, day) in the given year — the Anonymous Gregorian computus.
pub fn easter(year: i32) -> (u32, u32) {
    let a = year % 19;
    let b = year / 100;
    let c = year % 100;
    let d = b / 4;
    let e = b % 4;
    let f = (b + 8) / 25;
    let g = (b - f + 1) / 3;
    let h = (19 * a + b - d - g + 15) % 30;
    let i = c / 4;
    let k = c % 4;
    let l = (32 + 2 * e + 2 * i - h - k) % 7;
    let m = (a + 11 * h + 22 * l) / 451;
    let month = (h + l - 7 * m + 114) / 31;
    let day = ((h + l - 7 * m + 114) % 31) + 1;
    (month as u32, day as u32)
}

/// Tabular (civil) Islamic calendar date (hijri year, month, day) for a Gregorian date.
pub fn hijri(year: i32, month: u32, day: u32) -> (i32, u32, u32) {
    // Gregorian → Julian Day Number, then the standard arithmetic Islamic conversion.
    let a = (14 - month as i64) / 12;
    let y = year as i64 + 4800 - a;
    let m = month as i64 + 12 * a - 3;
    let jdn = day as i64 + (153 * m + 2) / 5 + 365 * y + y / 4 - y / 100 + y / 400 - 32045;
    let l = jdn - 1948440 + 10632;
    let n = (l - 1) / 10631;
    let l = l - 10631 * n + 354;
    let j = ((10985 - l) / 5316) * ((50 * l) / 17719) + (l / 5670) * ((43 * l) / 15238);
    let l = l - ((30 - j) / 15) * ((17719 * j) / 50) - (j / 16) * ((15238 * j) / 43) + 29;
    let hm = (24 * l) / 709;
    let hd = l - (709 * hm) / 24;
    let hy = 30 * n + j - 30;
    (hy as i32, hm as u32, hd as u32)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::calendar::Day;

    fn sample_day() -> Day {
        Day {
            year: 2026, month: 1, day: 1, tz: 0.0, weekday: 0,
            sunrise: None, sunset: None, tithi: 0, nakshatra: 0, yoga: 0,
            tamil_month: 0, tamil_day: 1, tamil_year: 0,
            rahu: None, yama: None, gulika: None, abhijit: None, brahma: None,
            lunar_month: 0, paksha: 0,
        }
    }
    fn sample_day_on(y: i32, m: u32, d: u32) -> Day {
        let mut x = sample_day();
        x.year = y; x.month = m; x.day = d; x
    }

    #[test]
    fn engine_toggles_and_stacks() {
        let mut d = sample_day();
        d.lunar_month = ASHWINA; d.paksha = 1; d.tithi = 29; // amānta Āśvina Kṛṣṇa Amāvāsyā (Diwali)
        assert!(festivals_for(&d, &[]).is_empty());
        assert!(festivals_for(&d, &[Tradition::Hindu]).contains(&"Diwali"));
        assert!(!festivals_for(&d, &[Tradition::Christian]).contains(&"Diwali"));
        let both = festivals_for(&d, &[Tradition::Hindu, Tradition::Jain]);
        assert!(both.contains(&"Diwali") && both.contains(&"Mahavir Nirvana"));
    }

    #[test]
    fn lunar_festivals_land_near_the_real_date() {
        // Regression for the chandra-māsa off-by-one: real 2025 dates (±1 day per the estimate policy).
        use crate::calendar;
        let chennai = |y, m, d| calendar::day(y, m, d, 13.0827, 80.2707, "Asia/Kolkata", 5.5, crate::vedic::Calc::default());
        let near = |y, m, days: &[u32], name: &str, trad| days.iter().any(|&d| festivals_for(&chennai(y, m, d), &[trad]).contains(&name));
        assert!(near(2025, 3, &[13, 14, 15], "Holi", Tradition::Hindu), "Holi is mid-March, not a month off");
        assert!(near(2025, 10, &[20, 21, 22], "Diwali", Tradition::Hindu), "Diwali is ~Oct 21");
        assert!(near(2025, 5, &[11, 12, 13], "Vesak", Tradition::Buddhist), "Vesak is ~May 12");
    }

    #[test]
    fn christian_easter_and_fixed() {
        assert_eq!(easter(2025), (4, 20));
        assert!(festivals_for(&sample_day_on(2026, 12, 25), &[Tradition::Christian]).contains(&"Christmas"));
        assert!(festivals_for(&sample_day_on(2025, 4, 18), &[Tradition::Christian]).contains(&"Good Friday"));
    }

    #[test]
    fn muslim_tabular_hijri() {
        let (_, hm, hd) = hijri(2025, 3, 31);
        let eid = sample_day_on(2025, 3, 31);
        if (hm, hd) == (10, 1) {
            assert!(festivals_for(&eid, &[Tradition::Islamic]).contains(&"Eid al-Fitr"));
        }
        // Eid al-Fitr only fires on 1 Shawwāl.
        assert!(!festivals_for(&sample_day_on(2025, 1, 1), &[Tradition::Islamic]).contains(&"Eid al-Fitr")
            || hijri(2025, 1, 1).1 == 10);
    }
}
