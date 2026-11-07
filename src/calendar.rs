//! Tamil / Vedic daily calendar: for each civil day at a place, the panchanga at sunrise, the Tamil
//! solar date, and the day's auspicious and inauspicious periods.
//!
//! Rules (standard almanac practice):
//! * Tithi, nakshatra and yoga are those running at local sunrise.
//! * Tamil month = the Sun's sidereal sign. A month starts on the day the Sun changes sign if that
//!   happens before sunset, otherwise on the next day.
//! * Rahu kalam, Yamagandam and Gulika kalam are eighths of daylight (sunrise to sunset), chosen by
//!   weekday: Rahu 8,2,7,5,6,4,3; Yamagandam 5,4,3,2,1,7,6; Gulika 7,6,5,4,3,2,1 (Sunday..Saturday).
//! * Abhijit muhurta = the 8th of the 15 daytime muhurtas (centred on local noon); traditionally
//!   avoided on Wednesdays.
//! * Brahma muhurta = the 14th of the 15 night muhurtas (two muhurtas before sunrise to one before).

use crate::astro::{self, norm360};
use crate::vedic::*;

/// One day of the calendar. Times are UT Julian days.
#[derive(Clone, Debug)]
pub struct Day {
    pub year: i32,
    pub month: u32,
    pub day: u32,
    /// Hours east of UTC used for this day (daylight saving follows the zone).
    pub tz: f64,
    /// 0 = Sunday.
    pub weekday: usize,
    pub sunrise: Option<f64>,
    pub sunset: Option<f64>,
    /// 0..29 (14 = Pournami, 29 = Amavasya).
    pub tithi: usize,
    pub nakshatra: usize,
    pub yoga: usize,
    /// 0 = Chithirai (Sun in Mesha) .. 11 = Panguni.
    pub tamil_month: usize,
    pub tamil_day: u32,
    /// Year of the 60-year cycle (0 = Prabhava).
    pub tamil_year: usize,
    pub rahu: Option<(f64, f64)>,
    pub yama: Option<(f64, f64)>,
    pub gulika: Option<(f64, f64)>,
    pub abhijit: Option<(f64, f64)>,
    pub brahma: Option<(f64, f64)>,
    /// Amānta lunar month at sunrise, 0 = Chaitra … 11 = Phalguna.
    pub lunar_month: usize,
    /// 0 = śukla (waxing, tithi < 15), 1 = kṛṣṇa (waning).
    pub paksha: u8,
}

const RAHU: [usize; 7] = [8, 2, 7, 5, 6, 4, 3];
const YAMA: [usize; 7] = [5, 4, 3, 2, 1, 7, 6];
const GULIKA: [usize; 7] = [7, 6, 5, 4, 3, 2, 1];

/// Hours east of UTC on that date: the zone's offset when known, else `tz`.
fn offset(tz_name: &str, tz: f64, y: i32, m: u32, d: u32) -> f64 {
    crate::geo::utc_offset(tz_name, y, m, d, 12, 0).unwrap_or(tz)
}

/// Hour of the day (Indian Standard Time) by which the Sun must have entered a sign for that civil
/// day to be the 1st of the Tamil month. Fitted to the reference Tamil year table
/// (the reference Tamil year table, 95 years, 1,140 month starts) with the Surya Siddhanta Sun:
/// a cut at 22:00 IST reproduces 1,079 of them (94.6 %); see MECHANICS.md.
pub const TAMIL_CUT_HOURS: f64 = 22.0;

/// Tamil solar month (0 = Chithirai) of a civil date, as traditional Tamil almanacs reckon it:
/// the Surya Siddhanta Sun's sign at the cut-off hour of that date, Indian Standard Time.
/// Like the reference table, this is the same for every place.
pub fn tamil_month_of(y: i32, m: u32, d: u32) -> usize {
    let at = astro::julian_day(y, m, d, 0.0) - 5.5 / 24.0 + TAMIL_CUT_HOURS / 24.0;
    sign_of(ss_sun(at))
}

