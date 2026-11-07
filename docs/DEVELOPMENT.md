# Development guide

See also [CONTRIBUTING.md](../CONTRIBUTING.md) (workflow and the quality gate) and
[AGENTS.md](../AGENTS.md) (invariants).

## Requirements

* Rust (stable, edition 2024; tested with 1.9x) and a C compiler (for the Swiss Ephemeris sources
  compiled by `build.rs` through the `cc` crate).
* `clippy` (`rustup component add clippy`) — the gate denies all warnings.
* Docker (optional) for the web image.

## Build and run

```sh
cargo build                                                         # library + web server
cargo run --release --bin vedic_astrology_web -- --listen 127.0.0.1:8080
cargo build --release                                               # optimised (LTO, stripped)
docker compose up -d --build                                        # run in Docker
```

### Command-line options

| Option | Effect |
|---|---|
| `--listen <addr:port>` (or `VA_LISTEN`) | Address to serve on (default `0.0.0.0:8080`) |
| `--knowledge <dir>` (or `VA_KNOWLEDGE`) | Knowledge folder (default `~/.vedic_astrology/knowledge`) |
| `--import-texts <knowledge dir> [--personal]` | Download classical texts into `<dir>/texts` |
| `--health-check` | Exit 0 when the local server answers `/health` (used by Docker) |

## Tests and the gate

```sh
./scripts/check.sh               # the gate: clippy -D warnings, tests, file-size caps
cargo test                       # all offline tests
cargo test -- --ignored          # network and data-file checks (see below)
```

* **Offline tests** check, among other things:
  * positions against the reference sample reports (`swiss_engine_matches_reference_samples`);
  * dasha strength (`dasa_strength_matches_samples`), life stages and transit houses;
  * Ashtakavarga, KP, Kalidina, the sunrise rule and time-zone offsets;
  * Tamil calendar dates (Pongal, Tamil New Year) and Tamil year names;
  * the knowledge base files, the HTML of the web pages, and form decoding.
* **Ignored tests** (`--ignored`) need the network (`live_lookup`, `live_import`) or the decoded
  original calendar table in `target/check/tamil_years.csv` (`tamil_table_report`, `tamil_ss_report`,
  `tamil_cutoff_report`).

Add a test next to the code it checks (`#[cfg(test)] mod tests` at the end of the module). For a
value validated against the reference samples, name the test after the evidence and record the value
in MECHANICS.md and `mechanics.json`.

## Conventions

* **Language:**
  * Every text shown to people goes through `i18n::tr("English text")` or
    `i18n::trf("… {0} …", &[args])`. The English text is the key.
  * Add the Tamil and Hindi forms to the `UI` table in `src/i18n.rs` as
    `("English", "தமிழ்", "हिन्दी")`. A missing entry falls back to English.
  * Longer reading texts live in `texts.rs` (`Tpl`), `readings.rs` (`R`, three-language arrays) and
    `pih_*.rs`.
* **Library vs binary:** calculations, texts, the report and the web pages belong in the library
  (`src/`); the binary (`src/bin/vedic_astrology_web.rs`) is a thin server. Keep logic testable in the
  library.
* **One position engine:** all planet/node/ascendant positions come from the Swiss Ephemeris
  (`src/swe.rs`). Do not reintroduce a built-in/Moshier engine or an engine toggle. `astro.rs` keeps
  only the shared helpers (Julian day, ΔT, sunrise, sidereal time, ayanamsa, calendar).
* **Background work on the server:** the AI pass releases the `RENDER` lock during network calls;
  `llm::expand` fans out with `std::thread::scope`. Never hold the render lock across the network.
* **Our own words:** never copy prose from a copyrighted edition or translation. Classical sources
  are registered in `knowledge/sources.json`; all wording here is written for this app.
* **Rights:** only add texts to the knowledge base or image when their licence allows it.
  Personal-study texts go in `texts/personal/` and are never packaged.

## Recipes

### Add a report section
1. Add its English title to `report::SECTIONS` (the table of contents).
2. In `report::build_with`, write it in order with `h.h2(id, tr("Title"))`, `h.h3`, `h.p`,
   `h.table`, `h.kv` (all escape their text).
3. To let the AI writer expand it, push `AiTask { key, title, facts: plain(&h.0[start..]) }` and call
   `append_ai`/`replace_ai` after the rule-based text (see the house and transit sections).
4. Check the output in the running app (`docker compose up -d --build`, then generate a report).

### Add a yoga
1. In `vedic::find_yogas`, push `YogaFound { name, why, planets }` when the condition holds.
   `planets` are the grahas that form it; their dashas mention it.
2. Add its result text, in three languages, to `readings::YOGA_RESULTS`, and map the name in
   `report::yoga_group` if it belongs to a group.
3. Add a test with a chart that has the yoga. Document the rule in MECHANICS.md.

### Add or change a rule in the knowledge base
* **Rule files:** `knowledge/rules/<en|ta|hi>/<file>.json` with `schema`, `source`, `status` and
  `rules`. Read them with a function in `knowledge.rs`, following `lord_in_house`.
* **New built-in files:** add them to `knowledge::EMBEDDED`, so they reach the program and are seeded
  into people's folders.
* **Status:** mark unverified paraphrases `"status": "paraphrase_unverified"`. The report then says
  so under the citation.

### Add a language string
Add `("English", "Tamil", "Hindi")` to `UI` in `i18n.rs`; use `tr`/`trf` with the exact English key.

### Add a web page or API
1. Add a route in `web::handle` returning `html(...)`, `json(...)` or `error(...)`.
2. Parse inputs with `form()` and `birth_from()`. Take the `RENDER` lock around anything that sets
   the language.
3. Add a case to the `pages_render` test.

### Change what the AI writes
Edit `knowledge/ai/instructions.md`: the "Report sections" heading is for expanding reports, and
"Questions" is for the Ask box. People can edit their own copy at ⚙ Settings. Code: `llm::expand`,
`llm::ask`, `knowledge::ai_instructions`.

### Add a calendar observance or day period
* **Observances:** in `calendar::observance` (tithi-based).
* **Day periods:** add a field to `calendar::Day`, computed in `calendar::day`.
* Show the new item in `web::calendar_page`, and translate the labels.

### Add a calculation setting
Add a field to `vedic::Calc` with `#[serde(default)]` (so old saved details still load), offer it on
the web form (`birth_from` in `web.rs`), and use it in `Chart::compute` / `sidereal`.
