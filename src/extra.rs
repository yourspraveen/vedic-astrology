//! Extra calculations for the full report: KP lords, Ashtakavarga, karakas, yogi point,
//! arudha padas, friendships, natural benefics, Kali day count and sign periods.

use crate::astro::norm360;
use crate::vedic::*;

/// Rahu dosha or Ketu dosha, judged separately: the node in house
/// 1, 2, 4, 7, 8 or 12 from the ascendant, its effect reduced when
/// Jupiter aspects it (RAHUJUPASPECT) or sits with it (RAHUJUPASSOCIATION).
pub struct NodeDosha {
    pub house: usize,
    pub present: bool,
    pub jupiter_aspects: bool,
    pub jupiter_with: bool,
}

pub fn node_dosha(c: &Chart, node: Graha) -> NodeDosha {
    let house = c.house_of(node);
    let from_jup = (c.get(node).sign() + 12 - c.get(Graha::Jupiter).sign()) % 12 + 1;
    NodeDosha {
        house,
        present: DOSHA_HOUSES.contains(&house),
        // Jupiter's full aspects fall on the 5th, 7th and 9th signs from it.
        jupiter_aspects: matches!(from_jup, 5 | 7 | 9),
        jupiter_with: from_jup == 1,
    }
}

/// KP lords of a longitude: star lord, sub-lord and sub-sub-lord. Each nakshatra is split in
/// the Vimshottari proportions starting from its own lord, and each sub again the same way.
pub fn kp_lords(lon: f64) -> (Graha, Graha, Graha) {
    let star = nakshatra_of(lon) % 9;
    let within = norm360(lon) % NAK_SPAN;
    let split = |start_idx: usize, span: f64, pos: f64| -> (usize, f64, f64) {
        let mut acc = 0.0;
        for k in 0..9 {
            let idx = (start_idx + k) % 9;
            let len = span * DASHA[idx].1 / 120.0;
            if pos < acc + len || k == 8 {
                return (idx, pos - acc, len);
            }
            acc += len;
        }
        (start_idx, 0.0, span)
    };
    let (sub, pos_in_sub, sub_len) = split(star, NAK_SPAN, within);
    let (subsub, _, _) = split(sub, sub_len, pos_in_sub);
    (DASHA[star].0, DASHA[sub].0, DASHA[subsub].0)
}

/// Benefic points (Bhinnashtakavarga, BPHS): for each of the seven planets, the houses counted
/// from each contributor (Sun, Moon, Mars, Mercury, Jupiter, Venus, Saturn, Ascendant).
/// Totals 48, 49, 39, 54, 56, 52, 39 = 337.
const BAV: [[&[u8]; 8]; 7] = [
    [&[1, 2, 4, 7, 8, 9, 10, 11], &[3, 6, 10, 11], &[1, 2, 4, 7, 8, 9, 10, 11], &[3, 5, 6, 9, 10, 11, 12], &[5, 6, 9, 11], &[6, 7, 12], &[1, 2, 4, 7, 8, 9, 10, 11], &[3, 4, 6, 10, 11, 12]],
    [&[3, 6, 7, 8, 10, 11], &[1, 3, 6, 7, 10, 11], &[2, 3, 5, 6, 9, 10, 11], &[1, 3, 4, 5, 7, 8, 10, 11], &[1, 4, 7, 8, 10, 11, 12], &[3, 4, 5, 7, 9, 10, 11], &[3, 5, 6, 11], &[3, 6, 10, 11]],
    [&[3, 5, 6, 10, 11], &[3, 6, 11], &[1, 2, 4, 7, 8, 10, 11], &[3, 5, 6, 11], &[6, 10, 11, 12], &[6, 8, 11, 12], &[1, 4, 7, 8, 9, 10, 11], &[1, 3, 6, 10, 11]],
    [&[5, 6, 9, 11, 12], &[2, 4, 6, 8, 10, 11], &[1, 2, 4, 7, 8, 9, 10, 11], &[1, 3, 5, 6, 9, 10, 11, 12], &[6, 8, 11, 12], &[1, 2, 3, 4, 5, 8, 9, 11], &[1, 2, 4, 7, 8, 9, 10, 11], &[1, 2, 4, 6, 8, 10, 11]],
    [&[1, 2, 3, 4, 7, 8, 9, 10, 11], &[2, 5, 7, 9, 11], &[1, 2, 4, 7, 8, 10, 11], &[1, 2, 4, 5, 6, 9, 10, 11], &[1, 2, 3, 4, 7, 8, 10, 11], &[2, 5, 6, 9, 10, 11], &[3, 5, 6, 12], &[1, 2, 4, 5, 6, 7, 9, 10, 11]],
    [&[8, 11, 12], &[1, 2, 3, 4, 5, 8, 9, 11, 12], &[3, 5, 6, 9, 11, 12], &[3, 5, 6, 9, 11], &[5, 8, 9, 10, 11], &[1, 2, 3, 4, 5, 8, 9, 10, 11], &[3, 4, 5, 8, 9, 10, 11], &[1, 2, 3, 4, 5, 8, 9, 11]],
    [&[1, 2, 4, 7, 8, 10, 11], &[3, 6, 11], &[3, 5, 6, 10, 11, 12], &[6, 8, 9, 10, 11, 12], &[5, 6, 11, 12], &[6, 11, 12], &[3, 5, 6, 11], &[1, 3, 4, 6, 10, 11]],
];