/// Day of the Tamil month (1-based) of a civil date.
pub fn tamil_day_of(y: i32, m: u32, d: u32) -> u32 {
    let month = tamil_month_of(y, m, d);
    let jd = astro::julian_day(y, m, d, 12.0);
    let mut n = 1;
    while n < 33 {
        let (py, pm, pd, _) = astro::calendar(jd - n as f64);
        if tamil_month_of(py, pm, pd) != month {
            break;
        }
        n += 1;
    }
    n
}

/// Index (0 = Prabhava) of the Tamil year in the 60-year cycle. The reference table names 1927-28
/// Prabhava; the year changes on Chithirai 1.
pub fn tamil_year_index(y: i32, m: u32, d: u32) -> usize {
    // January to mid-April (Margazhi..Panguni) still belong to the year that began the April before.
    let start_year = if m <= 4 && tamil_month_of(y, m, d) >= 8 { y - 1 } else { y };
    (start_year - 1927).rem_euclid(60) as usize
}

/// The 60 Samvatsara (year) names, by language: English, Tamil, Hindi.
pub const YEAR_NAMES: [[&str; 60]; 3] = [
    ["Prabhava", "Vibhava", "Shukla", "Pramoda", "Prajapati", "Angirasa", "Shrimukha", "Bhava", "Yuva", "Dhatu", "Ishvara", "Bahudhanya", "Pramathi", "Vikrama", "Vrisha", "Chitrabhanu", "Subhanu", "Tarana", "Parthiva", "Vyaya", "Sarvajit", "Sarvadhari", "Virodhi", "Vikriti", "Khara", "Nandana", "Vijaya", "Jaya", "Manmatha", "Durmukhi", "Hevilambi", "Vilambi", "Vikari", "Sharvari", "Plava", "Shubhakrit", "Shobhakrit", "Krodhi", "Vishvavasu", "Parabhava", "Plavanga", "Kilaka", "Saumya", "Sadharana", "Virodhikrit", "Paridhavi", "Pramadicha", "Ananda", "Rakshasa", "Nala", "Pingala", "Kalayukti", "Siddharthi", "Raudri", "Durmati", "Dundubhi", "Rudhirodgari", "Raktakshi", "Krodhana", "Akshaya"],
    ["பிரபவ", "விபவ", "சுக்ல", "பிரமோதூத", "பிரஜோத்பத்தி", "ஆங்கீரச", "ஸ்ரீமுக", "பவ", "யுவ", "தாது", "ஈஸ்வர", "வெகுதானிய", "பிரமாதி", "விக்கிரம", "விஷு", "சித்திரபானு", "சுபானு", "தாரண", "பார்த்திப", "விய", "சர்வஜித்", "சர்வதாரி", "விரோதி", "விக்ருதி", "கர", "நந்தன", "விஜய", "ஜய", "மன்மத", "துர்முகி", "ஹேவிளம்பி", "விளம்பி", "விகாரி", "சார்வரி", "பிலவ", "சுபகிருது", "சோபகிருது", "குரோதி", "விசுவாவசு", "பராபவ", "பிலவங்க", "கீலக", "சௌமிய", "சாதாரண", "விரோதகிருது", "பரிதாபி", "பிரமாதீச", "ஆனந்த", "ராட்சச", "நள", "பிங்கள", "காளயுக்தி", "சித்தார்த்தி", "ரௌத்திரி", "துன்மதி", "துந்துபி", "ருத்ரோத்காரி", "ரக்தாட்சி", "குரோதன", "அட்சய"],
    ["प्रभव", "विभव", "शुक्ल", "प्रमोद", "प्रजापति", "अंगिरा", "श्रीमुख", "भाव", "युवा", "धाता", "ईश्वर", "बहुधान्य", "प्रमाथी", "विक्रम", "वृष", "चित्रभानु", "सुभानु", "तारण", "पार्थिव", "व्यय", "सर्वजित", "सर्वधारी", "विरोधी", "विकृति", "खर", "नंदन", "विजय", "जय", "मन्मथ", "दुर्मुख", "हेमलंबी", "विलंबी", "विकारी", "शार्वरी", "प्लव", "शुभकृत", "शोभकृत", "क्रोधी", "विश्वावसु", "पराभव", "प्लवंग", "कीलक", "सौम्य", "साधारण", "विरोधकृत", "परिधावी", "प्रमादी", "आनंद", "राक्षस", "नल", "पिंगल", "कालयुक्त", "सिद्धार्थी", "रौद्र", "दुर्मति", "दुंदुभि", "रुधिरोद्गारी", "रक्ताक्षी", "क्रोधन", "अक्षय"],
];

