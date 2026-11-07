//! Vedic Astrology core library: everything the web server is built from.
//!
//! * `vedic_astrology_web` (src/bin/vedic_astrology_web.rs): the self-hosted website, packaged on a
//!   Chainguard base image.
//!
//! See docs/ARCHITECTURE.md for how the modules fit together.

/// Built-in astronomy: Julian days, Meeus/Schlyter positions, sunrise, ayanamsa tables.
pub mod astro;
/// Tamil / Vedic daily calendar: sunrise panchanga, Tamil date and year, day periods.
pub mod calendar;
/// Extra calculations: Ashtakavarga, KP lords, karakas, functional nature, strengths, life stages.
pub mod extra;
/// Multi-tradition festival overlay for the calendar (computed estimates).
pub mod festivals;
/// Place lookup (geocoding), IANA time zones and historical UTC offsets, preset places.
pub mod geo;
/// Interface languages (English, Tamil, Hindi): `tr`, `trf` and name tables.
pub mod i18n;
/// Download of classical texts into the knowledge folder.
pub mod import;
/// The knowledge base: rule files, Sanskrit glossary, AI instructions, passage search (RAG).
pub mod knowledge;
/// Optional AI writer: OpenAI-compatible chat completions.
pub mod llm;
/// Planet-in-house readings, English.
pub mod pih_en;
/// Planet-in-house readings, Hindi.
pub mod pih_hi;
/// Planet-in-house readings, Tamil.
pub mod pih_ta;
/// Reading texts and templates (houses, dashas, yogas, remedies).
pub mod readings;
/// The full HTML report.
pub mod report;

pub mod save;
/// Chart style choice (South / North / East).
pub mod style;
/// Swiss Ephemeris (C library, Moshier mode) bindings.
pub mod swe;
/// Panchanga and general reading texts.
pub mod texts;
/// Core Vedic chart: birth data, positions, houses, vargas, panchanga, dashas, yogas, matching.
pub mod vedic;
/// The self-hosted web server.
pub mod web;
