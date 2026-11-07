# AGENTS.md

Guidance for AI coding agents (and a quick orientation for humans). Read
[CONTRIBUTING.md](CONTRIBUTING.md) for the full workflow; this is the short version plus the
invariants that must not be broken.

## What this is

A self-hosted Vedic astrology web app in Rust. One library (`src/lib.rs`) holds all logic; one
binary (`src/bin/vedic_astrology_web.rs`) serves the website. Positions come from the Swiss Ephemeris
(`src/swe.rs`, built from `vendor/swisseph/`). See [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md).

## Before you finish, run the gate

```sh
./scripts/check.sh      # clippy -D warnings, tests, file-size caps
```

CI runs the same thing. Do not report a change as done until this is green.

## Invariants — do not break these

* **Swiss Ephemeris is the only position engine.** There is no built-in/Moshier toggle; don't add
  one back. `src/astro.rs` keeps only shared helpers (Julian day, sunrise, ayanamsa, calendar).
* **AGPL-3.0.** The Swiss Ephemeris is used under the AGPL, so the whole project is AGPL. Do not add
  a dependency or vendored code under an incompatible licence, and do not relicense.
* **Knowledge base is license-clean by construction.** Interpretive text we ship is written in our
  own words. Never paste text from a copyrighted edition/translation of a classical work. Register
  sources in `knowledge/sources.json`.
* **Library stays independent of the web layer.** Keep logic testable in the library; the binary is
  a thin server.
* **Secrets stay out of git.** Runtime config (AI keys) comes from `.env`/environment, which is
  gitignored. Never hardcode a key, host IP, or personal hostname.

## Conventions

* Rust naming (enforced by clippy): `snake_case` functions/modules, `CamelCase` types,
  `SCREAMING_SNAKE` constants.
* Match the surrounding dense style; don't bulk-reformat.
* Behaviour change ⇒ a test in the same change. Calculations and rule counts are pinned by tests in
  `src/extra.rs` and `src/knowledge.rs`.
* Conventional Commits (`feat(...)`, `fix(...)`, `docs: …`); branch as `feat/…`, `fix/…`, etc.

## Handy commands

```sh
cargo run --release --bin vedic_astrology_web -- --listen 127.0.0.1:8080   # run
curl -fsS http://127.0.0.1:8080/health                                  # health (200 "ok")
docker compose up -d --build                                            # run in Docker
```
