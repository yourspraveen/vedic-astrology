//! Birth-place lookup: "City, State, Country" → latitude, longitude and IANA time zone (Open-Meteo's
//! free geocoding service, asked only when the person presses Fetch), and the UTC offset that zone
//! had at the moment of birth (the bundled tz database, so daylight saving and historical changes
//! are counted).

use chrono::{NaiveDate, Offset, TimeZone};
use chrono_tz::Tz;
use std::time::Duration;

/// Preset places: name, latitude, longitude, standard offset (hours), IANA time zone.
pub const PLACES: [(&str, f64, f64, f64, &str); 24] = [
    ("Chennai, India", 13.0827, 80.2707, 5.5, "Asia/Kolkata"),
    ("Mumbai, India", 19.0760, 72.8777, 5.5, "Asia/Kolkata"),
    ("New Delhi, India", 28.6139, 77.2090, 5.5, "Asia/Kolkata"),
    ("Kolkata, India", 22.5726, 88.3639, 5.5, "Asia/Kolkata"),
    ("Bengaluru, India", 12.9716, 77.5946, 5.5, "Asia/Kolkata"),
    ("Hyderabad, India", 17.3850, 78.4867, 5.5, "Asia/Kolkata"),
    ("Thiruvananthapuram, India", 8.5241, 76.9366, 5.5, "Asia/Kolkata"),
    ("Kochi, India", 9.9312, 76.2673, 5.5, "Asia/Kolkata"),
    ("Pune, India", 18.5204, 73.8567, 5.5, "Asia/Kolkata"),
    ("Ahmedabad, India", 23.0225, 72.5714, 5.5, "Asia/Kolkata"),
    ("Jaipur, India", 26.9124, 75.7873, 5.5, "Asia/Kolkata"),
    ("Lucknow, India", 26.8467, 80.9462, 5.5, "Asia/Kolkata"),
    ("Varanasi, India", 25.3176, 82.9739, 5.5, "Asia/Kolkata"),
    ("Kathmandu, Nepal", 27.7172, 85.3240, 5.75, "Asia/Kathmandu"),
    ("Colombo, Sri Lanka", 6.9271, 79.8612, 5.5, "Asia/Colombo"),
    ("Dubai, UAE", 25.2048, 55.2708, 4.0, "Asia/Dubai"),
    ("Singapore", 1.3521, 103.8198, 8.0, "Asia/Singapore"),
    ("Kuala Lumpur, Malaysia", 3.1390, 101.6869, 8.0, "Asia/Kuala_Lumpur"),
    ("London, UK", 51.5074, -0.1278, 0.0, "Europe/London"),
    ("New York, USA", 40.7128, -74.0060, -5.0, "America/New_York"),
    ("Chicago, USA", 41.8781, -87.6298, -6.0, "America/Chicago"),
    ("San Francisco, USA", 37.7749, -122.4194, -8.0, "America/Los_Angeles"),
    ("Toronto, Canada", 43.6532, -79.3832, -5.0, "America/Toronto"),
    ("Sydney, Australia", -33.8688, 151.2093, 10.0, "Australia/Sydney"),
];

#[derive(Clone, Debug, PartialEq)]
pub struct Place {
    pub name: String,
    pub admin1: String,
    pub country: String,
    pub lat: f64,
    pub lon: f64,
    /// IANA zone such as "Asia/Kolkata" or "America/Chicago".
    pub tz_name: String,
}

impl Place {
    /// "Chennai, Tamil Nadu, India".
    pub fn label(&self) -> String {
        [self.name.as_str(), self.admin1.as_str(), self.country.as_str()].iter().filter(|s| !s.is_empty()).cloned().collect::<Vec<_>>().join(", ")
    }
}

/// Look a place up. `query` is "City", "City, State" or "City, State, Country"; the city is
/// searched and the other parts pick the best of the matches.
pub fn search(query: &str) -> Result<Vec<Place>, String> {
    let parts: Vec<String> = query.split(',').map(|p| p.trim().to_string()).filter(|p| !p.is_empty()).collect();
    let city = parts.first().ok_or("type a city first")?;
    let agent: ureq::Agent = ureq::Agent::config_builder().timeout_global(Some(Duration::from_secs(20))).build().into();
    let resp = agent
        .get("https://geocoding-api.open-meteo.com/v1/search")
        .query("name", city)
        .query("count", "20")
        .query("language", "en")
        .query("format", "json")
        .call()
        .map_err(|e| format!("could not reach the place service: {e}"))?;
    let v: serde_json::Value = resp.into_body().read_json().map_err(|e| format!("unreadable reply: {e}"))?;
    let mut places = parse(&v);
    rank(&mut places, &parts[1..]);
    Ok(places)
}