pub fn year_name(i: usize) -> &'static str {
    YEAR_NAMES[crate::i18n::li()][i % 60]
}

/// Calendar data for one civil day at a place.
#[allow(clippy::too_many_arguments)] // a date, a place and the calc settings — all independent inputs
pub fn day(y: i32, m: u32, d: u32, lat: f64, lon: f64, tz_name: &str, tz: f64, calc: Calc) -> Day {
    let tz = offset(tz_name, tz, y, m, d);
    let midnight = astro::julian_day(y, m, d, 0.0) - tz / 24.0;
    let (rise, set) = astro::sunrise_sunset(midnight, lat, lon);
    let (next_rise, _) = astro::sunrise_sunset(midnight + 1.0, lat, lon);
    let (_, prev_set) = astro::sunrise_sunset(midnight - 1.0, lat, lon);
    let at = rise.unwrap_or(midnight + 0.25);
    let moon = moon_sid(at, calc);
    let sun = sun_sid(at, calc);
    let weekday = ((astro::julian_day(y, m, d, 0.0) + 1.5).floor() as i64).rem_euclid(7) as usize;

    // Tamil date, as traditional Tamil almanacs (and the reference table) reckon it.
    let month = tamil_month_of(y, m, d);
    let tamil_day = tamil_day_of(y, m, d);
    let tamil_year = tamil_year_index(y, m, d);

    let part = |k: usize| -> Option<(f64, f64)> {
        let (r, s) = (rise?, set?);
        let p = (s - r) / 8.0;
        Some((r + (k - 1) as f64 * p, r + k as f64 * p))
    };
    let abhijit = rise.zip(set).map(|(r, s)| {
        let mu = (s - r) / 15.0;
        let mid = (r + s) / 2.0;
        (mid - mu / 2.0, mid + mu / 2.0)
    });
    let brahma = rise.zip(prev_set).map(|(r, ps)| {
        let mu = (r - ps) / 15.0;
        (r - 2.0 * mu, r - mu)
    });
    let _ = next_rise;
    let tithi = (norm360(moon - sun) / 12.0) as usize % 30;
    Day {
        year: y,
        month: m,
        day: d,
        tz,
        weekday,
        sunrise: rise,
        sunset: set,
        tithi,
        nakshatra: nakshatra_of(moon),
        yoga: (norm360(moon + sun) / NAK_SPAN) as usize % 27,
        tamil_month: month,
        tamil_day,
        tamil_year,
        rahu: part(RAHU[weekday]),
        yama: part(YAMA[weekday]),
        gulika: part(GULIKA[weekday]),
        abhijit,
        brahma,
        lunar_month: chandra_masa(at, calc),
        paksha: if tithi < 15 { 0 } else { 1 },
    }
}

/// The UT Julian day of the new moon at or before `jd` (Sun and Moon at the same sidereal longitude).
pub fn prev_new_moon(jd: f64, calc: Calc) -> f64 {
    // The Moon gains ~12.19°/day on the Sun (synodic); start from the current elongation, then refine.
    let elong = norm360(moon_sid(jd, calc) - sun_sid(jd, calc));
    let mut t = jd - elong / 12.19;
    for _ in 0..6 {
        let diff = moon_sid(t, calc) - sun_sid(t, calc);
        let diff = ((diff + 180.0).rem_euclid(360.0)) - 180.0; // nearest conjunction, in [-180,180)
        t -= diff / 12.19;
    }
    t
}

