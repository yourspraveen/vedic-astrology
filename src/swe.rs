//! Swiss Ephemeris (compiled from vendor/swisseph, AGPL-3.0), used in its built-in mode:
//! the built-in Moshier theory, so no ephemeris data files are needed. Positions are apparent
//! (with nutation), matching the reference sayana longitudes.

use std::os::raw::{c_char, c_double, c_int};
use std::sync::Mutex;

unsafe extern "C" {
    fn swe_calc(tjd_et: c_double, ipl: c_int, iflag: c_int, xx: *mut c_double, serr: *mut c_char) -> c_int;
    fn swe_houses(tjd_ut: c_double, geolat: c_double, geolon: c_double, hsys: c_int, cusps: *mut c_double, ascmc: *mut c_double) -> c_int;
}

pub const SUN: i32 = 0;
pub const MOON: i32 = 1;
pub const MERCURY: i32 = 2;
pub const VENUS: i32 = 3;
pub const MARS: i32 = 4;
pub const JUPITER: i32 = 5;
pub const SATURN: i32 = 6;
pub const MEAN_NODE: i32 = 10;
pub const TRUE_NODE: i32 = 11;
const ECL_NUT: i32 = -1;

const SEFLG_MOSEPH: i32 = 4;
const SEFLG_SPEED: i32 = 256;

/// The library keeps global state, so calls are serialised.
static LOCK: Mutex<()> = Mutex::new(());

/// Apparent tropical ecliptic longitude (degrees) and daily speed, for a Terrestrial Time Julian day.
pub fn calc(tjd_et: f64, body: i32) -> (f64, f64) {
    let _guard = LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let mut xx = [0.0f64; 6];
    let mut serr = [0 as c_char; 256];
    // SAFETY: xx holds the 6 doubles and serr the 256 bytes the library writes.
    unsafe {
        swe_calc(tjd_et, body, SEFLG_MOSEPH | SEFLG_SPEED, xx.as_mut_ptr(), serr.as_mut_ptr());
    }
    (xx[0].rem_euclid(360.0), xx[3])
}

/// Nutation in longitude (degrees) for a Terrestrial Time Julian day.
pub fn nutation(tjd_et: f64) -> f64 {
    let _guard = LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let mut xx = [0.0f64; 6];
    let mut serr = [0 as c_char; 256];
    // SAFETY: as above.
    unsafe {
        swe_calc(tjd_et, ECL_NUT, SEFLG_MOSEPH, xx.as_mut_ptr(), serr.as_mut_ptr());
    }
    xx[2]
}

/// Apparent tropical ascendant and midheaven for a UT Julian day and place (east longitude positive).
pub fn asc_mc(jd_ut: f64, lat: f64, lon: f64) -> (f64, f64) {
    let _guard = LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let mut cusps = [0.0f64; 13];
    let mut ascmc = [0.0f64; 10];
    // SAFETY: the library writes 13 cusps and 10 angles.
    unsafe {
        swe_houses(jd_ut, lat, lon, b'O' as c_int, cusps.as_mut_ptr(), ascmc.as_mut_ptr());
    }
    (ascmc[0].rem_euclid(360.0), ascmc[1].rem_euclid(360.0))
}