/// Reverse lookup: coordinates → the nearest settlement, named in `lang` ("en", "ta" or "hi").
/// Used by the calendar's "Use my location" so the page names where it thinks you are instead of
/// an anonymous "My location". Nominatim (OpenStreetMap) is the only free service that both
/// reverses and localises names; its data is ODbL, so the calendar credits OpenStreetMap.
/// `zoom=10` asks for city/district granularity rather than a street address.
pub fn reverse(lat: f64, lon: f64, lang: &str) -> Result<String, String> {
    if !(-90.0..=90.0).contains(&lat) || !(-180.0..=180.0).contains(&lon) {
        return Err("those coordinates are out of range".into());
    }
    let agent: ureq::Agent = ureq::Agent::config_builder().timeout_global(Some(Duration::from_secs(10))).build().into();
    let v: serde_json::Value = agent
        .get("https://nominatim.openstreetmap.org/reverse")
        .query("format", "jsonv2")
        .query("zoom", "10")
        .query("lat", lat.to_string())
        .query("lon", lon.to_string())
        .query("accept-language", lang)
        // Nominatim's usage policy asks every caller to identify itself.
        .header("User-Agent", "vedic-astrology/0.1 (AGPL-3.0; self-hosted)")
        .call()
        .map_err(|e| format!("could not reach the place service: {e}"))?
        .into_body()
        .read_json()
        .map_err(|e| format!("unreadable reply: {e}"))?;
    if let Some(e) = v["error"].as_str() {
        return Err(e.to_string());
    }
    Ok(reverse_label(&v).ok_or("no place was found at those coordinates")?)
}

/// "Chennai, Tamil Nadu, India" from a Nominatim reverse reply: the most local name it offers,
/// then the state and country, so the label matches what `search` produces for a typed city.
fn reverse_label(v: &serde_json::Value) -> Option<String> {
    let a = &v["address"];
    let pick = |keys: &[&str]| keys.iter().find_map(|k| a[*k].as_str().filter(|s| !s.is_empty()).map(str::to_string));
    let local = pick(&["city", "town", "village", "municipality", "suburb", "county", "state_district"])
        .or_else(|| v["name"].as_str().filter(|s| !s.is_empty()).map(str::to_string))?;
    let parts: Vec<String> = [Some(local), pick(&["state", "region"]), pick(&["country"])].into_iter().flatten().collect();
    // A county that is also the state (city-states) would read "Singapore, Singapore".
    let mut out: Vec<String> = Vec::new();
    for p in parts {
        if !out.contains(&p) {
            out.push(p);
        }
    }
    Some(out.join(", "))
}

fn parse(v: &serde_json::Value) -> Vec<Place> {
    let s = |r: &serde_json::Value, k: &str| r[k].as_str().unwrap_or("").to_string();
    v["results"]
        .as_array()
        .map(|a| {
            a.iter()
                .filter_map(|r| {
                    Some(Place { name: s(r, "name"), admin1: s(r, "admin1"), country: s(r, "country"), lat: r["latitude"].as_f64()?, lon: r["longitude"].as_f64()?, tz_name: s(r, "timezone") })
                })
                .collect()
        })
        .unwrap_or_default()
}

/// Put the places whose state/country match the extra parts of the query first (stable, so the
/// service's own order — largest places first — decides among equals).
fn rank(places: &mut [Place], hints: &[String]) {
    let score = |p: &Place| {
        hints
            .iter()
            .filter(|h| {
                let h = h.to_lowercase();
                let hit = |s: &str| {
                    let s = s.to_lowercase();
                    !s.is_empty() && (s.contains(&h) || h.contains(&s))
                };
                hit(&p.admin1) || hit(&p.country) || (h.len() == 2 && p.country.to_lowercase().starts_with(&h))
            })
            .count()
    };
    places.sort_by_key(|p| std::cmp::Reverse(score(p)));
}