/// The amānta lunar month (0 = Chaitra … 11 = Phalguna). The month is named for the rāśi the Sun
/// *enters during* the lunation (the sankrānti it contains) — one sign past the Sun's sign at the
/// starting new moon, so Chaitra (0) is the month in which the Sun enters Meṣa. Regional conventions
/// vary; festival dates are estimates (see the festivals module / UI note).
pub fn chandra_masa(jd: f64, calc: Calc) -> usize {
    ((sun_sid(prev_new_moon(jd, calc), calc) / 30.0) as usize + 1) % 12
}

/// The Sun's true longitude in the Surya Siddhanta, the basis of traditional Tamil (Vakya)
/// almanacs, at UT Julian day `jd`. Mean motion: 4,320,000 revolutions in 1,577,917,828 civil days
/// from the Kali-yuga epoch (JD 588465.5, midnight at Ujjain, all mean planets at 0° Aries).
/// Apogee (mandocca) 77°17′ (its motion, 387 revolutions a kalpa, is negligible), and the
/// equation of centre from an epicycle of 14° at the apsides narrowing to 13°40′ at the quadrants.
pub fn ss_sun(jd: f64) -> f64 {
    const UJJAIN: f64 = 75.7683; // the Siddhanta's prime meridian, degrees east
    let days = jd + UJJAIN / 360.0 - 588465.5;
    let mean = norm360(360.0 * (days * 4_320_000.0 / 1_577_917_828.0).fract());
    let apogee = 77.0 + 17.0 / 60.0;
    let kendra = (mean - apogee).to_radians();
    let epicycle = 14.0 - (20.0 / 60.0) * kendra.sin().abs();
    let eq = (epicycle / 360.0 * kendra.sin()).asin().to_degrees();
    norm360(mean - eq)
}

/// Every day of a civil month.
pub fn month(y: i32, m: u32, lat: f64, lon: f64, tz_name: &str, tz: f64, calc: Calc) -> Vec<Day> {
    let days = match m {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        _ if (y % 4 == 0 && y % 100 != 0) || y % 400 == 0 => 29,
        _ => 28,
    };
    (1..=days).map(|d| day(y, m, d, lat, lon, tz_name, tz, calc)).collect()
}

/// Observance of the day from its sunrise tithi, if any (English key for `tr`).
pub fn observance(day: &Day) -> Option<&'static str> {
    match day.tithi {
        14 => Some("Pournami (full moon)"),
        29 => Some("Amavasya (new moon)"),
        10 | 25 => Some("Ekadashi"),
        12 | 27 => Some("Pradosham"),
        18 => Some("Sankatahara Chaturthi"),
        _ => None,
    }
}

/// Tamil month names (Tamil solar months), by language: English, Tamil, Hindi.
pub const TAMIL_MONTHS: [[&str; 12]; 3] = [
    ["Chithirai", "Vaikasi", "Aani", "Aadi", "Avani", "Purattasi", "Aippasi", "Karthigai", "Margazhi", "Thai", "Maasi", "Panguni"],
    ["சித்திரை", "வைகாசி", "ஆனி", "ஆடி", "ஆவணி", "புரட்டாசி", "ஐப்பசி", "கார்த்திகை", "மார்கழி", "தை", "மாசி", "பங்குனி"],
    ["चित्तिरै", "वैकासी", "आनि", "आडि", "आवणि", "पुरट्टासि", "ऐप्पसि", "कार्तिगै", "मार्गळि", "तै", "मासि", "पंगुनि"],
];

