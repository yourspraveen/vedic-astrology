# Architecture

## Crate layout

One Cargo package (`vedic_astrology`, Rust edition 2024) with a library and one program:

```
src/lib.rs                    library "vedic_astrology": all the logic
src/bin/vedic_astrology_web.rs   program "vedic_astrology_web": the website
build.rs                      compiles the Swiss Ephemeris C sources in vendor/swisseph
knowledge/                    knowledge base, built into the program and copied to the user's folder
```

All logic lives in the library so it stays testable; the binary is a thin web server. Positions
come from the Swiss Ephemeris (`src/swe.rs`, built from `vendor/swisseph/`); `src/astro.rs` keeps
the shared helpers (Julian day, sunrise, ayanamsa, calendar).

## Data flow

```
BirthData (vedic.rs) ──► Chart::compute ──► Chart (positions, ascendant, houses)
   date, time, place,         │                 │
   tz, tz_name                │                 ├─► panchanga()      → Panchanga
Calc (ayanamsa, node)         │                 ├─► vimshottari()    → Vec<Period> (dasha tree)
                              │                 ├─► find_yogas()     → Vec<YogaFound>
                              │                 └─► extra::*         → Ashtakavarga, KP, strengths…
                              ▼
        swe.rs (Swiss Ephemeris — all positions and the ascendant)

report::Input {birth, chart, pancha, dasha, calc, style, now}
   └─► report::build_with(&input, &ai_texts) → (HTML, Vec<AiTask>)
          uses readings.rs / texts.rs / pih_*.rs (reading texts), knowledge.rs (rules, glossary),
          i18n.rs (language), style.rs (chart layout)
   AiTask list ─► llm::expand (optional) → key → (model, text) ─► build_with again with AI texts
```

The website computes everything per request from the submitted birth details; the optional AI
pass, when enabled, runs `llm::expand` and then `build_with` again with the AI texts.

## Modules (library)

| Module | Responsibility | Key items |
|---|---|---|
| `astro` | Shared astronomy helpers: Julian day, ΔT, obliquity, sidereal time, sunrise/sunset (upper limb, −0.833°), ayanamsa reference table, civil calendar. (Planet/node/ascendant positions come from `swe`.) | `julian_day`, `calendar`, `sunrise_sunset`, `lahiri_ayanamsa` |
| `swe` | Swiss Ephemeris bindings (Moshier mode, apparent positions, serialised by a mutex) — every planet, node and the ascendant | `calc`, `nutation`, `asc_mc` |
| `vedic` | Core chart model: grahas, signs, nakshatras, dignity and natural friendship, vargas, Sripati houses, panchanga, Vimshottari dasha, yogas, doshas, Ashtakoota matching | `BirthData`, `Calc`, `Chart`, `Panchanga`, `Period`, `find_yogas`, `ashtakoota`, `kuja_dosha` |
| `extra` | Further rules: node doshas, KP lords, Ashtakavarga, karakas, yogi/avayogi, arudha, five-fold relationship, benefic/malefic and functional nature, dasha strength, life stages, Kalidina, transit sign periods | `ashtakavarga`, `lordship_nature`, `dasa_strength`, `life_stage`, `sign_periods` |
| `calendar` | Tamil / Vedic calendar: panchanga at sunrise, Tamil month/day/year (Surya Siddhanta Sun, 22:00 IST cut-off), Rahu kalam, Yamagandam, Gulika, Abhijit, Brahma muhurta, observances | `day`, `month`, `ss_sun`, `tamil_month_of`, `YEAR_NAMES` |
| `geo` | Place lookup (Open-Meteo geocoding), historical UTC offsets from IANA zones (`chrono-tz`), preset places | `search`, `utc_offset`, `PLACES` |
| `i18n` | Interface language (process-wide), translation table, name tables | `set_lang`, `tr`, `trf`, `sign`, `planet`, `nak`, `tithi` |
| `texts`, `readings`, `pih_en/ta/hi` | Reading texts and templates in three languages: panchanga, lagna, houses, dashas, bhuktis, yogas, remedies, planet in house | `texts::tpl`, `readings::t`, `readings::planet_in_house`, `readings::remedy` |
| `knowledge` | Knowledge base: rule files (lord in house, dasha), Sanskrit glossary, AI instructions, seeding the user folder, passage search for retrieval | `lord_in_house`, `dasha`, `bhava`, `ai_instructions`, `search`, `seed` |
| `llm` | Optional AI writer: OpenAI-compatible `chat/completions`; expands report sections; answers questions | `LlmConfig`, `chat`, `expand`, `ask` |
| `import` | Downloads classical texts (public-domain corpus; BPHS for personal study) into `texts/` | `import`, `html_to_text` |
| `report` | The full HTML report (16 sections, SVG charts, CSS for print) | `Input`, `build`, `build_with`, `SECTIONS`, `fmt_*` |
| `style` | Chart style enum shared by the report and web | `Style` |
| `web` | HTTP server and pages for the website | `serve`, `handle`, `ai_config` |

## Website (`web.rs`, `bin/vedic_astrology_web.rs`)

A small threaded HTTP/1.1 server on `std::net` with no framework. Routes:

| Method and path | Purpose |
|---|---|
| `GET /` | The form: birth details, Fetch, presets, report options, calendar link, question box |
| `POST /report` | Form fields → full HTML report (with AI expansions when `ai` is ticked and the server is configured) |
| `GET /calendar?y&m&lat&lon&tz&tz_name&place&lang` | Month calendar with daily timings |
| `GET /api/geocode?q&date&time` | JSON place matches with zone and UTC offset at that date |
| `POST /api/ask` | JSON answer to a question about the chart |
| `GET /health` | `ok` (used by the container health check) |

The interface language is a process-wide setting, so page rendering takes the `RENDER` lock.
Form parsing is in `form`/`birth_from`. AI settings come from `VA_AI_*` environment variables
(`ai_config`).

## Knowledge base

`knowledge/` is built into the program (`knowledge::EMBEDDED`) and copied to the user's knowledge
folder (default `~/.vedic_astrology/knowledge`, in Docker `/opt/vedic_astrology/knowledge`) on start.
Missing files are added; edited files are never overwritten. The folder holds:

* rule files (`rules/<lang>/*.json`);
* the Sanskrit glossary and concepts (`sanskrit/`);
* the AI instructions (`ai/instructions.md`);
* the source register (`sources.json`);
* downloaded or added texts (`texts/`).

Rules are read from the folder first, then the built-in copy. The AI writer's search reads every
`.md`, `.txt` and `.json` file in the folder except `ai/`. See knowledge/README.md.

## Where values come from

Every number, table and rule that reproduces the original program, or that was chosen for this
one, is listed in MECHANICS.md (prose, with evidence) and `mechanics.json` (machine-readable).
Tests named after the samples check them against the original's printed reports.