/// Hours east of UTC that `tz_name` used at that local date and time, or None for an unknown zone.
pub fn utc_offset(tz_name: &str, year: i32, month: u32, day: u32, hour: u32, minute: u32) -> Option<f64> {
    let tz: Tz = tz_name.parse().ok()?;
    let naive = NaiveDate::from_ymd_opt(year, month, day)?.and_hms_opt(hour, minute, 0)?;
    // A clock time skipped by a daylight-saving jump has no mapping: use the hour after.
    let local = tz.from_local_datetime(&naive).earliest().or_else(|| tz.from_local_datetime(&(naive + chrono::Duration::hours(1))).earliest())?;
    Some(local.offset().fix().local_minus_utc() as f64 / 3600.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn offsets_follow_daylight_saving_and_history() {
        // Soverna's sample: Illinois, December → CST, UTC-6.
        assert_eq!(utc_offset("America/Chicago", 2012, 12, 4, 20, 6), Some(-6.0));
        // Nyra's sample: Virginia, September → EDT, UTC-4 (the sample used -5 standard time by choice).
        assert_eq!(utc_offset("America/New_York", 2019, 9, 21, 10, 11), Some(-4.0));
        assert_eq!(utc_offset("Asia/Kolkata", 1990, 8, 15, 6, 30), Some(5.5));
        // India used +6:30 war time in 1943.
        assert_eq!(utc_offset("Asia/Kolkata", 1943, 6, 1, 12, 0), Some(6.5));
        assert_eq!(utc_offset("Not/AZone", 2000, 1, 1, 0, 0), None);
    }

    #[test]
    fn reverse_label_prefers_the_most_local_name_and_drops_repeats() {
        let v = serde_json::json!({"name": "சென்னை", "address": {"city": "சென்னை", "state": "தமிழ்நாடு", "country": "இந்தியா"}});
        assert_eq!(reverse_label(&v).unwrap(), "சென்னை, தமிழ்நாடு, இந்தியா");
        // Rural: no city/town, so the district stands in.
        let v = serde_json::json!({"address": {"county": "Tiruchengode", "state_district": "Namakkal", "state": "Tamil Nadu", "country": "India"}});
        assert_eq!(reverse_label(&v).unwrap(), "Tiruchengode, Tamil Nadu, India");
        // City-state: the same name must not be repeated.
        let v = serde_json::json!({"address": {"city": "Singapore", "state": "Singapore", "country": "Singapore"}});
        assert_eq!(reverse_label(&v).unwrap(), "Singapore");
        // Nothing usable at all.
        assert!(reverse_label(&serde_json::json!({"address": {}})).is_none());
    }

    #[test]
    fn reverse_rejects_impossible_coordinates() {
        assert!(reverse(91.0, 0.0, "en").is_err());
        assert!(reverse(0.0, 181.0, "en").is_err());
    }

    #[test]
    fn state_and_country_pick_the_right_match() {
        let v = serde_json::json!({"results": [
            {"name": "Springfield", "admin1": "Missouri", "country": "United States", "latitude": 37.2, "longitude": -93.3, "timezone": "America/Chicago"},
            {"name": "Springfield", "admin1": "Illinois", "country": "United States", "latitude": 39.8, "longitude": -89.6, "timezone": "America/Chicago"},
        ]});
        let mut p = parse(&v);
        rank(&mut p, &["Illinois".into(), "USA".into()]);
        assert_eq!(p[0].admin1, "Illinois");
        assert_eq!(p[0].label(), "Springfield, Illinois, United States");
    }
}

#[cfg(test)]
mod live {
    /// Needs the network: `cargo test live_lookup -- --ignored`.
    #[test]
    #[ignore]
    fn live_lookup() {
        let p = super::search("Springfield, Illinois, USA").unwrap();
        println!("{:?}", &p[..p.len().min(3)]);
        assert_eq!(p[0].admin1, "Illinois");
        assert_eq!(p[0].tz_name, "America/Chicago");
        let p = super::search("Madurai, Tamil Nadu, India").unwrap();
        assert_eq!(p[0].tz_name, "Asia/Kolkata");
    }
}