pub fn tamil_month_name(i: usize) -> &'static str {
    TAMIL_MONTHS[crate::i18n::li()][i % 12]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lunar_month_and_paksha() {
        let c = Calc::default();
        let d = day(2026, 11, 8, 13.0827, 80.2707, "Asia/Kolkata", 5.5, c);
        assert!(d.lunar_month <= 11);
        assert_eq!(d.paksha, if d.tithi < 15 { 0 } else { 1 });
        // chandra_masa is the sidereal sign of the Sun at the new moon it derives from.
        let at = d.sunrise.unwrap();
        let nm = prev_new_moon(at, c);
        assert_eq!(chandra_masa(at, c), ((sun_sid(nm, c) / 30.0) as usize + 1) % 12);
    }

    #[test]
    fn pongal_and_tamil_new_year_fall_on_day_one() {
        // Thai 1 (Pongal) was 15 Jan 2024 and Chithirai 1 (Tamil New Year) 14 Apr 2024 in Chennai.
        let calc = Calc::default();
        let pongal = day(2024, 1, 15, 13.0827, 80.2707, "Asia/Kolkata", 5.5, calc);
        assert_eq!((pongal.tamil_month, pongal.tamil_day), (9, 1));
        let new_year = day(2024, 4, 14, 13.0827, 80.2707, "Asia/Kolkata", 5.5, calc);
        assert_eq!((new_year.tamil_month, new_year.tamil_day), (0, 1));
        assert_eq!(day(2024, 4, 13, 13.0827, 80.2707, "Asia/Kolkata", 5.5, calc).tamil_month, 11);
    }

    #[test]
    fn tamil_years_follow_the_sixty_year_cycle() {
        // The reference table: 1927-28 Prabhava, 2020-21 Sharvari. Public almanacs: 2024-25 Krodhi.
        assert_eq!(YEAR_NAMES[0][tamil_year_index(1927, 6, 1)], "Prabhava");
        assert_eq!(YEAR_NAMES[0][tamil_year_index(2020, 6, 1)], "Sharvari");
        assert_eq!(YEAR_NAMES[0][tamil_year_index(2024, 5, 1)], "Krodhi");
        // Before Chithirai 1 the old year continues.
        assert_eq!(YEAR_NAMES[0][tamil_year_index(2024, 4, 10)], "Shobhakrit");
    }

    #[test]
    fn rahu_kalam_follows_the_weekday() {
        // 7 Jan 2024 was a Sunday: Rahu kalam is the 8th part, ending at sunset.
        let d = day(2024, 1, 7, 13.0827, 80.2707, "Asia/Kolkata", 5.5, Calc::default());
        assert_eq!(d.weekday, 0);
        let (_, end) = d.rahu.unwrap();
        assert!((end - d.sunset.unwrap()).abs() < 1e-9);
        // Monday: the 2nd part.
        let d = day(2024, 1, 8, 13.0827, 80.2707, "Asia/Kolkata", 5.5, Calc::default());
        let (s, _) = d.rahu.unwrap();
        let p = (d.sunset.unwrap() - d.sunrise.unwrap()) / 8.0;
        assert!((s - (d.sunrise.unwrap() + p)).abs() < 1e-9);
    }

    #[test]
    fn full_moon_is_marked() {
        // 25 Jan 2024 was a Pournami (Thai Poosam) in India.
        let d = day(2024, 1, 25, 13.0827, 80.2707, "Asia/Kolkata", 5.5, Calc::default());
        assert_eq!(observance(&d), Some("Pournami (full moon)"));
    }
}

#[cfg(test)]
mod reference_year_table {
    use super::*;

