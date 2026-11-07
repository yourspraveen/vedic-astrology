//! Astronomy: Julian days, Sun, Moon, planets, lunar node, ayanamsa,
//! ascendant / midheaven and sunrise. All formulas are published ones
//! (Meeus "Astronomical Algorithms"; Schlyter's orbital elements) written
//! from the sheet in MECHANICS.md.

use std::f64::consts::PI;

pub const DEG: f64 = PI / 180.0;

pub fn norm360(x: f64) -> f64 {
    let r = x % 360.0;
    if r < 0.0 { r + 360.0 } else { r }
}

fn sind(x: f64) -> f64 {
    (x * DEG).sin()
}
fn cosd(x: f64) -> f64 {
    (x * DEG).cos()
}

/// Julian day for a Gregorian calendar date and a UT hour (may be fractional or out of range).
pub fn julian_day(year: i32, month: u32, day: u32, hour_ut: f64) -> f64 {
    let (mut y, mut m) = (year as f64, month as f64);
    if m <= 2.0 {
        y -= 1.0;
        m += 12.0;
    }
    let a = (y / 100.0).floor();
    let b = 2.0 - a + (a / 4.0).floor();
    (365.25 * (y + 4716.0)).floor() + (30.6001 * (m + 1.0)).floor() + day as f64 + b - 1524.5
        + hour_ut / 24.0
}

/// Calendar date (year, month, day, hour) of a Julian day.
pub fn calendar(jd: f64) -> (i32, u32, u32, f64) {
    let z = (jd + 0.5).floor();
    let f = jd + 0.5 - z;
    let a = if z < 2299161.0 {
        z
    } else {
        let alpha = ((z - 1867216.25) / 36524.25).floor();
        z + 1.0 + alpha - (alpha / 4.0).floor()
    };
    let b = a + 1524.0;
    let c = ((b - 122.1) / 365.25).floor();
    let d = (365.25 * c).floor();
    let e = ((b - d) / 30.6001).floor();
    let day_f = b - d - (30.6001 * e).floor() + f;
    let month = if e < 14.0 { e - 1.0 } else { e - 13.0 };
    let year = if month > 2.0 { c - 4716.0 } else { c - 4715.0 };
    let day = day_f.floor();
    (year as i32, month as u32, day as u32, (day_f - day) * 24.0)
}

/// Day of week, 0 = Sunday.
pub fn weekday(jd: f64) -> usize {
    (((jd + 1.5).floor() as i64).rem_euclid(7)) as usize
}

/// Approximate Delta T (TT - UT) in seconds, Espenak & Meeus polynomials.
pub fn delta_t(year: f64) -> f64 {
    let y = year;
    if (2005.0..2050.0).contains(&y) {
        let t = y - 2000.0;
        62.92 + 0.32217 * t + 0.005589 * t * t
    } else if (1986.0..2005.0).contains(&y) {
        let t = y - 2000.0;
        63.86 + 0.3345 * t - 0.060374 * t * t + 0.0017275 * t.powi(3) + 0.000651814 * t.powi(4)
            + 0.00002373599 * t.powi(5)
    } else if (1961.0..1986.0).contains(&y) {
        let t = y - 1975.0;
        45.45 + 1.067 * t - t * t / 260.0 - t.powi(3) / 718.0
    } else if (1941.0..1961.0).contains(&y) {
        let t = y - 1950.0;
        29.07 + 0.407 * t - t * t / 233.0 + t.powi(3) / 2547.0
    } else if (1920.0..1941.0).contains(&y) {
        let t = y - 1920.0;
        21.20 + 0.84493 * t - 0.076100 * t * t + 0.0020936 * t.powi(3)
    } else if (1900.0..1920.0).contains(&y) {
        let t = y - 1900.0;
        -2.79 + 1.494119 * t - 0.0598939 * t * t + 0.0061966 * t.powi(3) - 0.000197 * t.powi(4)
    } else {
        let u = (y - 1820.0) / 100.0;
        let base = -20.0 + 32.0 * u * u;
        if y >= 2050.0 { base - 0.5628 * (2150.0 - y) } else { base }
    }
}

fn year_of(jd: f64) -> f64 {
    2000.0 + (jd - 2451545.0) / 365.25
}

/// Julian Ephemeris Day (TT) from a UT Julian day.
pub fn jde(jd_ut: f64) -> f64 {
    jd_ut + delta_t(year_of(jd_ut)) / 86400.0
}

/// Geometric Sun: tropical longitude (mean equinox of date) and distance in AU. Meeus ch. 25.
pub fn sun_geometric(jde: f64) -> (f64, f64) {
    let t = (jde - 2451545.0) / 36525.0;
    let l0 = 280.46646 + 36000.76983 * t + 0.0003032 * t * t;
    let m = 357.52911 + 35999.05029 * t - 0.0001537 * t * t;
    let e = 0.016708634 - 0.000042037 * t - 0.0000001267 * t * t;
    let c = (1.914602 - 0.004817 * t - 0.000014 * t * t) * sind(m)
        + (0.019993 - 0.000101 * t) * sind(2.0 * m)
        + 0.000289 * sind(3.0 * m);
    let v = m + c;
    let r = 1.000001018 * (1.0 - e * e) / (1.0 + e * cosd(v));
    (norm360(l0 + c), r)
}

