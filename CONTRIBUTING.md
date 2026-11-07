# Contributing to Vedic Astrology

Thanks for your interest. This project aims to stay small, legible and future-ready, so the bar for
a change is: it builds clean, it is covered by a test where behaviour matters, and it reads like the
code around it. This guide is how to get there.

## Project shape

One Cargo crate, built two ways:

* a library (`src/lib.rs`) with all the logic — calculations, report, knowledge base, AI writer,
  calendar, web server;
* one binary, the web server (`src/bin/vedic_astrology_web.rs`).

Positions come from the Swiss Ephemeris (`src/swe.rs`, compiled from `vendor/swisseph/` by
`build.rs`). See [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) for the module map.

## Set up and run

```sh
cargo run --release --bin vedic_astrology_web -- --listen 127.0.0.1:8080   # run locally
cargo test                                                              # tests
docker compose up -d --build                                           # run in Docker
```

You need a Rust toolchain (edition 2024), a C compiler (for Swiss Ephemeris), and `clippy`
(`rustup component add clippy`).

## The quality gate

Before you push, run:

```sh
./scripts/check.sh
```

It enforces, and CI runs the same thing (see `.gitea/workflows/ci.yml`):

1. **Lint** — `cargo clippy -- -D warnings`. No warnings. This also denies Rust naming lints, so
   functions/modules are `snake_case`, types are `CamelCase`, and constants are `SCREAMING_SNAKE`.
2. **Tests** — `cargo test` must pass.
3. **Structure** — no source file over **1600 lines** (you get a warning over 1000). Split a module
   before it becomes a god-file; group a long data table into its own file.

If a clippy lint is genuinely wrong for a spot, silence it narrowly with
`#[allow(clippy::the_lint)]` and a one-line comment saying why — never a crate-wide allow.

## Code style

* **Match the surrounding style.** The codebase is deliberately dense in places (long embedded
  HTML/CSS strings, packed match arms). Formatting is *not* auto-enforced; write new code to look
  like its neighbours rather than reformatting existing code.
* **Keep functions focused.** If a function needs a long comment to explain its halves, it is
  probably two functions.
* **The library must not depend on the web server**, and vice-versa beyond the public API — logic
  lives in the library so it stays testable.
* **Name things for the reader**, in plain words, not for how the system is built.

## Tests go with the change

If you change behaviour in `src/`, add or update a test in the same change. Calculation and rule
changes especially must be pinned by a test (see the reference-sample tests in `src/extra.rs` and
the rule-count tests in `src/knowledge.rs`). A pure refactor that keeps behaviour needs no new test,
but must keep the existing ones green.

## Knowledge base and licensing

* Interpretive text we ship (`knowledge/texts/concepts-*.md`, rule JSON, the glossary) is written in
  **our own words** so it is license-clean. Do not paste text from a copyrighted edition or
  translation of a classical work.
* Register any classical source you lean on in `knowledge/sources.json`.
* The app bundles the **Swiss Ephemeris under the AGPL**, which is why the whole project is
  AGPL-3.0. Keep it that way, or the dependency obligation is broken. See
  [THIRD-PARTY-NOTICES.md](THIRD-PARTY-NOTICES.md).

## Branches and commits

* Branch from `main`: `feat/…`, `fix/…`, `docs/…`, `refactor/…`, `chore/…`.
* Write [Conventional Commits](https://www.conventionalcommits.org/): `feat(report): …`,
  `fix(web): …`, `docs: …`. Keep the subject in the imperative and under ~72 characters.
* One logical change per pull request. In the PR description, say what changed and why, and name any
  test you added.

## Pull request checklist

- [ ] `./scripts/check.sh` passes locally
- [ ] Tests added/updated for behaviour changes
- [ ] No copyrighted text added to the knowledge base
- [ ] The change reads like the code around it

By contributing you agree your work is licensed under AGPL-3.0-or-later.
