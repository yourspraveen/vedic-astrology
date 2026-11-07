# Vedic Astrology: mechanics sheet

This app is written independently from published astronomical and Jyotisha formulae. Below are the
calculation **values, formulas and rules** it uses, each with its source and how it was checked. Two
anonymised **reference sample reports** (referred to as sample A and sample B) are used only as
expected-output fixtures for the tests; no third-party code, text or artwork is included.

The ephemeris is the Swiss Ephemeris in Moshier mode (no data files), which supplies every planet,
node and ascendant position. A few built-in routines remain, used only for the calendar and timing
(ΔT, sunrise/sunset, sidereal time) and the ayanamsa table. Where a value is marked **estimated**,
no authoritative source fixed it and it was chosen to best fit the reference samples or common
convention.

## Astronomy and the ephemeris

| Item | Rule / value | Source |
|---|---|---|
| Engine | Swiss Ephemeris 2.10.03 (github.com/aloistr/swisseph, AGPL-3.0), Moshier mode, apparent positions with nutation — all planet, node and ascendant/MC positions | compiled in `vendor/swisseph`, `build.rs`, `src/swe.rs`; matches the reference sample sayana values within 2″ |
| Mean / true node | selectable (`NodeKind`); taken from Swiss (mean or true); **mean** is the default | reference samples show Rahu within 0.3′ of the mean node, 50′+ from the true node |
| ΔT | Espenak–Meeus polynomials (TT⇄UT and the sunrise instant) | NASA |
| Sunrise / sunset, obliquity, sidereal time | built-in (Meeus ch. 12/15; upper limb −0.833°), used for the panchanga day and the auspicious-time windows | Meeus, *Astronomical Algorithms* |

## Ayanamsa

The app offers Chitra Paksha (Lahiri), Raman, Krishnamurti and a user-defined ayanamsa
(`vedic::Ayanamsa`). Reference values at J1900 (JD 2415020.0):

| Ayanamsa | Value at J1900 | Used in |
|---|---|---|
| Lahiri (Chitra Paksha) | 22.46047° | `astro::lahiri_ayanamsa` |
| Raman | 21.01444° | `Ayanamsa::Raman` |
| Krishnamurti | 22.363889° | `Ayanamsa::Krishnamurti` |
| User-defined | name, base year, precision (sample: "Fagen-Allen", 200, 50″/yr) | `Ayanamsa::User { base_year, arcsec_per_year }` |

Precession between the table epoch and the chart date follows general precession in longitude
(IAU 1976: 5029.0966″T + 1.11113″T² − 0.000006″T³). The Lahiri entry carried to 1956-03-21 gives
23.24557°, the official 23°15′00.658″ less nutation (test `lahiri_matches_1956_definition`).
Positions are apparent, so the table's mean ayanamsa gets the same nutation added, matching the
reference sample reports.

**User-defined ayanamsa:** inputs are limited to a base year of 200–600 and a precision of
49–51″/year (the reference input limits; the sample value is 200 / 50). The formula
`(year − base) × precision / 3600` is **estimated** — no source fixed how part-years are treated.

## Panchanga and sunrise

| Item | Rule | Source |
|---|---|---|
| Tithi, nakshatra, yoga, karana, vara | classical definitions (tithi span 12°, nakshatra span 360/27°) | classical |
| Weekday (vāra) | sunrise-to-sunrise; a pre-sunrise birth takes the previous weekday | classical; a civil (midnight) option is offered |
| Sunrise / sunset | Sun's upper limb at −0.833° (refraction included) | matches both reference samples within 1 min; other conventions are 2–5 min off (test `sunrise_conventions_vs_samples`) |
| Kalidina sankhya | civil day number (JD at local noon) − 588466 | checked against the reference sample report (1870282 on 2019-09-21) |

## Houses, vargas, dashas

| Item | Rule / value | Source |
|---|---|---|
| House system | Sripati (Sripathy) bhava | `vedic::sripati_bhavas` |
| Divisional charts | D1, D2, D3, D4, D7, D9, D10, D12, D16, D30, D60 | classical Shodasavarga |
| Varga rules, dignities, friendships | BPHS; natural and temporary friendship, five-fold relationship | BPHS |
| Vimshottari | Ke 7, Ve 20, Su 6, Mo 10, Ma 7, Ra 18, Ju 16, Sa 19, Me 17; sub-period = maha × minor / 120; year = 365.25 days | BPHS; reference dasa base-year length |

## Yogas, doshas and matching

| Item | Rule / value | Source |
|---|---|---|
| Yogas | 22 implemented from classical rules (Pancha Mahapurusha, Gaja Kesari, Vasumathi, Amala, Trigraha, Raja, Parivartana, Adhi, Sunapha/Anapha/Durudhara, Vesi, Kala Sarpa…) | classical; for the reference sample the app finds exactly the printed yogas (plus Vesi) |
| Kuja (Mars) dosha | Mars in houses 1, 2, 4, 7, 8, 12; judged from the Lagna (and, for reference, the Moon) | reference Kuja dosha houses |
| Rahu / Ketu dosha | node in houses 1, 2, 4, 7, 8, 12, judged separately; Jupiter aspect (5th/7th/9th) or same-sign association reduces it | reference node-dosha houses and relief rule |
| Ashtakoota matching | 36-point Ashtakoota with a Kuja-dosha check; 18 of 36 acceptable | common Ashtakoota convention |