    /// Compares the Tamil month lengths with the reference Tamil year table
    /// (the reference Tamil year table, decoded to target/check/tamil_years.csv).
    /// `cargo test tamil_table_report -- --ignored --nocapture`.
    #[test]
    #[ignore]
    fn tamil_table_report() {
        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/target/check/tamil_years.csv");
        let text = std::fs::read_to_string(path).unwrap();
        let (mut years, mut ok_start, mut ok_all) = (0, 0, 0);
        for line in text.lines() {
            let f: Vec<&str> = line.split(';').collect();
            let date: Vec<u32> = f[1].split(' ').next().unwrap().split('/').map(|x| x.parse().unwrap()).collect();
            let (m, d, y) = (date[0], date[1], date[2] as i32);
            let want: Vec<u32> = f[3..15].iter().map(|x| x.parse().unwrap()).collect();
            // Our month lengths starting from the table's Chithirai 1.
            let mut jd = crate::astro::julian_day(y, m, d, 0.0);
            let mut got = Vec::new();
            let day_at = |jd: f64| {
                let (yy, mm, dd, _) = crate::astro::calendar(jd + 0.5);
                day(yy, mm, dd, 13.0827, 80.2707, "Asia/Kolkata", 5.5, Calc::default())
            };
            let first = day_at(jd);
            for _ in 0..12 {
                let mut n = 1;
                while day_at(jd + n as f64).tamil_day != 1 && n < 33 {
                    n += 1;
                }
                got.push(n);
                jd += n as f64;
            }
            years += 1;
            let start_ok = first.tamil_month == 0 && first.tamil_day == 1;
            ok_start += start_ok as usize;
            ok_all += (start_ok && got == want) as usize;
            if !(start_ok && got == want) {
                println!("{y}: start {}/{} want {:?} got {:?}", first.tamil_month, first.tamil_day, want, got);
            }
        }
        println!("years {years}: Chithirai 1 agrees {ok_start}, every month agrees {ok_all}");
    }

    /// Exact moment (UT JD) the Sun's sidereal longitude reaches `target` degrees, near `guess`.
    fn sankranti(target: f64, guess: f64, calc: Calc) -> f64 {
        let f = |t: f64| ((sun_sid(t, calc) - target + 540.0) % 360.0) - 180.0;
        let (mut lo, mut hi) = (guess - 3.0, guess + 3.0);
        for _ in 0..60 {
            let mid = 0.5 * (lo + hi);
            if f(mid) < 0.0 { lo = mid } else { hi = mid }
        }
        0.5 * (lo + hi)
    }

    /// Moment the Surya Siddhanta Sun reaches `target` degrees near `guess` (UT JD).
    fn ss_sankranti(target: f64, guess: f64) -> f64 {
        let f = |t: f64| ((super::ss_sun(t) - target + 540.0) % 360.0) - 180.0;
        let (mut lo, mut hi) = (guess - 3.0, guess + 3.0);
        for _ in 0..60 {
            let mid = 0.5 * (lo + hi);
            if f(mid) < 0.0 { lo = mid } else { hi = mid }
        }
        0.5 * (lo + hi)
    }