/// Bhinnashtakavarga points per sign for the seven planets (index = Graha 0..6).
pub fn ashtakavarga(c: &Chart) -> [[u8; 12]; 7] {
    let mut out = [[0u8; 12]; 7];
    let from: [usize; 8] = [
        c.get(Graha::Sun).sign(),
        c.get(Graha::Moon).sign(),
        c.get(Graha::Mars).sign(),
        c.get(Graha::Mercury).sign(),
        c.get(Graha::Jupiter).sign(),
        c.get(Graha::Venus).sign(),
        c.get(Graha::Saturn).sign(),
        c.asc_sign(),
    ];
    for p in 0..7 {
        for (k, &f) in from.iter().enumerate() {
            for &h in BAV[p][k] {
                out[p][(f + h as usize - 1) % 12] += 1;
            }
        }
    }
    out
}

pub fn sarvashtakavarga(bav: &[[u8; 12]; 7]) -> [u16; 12] {
    let mut s = [0u16; 12];
    for row in bav {
        for (i, v) in row.iter().enumerate() {
            s[i] += *v as u16;
        }
    }
    s
}

/// Chara karakas (seven-karaka scheme): Sun..Saturn ordered by degrees within their sign,
/// highest first. [0] is the Atma karaka, [1] the Amatya karaka.
pub fn chara_karakas(c: &Chart) -> Vec<Graha> {
    let mut v: Vec<Graha> = GRAHAS[..7].to_vec();
    v.sort_by(|a, b| (norm360(c.get(*b).lon) % 30.0).total_cmp(&(norm360(c.get(*a).lon) % 30.0)));
    v
}

/// Yogi point (Sun + Moon + 93°20') and avayogi point (yogi + 186°40').
pub fn yogi_points(c: &Chart) -> (f64, f64) {
    let yogi = norm360(c.get(Graha::Sun).lon + c.moon() + 93.0 + 20.0 / 60.0);
    (yogi, norm360(yogi + 186.0 + 40.0 / 60.0))
}

/// Arudha (pada) sign of house `h` (1..12): count from the house to its lord, the same count again
/// from the lord; if that lands in the house itself, take the 10th from there. The reference program does not
/// move a pada that lands in the 7th (sample: Dhana pada Aquarius for a Cancer lagna, Sun in Scorpio).
pub fn arudha(c: &Chart, h: usize) -> usize {
    let house_sign = (c.asc_sign() + h - 1) % 12;
    let lord_sign = c.get(SIGN_LORD[house_sign]).sign();
    let n = (lord_sign + 12 - house_sign) % 12;
    let mut pada = (lord_sign + n) % 12;
    let rel = (pada + 12 - house_sign) % 12;
    if rel == 0 {
        pada = (pada + 9) % 12;
    }
    pada
}

