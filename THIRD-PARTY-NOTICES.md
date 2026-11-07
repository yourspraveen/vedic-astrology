# Third-party notices

Vedic Astrology is distributed under the GNU Affero General Public License v3.0 or later
(see [LICENSE](LICENSE)). It includes and depends on the following third-party work.

## Swiss Ephemeris — bundled source

The planetary and house calculations use the **Swiss Ephemeris** (Moshier mode, so no
ephemeris data files are needed). Its C source is vendored under `vendor/swisseph/`.

> Swiss Ephemeris © 1997–2021 Astrodienst AG, Switzerland. All rights reserved.

Swiss Ephemeris is dual-licensed under the AGPL or a paid Swiss Ephemeris Professional License.
**This project uses it under the AGPL**, which is why the whole project is AGPL. The full Swiss
Ephemeris licence is kept verbatim at `vendor/swisseph/LICENSE` and `vendor/swisseph/agpl-3.0.txt`.
See <https://www.astro.com/swisseph/> for the professional-licence option.

## Rust dependencies

Resolved at build time by Cargo; all are permissively licensed (MIT / Apache-2.0 / ISC /
BSD-style). The direct ones:

| Crate | Purpose | Licence |
|---|---|---|
| `serde`, `serde_json` | (de)serialisation of config and knowledge files | MIT OR Apache-2.0 |
| `ureq` (with `rustls`, `ring`) | HTTPS client for the optional AI writer | MIT OR Apache-2.0 (ring: ISC-style) |
| `chrono`, `chrono-tz` | dates and time zones | MIT OR Apache-2.0 |
| `cc` (build) | compiles the vendored Swiss Ephemeris C source | MIT OR Apache-2.0 |

Each crate's full licence text is available in its source package and via
`cargo tree` / crates.io. Run `cargo tree` for the complete transitive list.

## Optional downloaded texts

The optional `--import-texts` command can fetch the public **jyotisha-corpus**
(classical Jyotisha texts and extracted rules) into a local knowledge folder at runtime.
That material is licensed **CC BY 4.0**, attributed to the *maximally0/jyotisha-corpus*
contributors. It is **not** committed to this repository and is not part of the shipped image
unless you import it yourself.

## Classical sources

The app's own rule writeups draw on classical Jyotisha works (Brihat Parashara Hora Shastra,
Phaladeepika, Brihat Jataka, Saravali, and others — see `knowledge/sources.json`). These are
ancient, public-domain traditions; **no copyrighted edition or translation text is included** —
the rules are restated in the project's own words.
