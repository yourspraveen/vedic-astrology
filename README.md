# Vedic Astrology

[![License: AGPL-3.0-or-later](https://img.shields.io/badge/License-AGPL--3.0--or--later-blue.svg)](LICENSE)

A free, self-hosted Vedic astrology (Jyotisha) web app in English, Tamil and Hindi. It computes a
full birth chart and produces a detailed, printable report, with an optional AI writer grounded in
a knowledge base of classical rules and texts.

* Birth, divisional and house charts (South, North and East styles)
* Panchanga, Vimshottari dashas (with bhuktis), yogas and doshas, Ashtakavarga, marriage-matching
  attributes (yoni, gana, nadi…) and transits
* A detailed, printable report you can download as a PDF
* A Tamil / Vedic calendar with daily auspicious times
* An optional AI writer and an Ask box, both answering only from this chart plus the knowledge base
  (retrieval-augmented), with a saved Q&A log you can export

Positions come from the **Swiss Ephemeris** (Moshier mode — no ephemeris data files needed).

## Quick start (one command)

```sh
./scripts/setup.sh     # open the URL it prints
```

This gets the app running in your own space with minimal inputs. It uses Docker if it finds it,
otherwise builds from source with Cargo, and either way imports the public-domain **jyotisha-corpus**
(classical texts) into your knowledge folder so the AI writer and Ask box have them to draw on. Flags:
`--local` (force the source build), `--no-texts` (skip the corpus), `--listen ADDR:PORT`,
`--knowledge DIR`. The AI writer stays off until you set `VA_AI_BASE_URL`/`VA_AI_API_KEY` (see below).

If you prefer to drive it yourself, the manual paths follow.

## Run it with Docker (recommended)

```sh
cp .env.example .env     # optional: fill in AI settings (see below). Blank = app runs without AI.
docker compose up -d --build
# open http://localhost:8080
```

The image is built on a minimal Chainguard base, runs as a non-root user with a read-only root
filesystem, and has a built-in health check (`vedic_astrology_web --health-check`).

Stop it with `docker compose down`.

## Run it from source

```sh
cargo run --release --bin vedic_astrology_web -- --listen 127.0.0.1:8080
# open http://127.0.0.1:8080
```

Requires a Rust toolchain (edition 2024) and a C compiler (to build the vendored Swiss Ephemeris).

## Configure the AI writer (optional)

The app works fully without AI — the report is produced by the rule engine. Turning the AI writer
on adds a narrative to each section and enables the Ask box. Set these in `.env` (see
[.env.example](.env.example) for the full list):

| Variable | Meaning |
|---|---|
| `VA_AI_BASE_URL` | Base URL of an OpenAI-compatible service (include `/v1`) |
| `VA_AI_API_KEY` | API key for that service |
| `VA_AI_MODEL` | Fast model for the report body (fans out into ~20 parallel calls) |
| `VA_AI_ASK_MODEL` | Reasoning model for the Ask box (blank = reuse `VA_AI_MODEL`) |
| `VA_AI_MAX_TOKENS` | Max tokens per reply (default 2000) |

These live only on the server and are never sent to the browser. You can also adjust the model,
temperature, knowledge passages and AI instructions at **Settings** in the running app.

### Using your own knowledge folder

The optional `--import-texts` command fetches public-domain classical texts (the
[jyotisha-corpus](https://github.com/maximally0/jyotisha-corpus), CC BY 4.0) into a local knowledge
folder — this is what `./scripts/setup.sh` runs for you. To do it by hand:

```sh
vedic_astrology_web --import-texts ~/.vedic_astrology/knowledge          # public-domain corpus
vedic_astrology_web --listen 127.0.0.1:8080 --knowledge ~/.vedic_astrology/knowledge
```

To run the container against your own folder, uncomment the `volumes:` mapping in
`docker-compose.yml` (the folder must be readable by uid 65532).

## Documentation

* [CONTRIBUTING.md](CONTRIBUTING.md) — how to build, test, the quality gates, and the contribution workflow
* [RELEASE.md](RELEASE.md) — how to cut a versioned release and publish the image
* [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) — module map and data flow
* [docs/PACKAGING.md](docs/PACKAGING.md) — the Chainguard image, deployment, security and licensing
* [MECHANICS.md](MECHANICS.md) — every value, formula and rule, with its source and how it was checked
* [knowledge/README.md](knowledge/README.md) — the knowledge base (rules, glossary, AI instructions, texts)

## Licence

**AGPL-3.0-or-later** — see [LICENSE](LICENSE). The app bundles the Swiss Ephemeris, which it uses
under the AGPL; that is what sets the project's licence. Attribution and dependency licences are in
[THIRD-PARTY-NOTICES.md](THIRD-PARTY-NOTICES.md). If you run a modified version as a network service,
the AGPL requires you to offer users its source. The running app does this for you: every page has a
footer linking to the source code. Point that link at your own published source by setting
`VA_SOURCE_URL` (otherwise it uses a generic default).