/// Avayogi star: the 6th star counted from the yogi star (as in the reference reports).
pub fn avayogi_star(yogi: f64) -> usize {
    (nakshatra_of(yogi) + 5) % 27
}

/// Temporary (tatkalika) friendship: planets in the 2nd, 3rd, 4th, 10th, 11th or 12th from each other.
pub fn temporary_friend(c: &Chart, a: Graha, b: Graha) -> bool {
    let d = (c.get(b).sign() + 12 - c.get(a).sign()) % 12 + 1;
    matches!(d, 2 | 3 | 4 | 10 | 11 | 12)
}

/// Five-fold (panchadha) relationship of `a` towards `b` (English key for `tr`).
pub fn fivefold(c: &Chart, a: Graha, b: Graha) -> &'static str {
    match (relation(a, b), temporary_friend(c, a, b)) {
        (Rel::Friend, true) => "Great friend",
        (Rel::Friend, false) | (Rel::Enemy, true) => "Neutral",
        (Rel::Neutral, true) => "Friend",
        (Rel::Neutral, false) => "Enemy",
        (Rel::Enemy, false) => "Great enemy",
    }
}

/// The Moon has paksha bala from Shukla Shashthi up to (not including) Krishna Shashthi, as in
/// the reference reports (a Krishna Shashthi birth is printed "without Paksha Bala").
pub fn moon_has_paksha_bala(c: &Chart) -> bool {
    let tithi = (norm360(c.moon() - c.get(Graha::Sun).lon) / 12.0).floor() as usize;
    (5..20).contains(&tithi)
}

/// Mercury joined by a malefic (Sun, Mars, Saturn, Rahu or Ketu in its sign).
pub fn mercury_afflicted(c: &Chart) -> bool {
    use Graha::*;
    [Sun, Mars, Saturn, Rahu, Ketu].iter().any(|&m| c.get(m).sign() == c.get(Mercury).sign())
}

