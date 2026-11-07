# Packaging and deployment

Vedic Astrology ships as a self-hosted web app, packaged as a Docker image.

## Web app on Chainguard images

```sh
./scripts/package-web.sh            # build image vedic-astrology-web:<version> and :latest,
                                    # export dist/vedic-astrology-web-<version>-<arch>.tar.gz
docker compose up -d --build        # or run it here
```

### Images

| Stage | Image | Why |
|---|---|---|
| build | `cgr.dev/chainguard/rust:latest-dev` | Rust toolchain with gcc (the Swiss Ephemeris is C), runs as non-root |
| run | `cgr.dev/chainguard/glibc-dynamic:latest` | Minimal runtime for a glibc program: no shell, no package manager, CA certificates for HTTPS, non-root uid 65532 |

The image builds `vedic_astrology_web`. The runtime image is about 28 MB. The build also downloads the
public-domain classical texts into `/opt/vedic_astrology/knowledge`; use `--build-arg WITH_TEXTS=0` to
skip that.

### Run

```sh
docker run -d -p 8080:8080 --read-only vedic-astrology-web:latest
# with the AI writer:
docker run -d -p 8080:8080 --read-only \
  -e VA_AI_BASE_URL=https://api.openai.com/v1 -e VA_AI_API_KEY=sk-... -e VA_AI_MODEL=gpt-4o-mini \
  vedic-astrology-web:latest
```

Deploy elsewhere: copy the exported file and run `docker load -i vedic-astrology-web-<version>-<arch>.tar.gz`.
For several CPU types, push a multi-architecture image to your registry:
`PLATFORMS=linux/amd64,linux/arm64 IMAGE=registry.example.com/vedic-astrology-web ./scripts/package-web.sh`

### Configuration

| Variable | Default | Meaning |
|---|---|---|
| `VA_LISTEN` | `0.0.0.0:8080` | Address and port |
| `VA_KNOWLEDGE` | `/opt/vedic_astrology/knowledge` | Knowledge folder |
| `VA_SOURCE_URL` | generic default | Source-code link shown in the page footer (AGPL §13). Set it to your own published source. |
| `VA_AI_BASE_URL` | (none) | OpenAI-compatible service, e.g. `https://api.openai.com/v1`, or `http://host.docker.internal:11434/v1` for Ollama |
| `VA_AI_API_KEY` | (none) | Key (empty for local models) |
| `VA_AI_MODEL` | (none) | Model name; the AI writer is off unless both base URL and model are set |
| `VA_AI_USER`, `VA_AI_ORG` | (none) | Optional `user` field and OpenAI-Organization header |

* **Your own knowledge folder:** mount it over `/opt/vedic_astrology/knowledge`, for example for edited AI instructions or texts kept for personal study. It must be readable by uid 65532.
* **Health:** the image declares `HEALTHCHECK` with `vedic_astrology_web --health-check`, because there is no shell or curl in the image.

### Security notes

* **No login:** the website has none. Serve it on a private network or behind a reverse proxy that adds authentication and HTTPS.
* **Hardening:** it runs as non-root and works with a read-only root filesystem (`read_only: true` in `docker-compose.yml`).
* **AI key:** it stays on the server; browsers never see it.
* **Place lookup:** Fetch sends the typed place name to Open-Meteo's free geocoding service, which is for non-commercial use. People can always type coordinates and the zone instead.

### Licences

* **Code:** Vedic Astrology is AGPL-3.0-or-later and includes the Swiss Ephemeris (AGPL). If you offer the website to others, offer them the source too. The licence files are in `/opt/vedic_astrology/licenses` in the image.
* **Classical texts in the image:** public-domain editions with CC BY 4.0 corpus data; the attribution is in `texts/SOURCES.md`.
* **Where those texts come from at build time:** if the build context has a `corpus/` folder with
  `jyotisha-corpus/` in it, the build bakes that in and skips the download; otherwise it downloads
  the corpus itself (unless `--build-arg WITH_TEXTS=0`). `corpus/` is gitignored, so it is empty in
  a clone: we ship our own machine the copy we already hold, and everyone else fetches their own
  rather than receiving a redistributed one. See `corpus/README.md`.
* **Personal-study texts** (BPHS from indike.org, whose Sanskrit comes from sanskritdocuments.org) are never built into the image: `.dockerignore` keeps `**/texts/personal/` out of the build context, and the build deletes the path again before the runtime stage copies the folder. Mount them at runtime instead.