/// Apparent tropical Sun longitude (aberration applied).
pub fn sun_longitude(jde: f64) -> f64 {
    norm360(sun_geometric(jde).0 - 0.00569)
}

/// Accumulated general precession in longitude since J2000, degrees (IAU 1976).
fn general_precession(jd: f64) -> f64 {
    let t = (jd - 2451545.0) / 36525.0;
    (5029.0966 * t + 1.11113 * t * t - 0.000006 * t * t * t) / 3600.0
}

/// Ayanamsa from a reference epoch `t0` (JD) where it equalled `ayan_t0` degrees,
/// carried to `jde` by general precession.
pub fn ayanamsa_from(jde: f64, t0: f64, ayan_t0: f64) -> f64 {
    ayan_t0 + general_precession(jde) - general_precession(t0)
}

/// Lahiri (Chitrapaksha) ayanamsa: 22.46047 deg at J1900 (see MECHANICS.md).
pub fn lahiri_ayanamsa(jde: f64) -> f64 {
    ayanamsa_from(jde, 2415020.0, 22.46047)
}

/// Mean obliquity of the ecliptic. Meeus eq. 22.2.
pub fn obliquity(jde: f64) -> f64 {
    let t = (jde - 2451545.0) / 36525.0;
    23.0 + 26.0 / 60.0 + 21.448 / 3600.0 - (46.8150 * t + 0.00059 * t * t - 0.001813 * t.powi(3)) / 3600.0
}

/// Greenwich mean sidereal time in degrees. Meeus eq. 12.4.
pub fn gmst(jd_ut: f64) -> f64 {
    let t = (jd_ut - 2451545.0) / 36525.0;
    norm360(280.46061837 + 360.98564736629 * (jd_ut - 2451545.0) + 0.000387933 * t * t
        - t.powi(3) / 38710000.0)
}

/// Altitude of the Sun's centre in degrees.
fn sun_altitude(jd_ut: f64, lat: f64, lon: f64) -> f64 {
    let t = jde(jd_ut);
    let l = sun_longitude(t);
    let eps = obliquity(t);
    let ra = (sind(l) * cosd(eps)).atan2(cosd(l)) / DEG;
    let dec = (sind(eps) * sind(l)).asin() / DEG;
    let ha = gmst(jd_ut) + lon - ra;
    (sind(lat) * sind(dec) + cosd(lat) * cosd(dec) * cosd(ha)).asin() / DEG
}

/// Sunrise and sunset (UT Julian days) during the 24 h starting at `jd_start`.
/// Uses the upper limb with standard refraction (-0.833 deg).
pub fn sunrise_sunset(jd_start: f64, lat: f64, lon: f64) -> (Option<f64>, Option<f64>) {
    sunrise_sunset_at(jd_start, lat, lon, -0.833)
}

/// Sunrise and sunset for a chosen altitude of the Sun's centre `h0` (degrees).
pub fn sunrise_sunset_at(jd_start: f64, lat: f64, lon: f64, h0: f64) -> (Option<f64>, Option<f64>) {
    let f = |jd: f64| sun_altitude(jd, lat, lon) - h0;
    let step = 10.0 / 1440.0;
    let (mut rise, mut set) = (None, None);
    let mut a = jd_start;
    let mut fa = f(a);
    while a < jd_start + 1.0 {
        let b = a + step;
        let fb = f(b);
        if fa.signum() != fb.signum() {
            let (mut lo, mut hi, mut flo) = (a, b, fa);
            for _ in 0..40 {
                let mid = 0.5 * (lo + hi);
                let fm = f(mid);
                if fm.signum() == flo.signum() {
                    lo = mid;
                    flo = fm;
                } else {
                    hi = mid;
                }
            }
            let t = 0.5 * (lo + hi);
            if fa < 0.0 && rise.is_none() {
                rise = Some(t);
            } else if fa > 0.0 && set.is_none() {
                set = Some(t);
            }
        }
        a = b;
        fa = fb;
    }
    (rise, set)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn meeus_sun_example() {
        // Meeus example 25.a: 1992 Oct 13.0 TD, true longitude 199.90988
        let (l, _) = sun_geometric(2448908.5);
        assert!((l - 199.90988).abs() < 0.001, "{l}");
    }

    #[test]
    fn calendar_roundtrip() {
        let jd = julian_day(1987, 4, 10, 19.0 + 21.0 / 60.0);
        let (y, m, d, h) = calendar(jd);
        assert_eq!((y, m, d), (1987, 4, 10));
        assert!((h - 19.35).abs() < 1e-6);
        assert_eq!(weekday(julian_day(2024, 1, 1, 12.0)), 1); // Monday
    }

    #[test]
    fn lahiri_matches_1956_definition() {
        // The ayanamsa table entry (22.46047 at J1900) carried to 1956-03-21 must give
        // the Calendar Reform Committee value 23deg15'00.658" less that date's nutation
        // (23.250182778 - 0.004658035 = 23.245524743, mean equinox) within a few arcsec.
        let a = lahiri_ayanamsa(2435553.5);
        assert!((a - 23.245524743).abs() < 0.001, "{a}");
    }

    #[test]
    fn lahiri_2000() {
        let a = lahiri_ayanamsa(2451545.0);
        assert!((a - 23.857).abs() < 0.01, "{a}");
    }
}