    /// Same as below, for the Surya Siddhanta Sun: where its sign change fell relative to
    /// The reference day 1, by month, and the best single cut-off.
    #[test]
    #[ignore]
    fn tamil_ss_report() {
        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/target/check/tamil_years.csv");
        let text = std::fs::read_to_string(path).unwrap();
        let mut deltas = Vec::new();
        for line in text.lines() {
            let f: Vec<&str> = line.split(';').collect();
            let date: Vec<u32> = f[1].split(' ').next().unwrap().split('/').map(|x| x.parse().unwrap()).collect();
            let mut start = crate::astro::julian_day(date[2] as i32, date[0], date[1], 0.0) - 5.5 / 24.0;
            let lens: Vec<f64> = f[3..15].iter().map(|x| x.parse().unwrap()).collect();
            for (k, len) in lens.iter().enumerate() {
                deltas.push((k, (ss_sankranti(30.0 * k as f64, start + 0.5) - start) * 24.0));
                start += len;
            }
        }
        for k in 0..12 {
            let v: Vec<f64> = deltas.iter().filter(|d| d.0 == k).map(|d| d.1).collect();
            let (lo, hi) = v.iter().fold((f64::MAX, f64::MIN), |(a, b), &x| (a.min(x), b.max(x)));
            println!("  month {k:2}: range {lo:6.2} .. {hi:6.2}");
        }
        // Drift: the latest same-day and earliest previous-day times, by decade.
        for dec in 0..10 {
            let v: Vec<f64> = deltas.iter().enumerate().filter(|(i, _)| i / 12 / 10 == dec).map(|(_, d)| d.1).collect();
            if v.is_empty() {
                continue;
            }
            let same = v.iter().filter(|h| **h >= 0.0).fold(f64::MIN, |a, &b| a.max(b));
            let prev = v.iter().filter(|h| **h < 0.0).fold(f64::MAX, |a, &b| a.min(b + 24.0));
            println!("  years {}..: same day up to {same:6.2} h, previous day from {prev:6.2} h", 1926 + dec * 10);
        }
        // Best cut c (hours after midnight): same day when 0 <= h < c, previous day when h >= c - 24 and h < 0.
        let mut best = (usize::MAX, 0.0);
        for i in 0..=240 {
            let c = i as f64 * 0.1;
            let bad = deltas.iter().filter(|(_, h)| !((*h >= c - 24.0 && *h < 0.0 && c < 24.0 && *h >= c - 24.0) || (*h >= 0.0 && *h < c)) || *h < c - 24.0).count();
            if bad < best.0 {
                best = (bad, c);
            }
        }
        println!("Surya Siddhanta: best cut {:.1} h after local midnight, {} of {} month starts disagree", best.1, best.0, deltas.len());
    }

    /// For every month start in the reference table: where the sign change fell, in hours after local
    /// midnight of the day the reference made day 1 (negative = the evening before). Prints the range
    /// that separates "same day" from "previous day", which is the reference cut-off.
    #[test]
    #[ignore]
    fn tamil_cutoff_report() {
        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/target/check/tamil_years.csv");
        let text = std::fs::read_to_string(path).unwrap();
        for calc in [Calc::default(), Calc { ayanamsa: Ayanamsa::Raman, ..Calc::default() }, Calc { ayanamsa: Ayanamsa::Krishnamurti, ..Calc::default() }] {
            let (mut latest_same, mut earliest_prev) = (f64::MIN, f64::MAX);
            let mut deltas = Vec::new();
            for line in text.lines() {
                let f: Vec<&str> = line.split(';').collect();
                let date: Vec<u32> = f[1].split(' ').next().unwrap().split('/').map(|x| x.parse().unwrap()).collect();
                let mut start = crate::astro::julian_day(date[2] as i32, date[0], date[1], 0.0) - 5.5 / 24.0; // local midnight, IST
                let lens: Vec<f64> = f[3..15].iter().map(|x| x.parse().unwrap()).collect();
                for (k, len) in lens.iter().enumerate() {
                    let t = sankranti(30.0 * k as f64, start + 0.5, calc);
                    let hours = (t - start) * 24.0;
                    deltas.push(hours);
                    if hours >= 0.0 { latest_same = latest_same.max(hours) } else { earliest_prev = earliest_prev.min(hours + 24.0) }
                    start += len;
                }
            }
            // By month: the spread of sign-change times (relative to the reference day 1 midnight).
            for k in 0..12 {
                let v: Vec<f64> = deltas.iter().skip(k).step_by(12).cloned().collect();
                let mean = v.iter().sum::<f64>() / v.len() as f64;
                let (lo, hi) = v.iter().fold((f64::MAX, f64::MIN), |(a, b), &x| (a.min(x), b.max(x)));
                println!("  month {k:2}: mean {mean:6.2} h, range {lo:6.2} .. {hi:6.2}");
            }
            let outside = deltas.iter().filter(|h| **h < -24.0 || **h >= 24.0).count();
            println!("{:?}: sign change on the start day up to {latest_same:.2} h after midnight; on the previous day from {earliest_prev:.2} h; {} boundaries, {outside} more than a day off", calc.ayanamsa, deltas.len());
        }
    }
}