## Readings and the rule knowledge base

Readings are written for this app in English, Tamil and Hindi; no other program's wording is used.
The report cites the rule each reading came from.

| Item | Rule | Source |
|---|---|---|
| Lord-in-house | 144 rules per language, id `BHAVESH-LL-PP` ("lord of house LL in house PP") | `knowledge/rules/*/lord_in_house.json`; BPHS ch. 24, restated |
| Dasha | one rule per planet with general/strong/weak text | `knowledge/rules/*/dasha.json`; BPHS + Phaladeepika, restated |
| Planet-in-house | 108 texts × 3 languages | `pih_en/ta/hi.rs`, from classical meanings |
| Functional nature | owning 1/5/9 → benefic; 3/6/11 = −1; kendra ±½; 8th −½; 2nd/12th 0 | reproduces all 14 rows of both reference samples' lordship tables (test `lordship_nature_matches_samples`) |
| Moon's paksha bala | benefic from Shukla Shashthi up to Krishna Shashthi (tithi 5–19) | reference samples: Krishna Shashthi/Ashtami births printed "without Paksha Bala" |
| Adverse dasas | lords of the 3rd, 5th and 7th stars from the birth star | reference sample: Magha → Sun, Mars, Jupiter |
| Avayogi star | 6th star from the yogi star | reference sample: Punarvasu → Uttara Phalguni |
| Good/bad dasa reading | seven planets: strong in exaltation/own/friendly sign, weak in neutral/enemy/debilitated; Rahu/Ketu strong in houses 1,4,5,7,9,10, neither in 2,3,11, weak in 6,8,12 (weak **estimated**) | `extra::dasa_strength`, matched on 11 dasas of the two reference sample reports (test `dasa_strength_matches_samples`) |
| Life-stage of a sub-period | by age at the **end** of the sub-period; breaks at 13 and 21 (observed), 37 and 65 (**estimated**) | `extra::life_stage`, matched on 16 sub-periods of the two reference samples (test `life_stage_matches_samples`) |
| Ashtakavarga | BPHS benefic-point tables; planet totals 48, 49, 39, 54, 56, 52, 39 = 337 | BPHS (test `ashtakavarga_total_is_337`) |
| KP lords | nakshatra split in Vimshottari proportions | Krishnamurti system |
| Chara karakas | Sun–Saturn by degrees in sign (7-karaka) | Jaimini |
| Transit forecast | houses counted from the birth Moon sign; Sun (next 12 months, good in 3/6/10/11), Jupiter, Saturn, Sade Sati, Ashtama | classical gochara; reference transit records in the two samples (test `transit_houses_match_samples`) |

House sections follow the usual life topics (personality, wealth, siblings, property/mother,
children/mind, health/enemies, marriage, longevity, fortune, career, income, expenditure).

## Remedies

Star, dasa and dosha remedies use traditional colours, fasting days, alms, flowers, gems, deities
and navagraha beeja mantras. The adverse taras (3rd/5th/7th from the birth star) and the
functional-malefic dasas drive which remedies are shown.

## Calendar (Tamil / Vedic)

| Item | Rule | Source |
|---|---|---|
| Tamil month | Surya Siddhanta Sun (4,320,000 revolutions / 1,577,917,828 days from JD 588465.5 at Ujjain 75.77° E; apogee 77°17′; epicycle 14° to 13°40′), sign change at a 22:00 IST cut-off, same for every place | fitted to the reference Tamil year table (95 years; 1,079 of 1,140 month starts match; tests `tamil_ss_report`, `pongal_and_tamil_new_year_fall_on_day_one`) |
| Tamil year (60-cycle) | 1927-28 = Prabhava | reference Tamil year table (Kali years 5029 and 5122); names written for the app in three languages |
| Daily periods | Rahu kalam, Yamagandam, Gulika (eighths of daylight, per weekday), Abhijit (8th of 15 day muhurtas), Brahma muhurta (14th of 15 night muhurtas) | classical; added by this app |
| Observances | Pournami, Amavasya, Ekadashi, Pradosham, Sankatahara Chaturthi | from the sunrise tithi |
| Time-zone check | warns when the UTC offset and longitude are on opposite sides of Greenwich, beyond 15° from the meridian and the Date Line (the 15° margin is **estimated**) | the reference warns on opposite-side time zones except near Greenwich or the Date Line |

## Optional AI writer

`llm.rs` (off by default) works with any OpenAI-compatible `chat/completions` service. It only
rewrites and expands the rule-based readings, using the chart facts plus passages retrieved from the
knowledge folder (RAG); it never calculates positions, predicts death, names illnesses, or gives
medical/legal/financial instructions, and it cites rule ids. The knowledge folder holds the rule
files, a Sanskrit glossary, editable AI instructions, and downloaded public-domain texts
(jyotisha-corpus). Settings expose a connection test, model selection, temperature, passages per
section and the instructions, and show the knowledge-base size that RAG will consult.

## Not built

Shadbala, Bhavabala, full Jaimini, KP sub-lords, Upagrahas, Sudarshana chakra and the Kerala chart
style are not in this version.