/// Natural benefic in this chart: Jupiter and Venus; the Moon with paksha bala; Mercury unless it
/// shares a sign with a malefic. Sun, Mars, Saturn, Rahu and Ketu are malefic.
pub fn is_benefic(c: &Chart, g: Graha) -> bool {
    use Graha::*;
    match g {
        Jupiter | Venus => true,
        Moon => moon_has_paksha_bala(c),
        Mercury => !mercury_afflicted(c),
        _ => false,
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Nature {
    Benefic,
    Neutral,
    Malefic,
}

/// Functional nature from the houses a planet rules for this ascendant. Rules (matching all 14
/// rows of the two sample reports' "Benefic / malefic analysis based on lordship" tables):
/// owning 1, 5 or 9 makes it benefic; otherwise 3/6/11 count −1, a kendra (4/7/10) counts +½
/// for natural malefics and −½ for natural benefics (Moon, Mercury, Jupiter, Venus), the 8th
/// counts −½ and the 2nd/12th 0.
pub fn lordship_nature(asc_sign: usize, g: Graha) -> (Vec<usize>, Nature) {
    let houses: Vec<usize> = (1..=12).filter(|h| SIGN_LORD[(asc_sign + h - 1) % 12] == g).collect();
    if houses.is_empty() {
        return (houses, Nature::Neutral);
    }
    if houses.iter().any(|h| matches!(h, 1 | 5 | 9)) {
        return (houses, Nature::Benefic);
    }
    let natural_benefic = matches!(g, Graha::Moon | Graha::Mercury | Graha::Jupiter | Graha::Venus);
    let score: f64 = houses
        .iter()
        .map(|h| match h {
            3 | 6 | 11 => -1.0,
            4 | 7 | 10 => if natural_benefic { -0.5 } else { 0.5 },
            8 => -0.5,
            _ => 0.0,
        })
        .sum();
    let nature = if score > 0.0 {
        Nature::Benefic
    } else if score < 0.0 {
        Nature::Malefic
    } else {
        Nature::Neutral
    };
    (houses, nature)
}

/// Strength of a planet by its sign: 1 strong (exalted, own), -1 weak (debilitated, enemy), 0 average.
/// Strength used to pick the good or bad dasa reading, as the reference reports do (records
/// checked on 11 dasas of the two English sample reports). The seven planets are strong in their
/// exaltation, own or a naturally friendly sign and weak otherwise (a neutral sign counts as weak).
/// Rahu and Ketu are strong in a kendra or trikona (1, 4, 5, 7, 9, 10), weak in 6, 8, 12 (estimated)
/// and neither in 2, 3, 11.
pub fn dasa_strength(c: &Chart, g: Graha) -> i32 {
    if matches!(g, Graha::Rahu | Graha::Ketu) {
        return match c.house_of(g) {
            1 | 4 | 5 | 7 | 9 | 10 => 1,
            6 | 8 | 12 => -1,
            _ => 0,
        };
    }
    match dignity(g, c.get(g).sign()) {
        "Exalted" | "Own sign" | "Friendly" => 1,
        _ => -1,
    }
}

/// Upper ages of the reference life stages for sub-period (bhukti) readings: childhood, youth,
/// adulthood, maturity, old age. The reference program picks one of five sub-period texts by the person's age at
/// the END of the sub-period (the reference age-band records, matched on 27 sub-periods of the two English
/// samples: the switches fall at 13 and 21). 37 is the next break in the reference transit bands;
/// 65 comes from the reference age-band header. Both later breaks are estimates.
pub const LIFE_STAGE_ENDS: [f64; 4] = [13.0, 21.0, 37.0, 65.0];

/// Life stage (0 childhood .. 4 old age) for an age in years.
pub fn life_stage(age: f64) -> usize {
    LIFE_STAGE_ENDS.iter().take_while(|&&end| age >= end).count()
}

pub fn sign_strength(c: &Chart, g: Graha) -> i32 {
    match dignity(g, c.get(g).sign()) {
        "Exalted" | "Own sign" => 1,
        "Debilitated" | "Enemy" => -1,
        _ => 0,
    }
}

/// Lords of the 3rd, 5th and 7th stars from the birth star (Vipat, Pratyak, Naidhana taras),
/// whose dasas are less favourable. Matches the sample report: Magha → Sun, Mars, Jupiter.
pub fn adverse_tara_lords(moon: f64) -> [Graha; 3] {
    let n = nakshatra_of(moon);
    [DASHA[(n + 2) % 27 % 9].0, DASHA[(n + 4) % 27 % 9].0, DASHA[(n + 6) % 27 % 9].0]
}

/// Kali day count (Kalidina sankhya): days since the start of the Kali Yuga.
pub fn kalidina(jd_ut: f64, tz: f64) -> i64 {
    (jd_ut + tz / 24.0 + 0.5).floor() as i64 - 588466
}

/// Signs a slow planet passes through between `from` and `to` (UT JD): (start, end, sign).
pub fn sign_periods(g: Graha, from: f64, to: f64, calc: Calc) -> Vec<(f64, f64, usize)> {
    let mut out = Vec::new();
    let mut cur = sidereal(g, from, calc).sign();
    let mut start = from;
    let mut t = from;
    while t < to {
        let next = t + 1.0;
        let s = sidereal(g, next, calc).sign();
        if s != cur {
            let (mut lo, mut hi) = (t, next);
            for _ in 0..12 {
                let mid = 0.5 * (lo + hi);
                if sidereal(g, mid, calc).sign() == cur {
                    lo = mid
                } else {
                    hi = mid
                }
            }
            out.push((start, hi, cur));
            cur = s;
            start = hi;
        }
        t = next;
    }
    out.push((start, to, cur));
    out
}

/// Bhava (1..12) holding a longitude, by Sripati sandhis.
pub fn bhava_of(bhavas: &[(f64, f64)], lon: f64) -> usize {
    for i in 0..12 {
        let start = bhavas[i].1;
        let end = bhavas[(i + 1) % 12].1;
        if norm360(lon - start) < norm360(end - start) {
            return i + 1;
        }
    }
    1
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ashtakavarga_total_is_337() {
        let c = Chart::compute(2451545.0, 13.0, 80.0, 5.5, Calc::default());
        let bav = ashtakavarga(&c);
        let totals: Vec<u32> = bav.iter().map(|r| r.iter().map(|&x| x as u32).sum()).collect();
        assert_eq!(totals, vec![48, 49, 39, 54, 56, 52, 39]);
        assert_eq!(sarvashtakavarga(&bav).iter().sum::<u16>(), 337);
    }

    #[test]
    fn kp_first_sub_is_star_lord() {
        // 0°10' Aries: Ketu star, first sub (Ketu, 7/120 of 13°20' = 46'40") is Ketu itself.
        let (s, sub, _) = kp_lords(10.0 / 60.0);
        assert_eq!((s, sub), (Graha::Ketu, Graha::Ketu));
        let (_, sub, _) = kp_lords(1.0); // past 46'40" -> Venus sub
        assert_eq!(sub, Graha::Venus);
    }

    #[test]
    fn kalidina_matches_known_day() {
        // 2019-09-21 local noon: Kalidina 1870282.
        let jd = crate::astro::julian_day(2019, 9, 21, 12.0 + 5.0);
        assert_eq!(kalidina(jd, -5.0), 1870282);
    }

    /// Tropical (sayana) longitudes printed in the two reference sample reports.
    /// Order: Lagna, Moon, Sun, Mercury, Venus, Mars, Jupiter, Saturn, Node.
    #[allow(clippy::type_complexity)] // a fixed-shape fixture row: label, lat, lon, tz, and 9 wanted longitudes
    pub(super) fn sample_cases() -> Vec<(&'static str, f64, f64, f64, f64, [f64; 9])> {
        let d = |d: f64, m: f64, s: f64| d + m / 60.0 + s / 3600.0;
        vec![
            (
                "2012-12-04 20:06 UTC-6, 89°15'W 40°00'N",
                crate::astro::julian_day(2012, 12, 4, 20.0 + 6.0 / 60.0 + 6.0),
                40.0,
                -(89.0 + 15.0 / 60.0),
                -6.0,
                [d(121., 49., 10.), d(144., 50., 24.), d(253., 20., 29.), d(232., 55., 8.), d(226., 9., 27.), d(283., 43., 1.), d(71., 1., 23.), d(217., 3., 17.), d(235., 1., 20.)],
            ),
            (
                "2019-09-21 10:11 UTC-5, 77°22'W 37°33'N",
                crate::astro::julian_day(2019, 9, 21, 10.0 + 11.0 / 60.0 + 5.0),
                37.0 + 33.0 / 60.0,
                -(77.0 + 22.0 / 60.0),
                -5.0,
                [d(229., 2., 6.), d(82., 30., 44.), d(178., 20., 36.), d(192., 15., 0.), d(188., 46., 11.), d(171., 56., 28.), d(256., 59., 29.), d(283., 55., 6.), d(103., 36., 24.)],
            ),
        ]
    }

    fn diff(a: f64, b: f64) -> f64 {
        let x = (a - b).rem_euclid(360.0);
        if x > 180.0 { x - 360.0 } else { x }
    }

    /// With Swiss Ephemeris every sayana value and the printed
    /// Chitra Paksha ayanamsa must match the reference sample reports to within a few arcseconds.
    #[test]
    fn swiss_engine_matches_reference_samples() {
        let ayan = [24.0 + 2.0 / 60.0 + 28.0 / 3600.0, 24.0 + 7.0 / 60.0 + 40.0 / 3600.0];
        for (k, (label, jd, lat, lon, tz, want)) in sample_cases().into_iter().enumerate() {
            let c = Chart::compute(jd, lat, lon, tz, Calc::default());
            let got = [
                c.asc + c.ayanamsa,
                c.moon() + c.ayanamsa,
                c.get(Graha::Sun).lon + c.ayanamsa,
                c.get(Graha::Mercury).lon + c.ayanamsa,
                c.get(Graha::Venus).lon + c.ayanamsa,
                c.get(Graha::Mars).lon + c.ayanamsa,
                c.get(Graha::Jupiter).lon + c.ayanamsa,
                c.get(Graha::Saturn).lon + c.ayanamsa,
                c.get(Graha::Rahu).lon + c.ayanamsa,
            ];
            println!("{label}  ayanamsa diff {:+.1}\"", diff(c.ayanamsa, ayan[k]) * 3600.0);
            for i in 0..9 {
                let e = diff(got[i], want[i]) * 3600.0;
                println!("  body {i}: diff {e:+.1}\"");
                assert!(e.abs() < 6.0, "body {i} off by {e:.1} arcsec");
            }
            assert!(diff(c.ayanamsa, ayan[k]).abs() * 3600.0 < 6.0);
        }
    }

    /// The two sample reports' lordship tables: Cancer and Libra ascendants.
    #[test]
    fn lordship_nature_matches_samples() {
        use Graha::*;
        use Nature::*;
        let cancer = [(Moon, Benefic), (Sun, Neutral), (Mercury, Malefic), (Venus, Malefic), (Mars, Benefic), (Jupiter, Benefic), (Saturn, Neutral)];
        for (g, want) in cancer {
            assert_eq!(lordship_nature(3, g).1, want, "Cancer {g:?}");
        }
        let libra = [(Moon, Malefic), (Sun, Malefic), (Mercury, Benefic), (Venus, Benefic), (Mars, Benefic), (Jupiter, Malefic), (Saturn, Benefic)];
        for (g, want) in libra {
            assert_eq!(lordship_nature(6, g).1, want, "Libra {g:?}");
        }
        // Magha (index 9): adverse dasas Sun, Mars, Jupiter (sample report's star remedies).
        assert_eq!(adverse_tara_lords(9.0 * NAK_SPAN + 1.0), [Sun, Mars, Jupiter]);
    }

    #[test]
    fn ayanamsa_2019_matches_sample_report() {
        // 2019-09-21 10:11 at UTC-5: Chitra paksha 24°07'40" in the person's sample report.
        let jd = crate::astro::julian_day(2019, 9, 21, 10.0 + 11.0 / 60.0 + 5.0);
        let a = crate::astro::lahiri_ayanamsa(crate::astro::jde(jd));
        let want = 24.0 + 7.0 / 60.0 + 40.0 / 3600.0;
        assert!((a - want).abs() < 20.0 / 3600.0, "{a} vs {want}");
    }
}

#[cfg(test)]
mod sunrise_probe {
    /// Prints sunrise/sunset for several conventions next to the reference printed times.
    #[test]
    fn sunrise_conventions_vs_samples() {
        use crate::astro;
        // (label, date, tz, lat, lon, reference sunrise, sunset in local hours)
        let cases = [
            ("Soverna 2012-12-04", (2012, 12, 4), -6.0, 40.0, -(89.0 + 15.0 / 60.0), 7.0 + 4.0 / 60.0, 16.0 + 32.0 / 60.0),
            ("Nyra 2019-09-21", (2019, 9, 21), -5.0, 37.0 + 33.0 / 60.0, -(77.0 + 22.0 / 60.0), 5.0 + 56.0 / 60.0, 18.0 + 9.0 / 60.0),
        ];
        for (label, (y, m, d), tz, lat, lon, want_rise, want_set) in cases {
            let midnight = astro::julian_day(y, m, d, 0.0) - tz / 24.0;
            println!("{label}: reference {:.3} / {:.3}", want_rise, want_set);
            for (name, h0) in [("upper limb + refraction", -0.833), ("centre + refraction", -0.5667), ("centre, no refraction", 0.0), ("upper limb, no refraction", -0.2667)] {
                let (r, s) = astro::sunrise_sunset_at(midnight, lat, lon, h0);
                let loc = |j: Option<f64>| j.map(|j| (j - midnight) * 24.0).unwrap_or(f64::NAN);
                println!("  {name:<26} rise {:.3} ({:+.1} min)  set {:.3} ({:+.1} min)", loc(r), (loc(r) - want_rise) * 60.0, loc(s), (loc(s) - want_set) * 60.0);
                if h0 == -0.833 {
                    // The app's rule: the reference prints whole minutes, so allow just over one minute.
                    assert!(((loc(r) - want_rise) * 60.0).abs() < 1.1 && ((loc(s) - want_set) * 60.0).abs() < 1.1, "{label}");
                }
            }
        }
    }
}

#[cfg(test)]
mod life_stage_rule {
    use super::*;

    /// Age-band file used for each sub-period of the samples, by the sub-period's end date.
    #[test]
    fn life_stage_matches_samples() {
        let soverna = crate::astro::julian_day(2012, 12, 5, 2.1);
        let nyra = crate::astro::julian_day(2019, 9, 21, 15.2);
        let obs: [(f64, (i32, u32, u32), usize); 16] = [
            (soverna, (2022, 11, 3), 0), (soverna, (2023, 11, 4), 0), (soverna, (2025, 7, 4), 0),
            (soverna, (2026, 9, 4), 1), (soverna, (2029, 9, 3), 1), (soverna, (2032, 5, 4), 1),
            (soverna, (2035, 7, 5), 2), (soverna, (2038, 5, 5), 2), (soverna, (2044, 7, 4), 2),
            (nyra, (2026, 10, 8), 0), (nyra, (2029, 3, 3), 0), (nyra, (2032, 1, 8), 0),
            (nyra, (2034, 7, 27), 1), (nyra, (2038, 8, 14), 1), (nyra, (2039, 7, 9), 1),
            (nyra, (2042, 1, 26), 2),
        ];
        for (birth, (y, m, d), want) in obs {
            let end = crate::astro::julian_day(y, m, d, 12.0);
            assert_eq!(life_stage((end - birth) / 365.25), want, "{y}-{m}-{d}");
        }
    }
}

#[cfg(test)]
mod dasa_rule {
    use super::*;

    /// The reference choice of the good or bad dasa reading in the two English sample reports
    /// (matched against the reference good/bad records): 1 good, -1 bad, 0 neither.
    #[test]
    fn dasa_strength_matches_samples() {
        let want: [&[(Graha, i32)]; 2] = [
            &[(Graha::Venus, 1), (Graha::Sun, 1), (Graha::Moon, 1), (Graha::Mars, 1), (Graha::Rahu, 1)],
            &[(Graha::Mars, 1), (Graha::Rahu, 1), (Graha::Jupiter, 1), (Graha::Saturn, -1), (Graha::Mercury, 1), (Graha::Ketu, 0)],
        ];
        for ((label, jd, lat, lon, tz, _), w) in super::tests::sample_cases().into_iter().zip(want) {
            let c = Chart::compute(jd, lat, lon, tz, Calc::default());
            for &(g, v) in w {
                assert_eq!(dasa_strength(&c, g), v, "{label}: {g:?}");
            }
        }
    }
}

#[cfg(test)]
mod transit_rule {
    use super::*;

    /// The reference transit records are numbered by house from the
    /// birth Moon sign. Both samples print 5 (Soverna, Moon in Leo) and 8 (Nyra, Moon in Taurus) for
    /// all three planets, which places their print date around late December 2019.
    #[test]
    fn transit_houses_match_samples() {
        let when = crate::astro::julian_day(2019, 12, 25, 12.0);
        for ((label, jd, lat, lon, tz, _), want) in super::tests::sample_cases().into_iter().zip([5, 8]) {
            let c = Chart::compute(jd, lat, lon, tz, Calc::default());
            let msign = sign_of(c.moon());
            for g in [Graha::Sun, Graha::Jupiter, Graha::Saturn] {
                let house = (sidereal(g, when, Calc::default()).sign() + 12 - msign) % 12 + 1;
                assert_eq!(house, want, "{label}: {g:?}");
            }
        }
    }
}
