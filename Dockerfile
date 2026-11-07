# Vedic Astrology web app on Chainguard images (https://hub.docker.com/u/chainguard).
#
#   docker build -t vedic-astrology-web .
#   docker run -p 8080:8080 vedic-astrology-web           then open http://localhost:8080
#
# Optional AI writer: -e VA_AI_BASE_URL=https://api.openai.com/v1 -e VA_AI_API_KEY=... -e VA_AI_MODEL=gpt-4o-mini
# Details: docs/PACKAGING.md.

# ---- build: Chainguard Rust (with gcc for the Swiss Ephemeris C sources) ----------------------
FROM cgr.dev/chainguard/rust:latest-dev AS build
WORKDIR /home/nonroot/app
COPY --chown=nonroot:nonroot Cargo.toml Cargo.lock build.rs ./
COPY --chown=nonroot:nonroot vendor ./vendor
COPY --chown=nonroot:nonroot src ./src
COPY --chown=nonroot:nonroot knowledge ./knowledge
# The web server (the only program).
RUN cargo build --release --locked --bin vedic_astrology_web

# A corpus already present on the building machine, if any. Copied after the compile so swapping
# texts does not invalidate the cargo cache. Everything inside corpus/ except its README is
# gitignored, so this folder is empty in a fresh clone and the build falls through to the download
# below -- we bake in the copy we already hold, other people fetch their own. See corpus/README.md.
COPY --chown=nonroot:nonroot corpus ./corpus

# The knowledge base: built-in rules, Sanskrit glossary, AI instructions, plus the public-domain
# classical texts of jyotisha-corpus (CC BY 4.0; attribution in texts/SOURCES.md), taken from
# corpus/ when it has them and downloaded otherwise. --build-arg WITH_TEXTS=0 skips the download.
# Texts kept only for personal study (BPHS from indike.org) are never baked in: .dockerignore keeps
# them out of the build context and the rm below is the second lock. Mount your own knowledge
# folder at runtime for those.
ARG WITH_TEXTS=1
RUN mkdir -p /home/nonroot/knowledge/texts \
 && cp -r knowledge/. /home/nonroot/knowledge/ \
 && rm -rf /home/nonroot/knowledge/texts/personal \
 && if [ -d corpus/jyotisha-corpus ]; then \
      echo "==> corpus found in the build context; baking it in without downloading"; \
      cp -r corpus/jyotisha-corpus /home/nonroot/knowledge/texts/; \
      if [ -f corpus/SOURCES.md ]; then cp corpus/SOURCES.md /home/nonroot/knowledge/texts/SOURCES.md; fi; \
    elif [ "$WITH_TEXTS" = "1" ]; then \
      echo "==> no corpus in the build context; downloading the public-domain jyotisha-corpus"; \
      ./target/release/vedic_astrology_web --import-texts /home/nonroot/knowledge || echo "text download failed; the image works without it"; \
    else \
      echo "==> WITH_TEXTS=0: building without the classical texts"; \
    fi \
 && rm -rf /home/nonroot/knowledge/texts/personal

# ---- run: Chainguard glibc-dynamic (no shell, no package manager, runs as nonroot 65532) -------
FROM cgr.dev/chainguard/glibc-dynamic:latest
LABEL org.opencontainers.image.title="Vedic Astrology web" \
      org.opencontainers.image.description="Free Vedic astrology: charts, reports, Tamil/Vedic calendar, optional AI writer" \
      org.opencontainers.image.licenses="AGPL-3.0-or-later"
COPY --from=build /home/nonroot/app/target/release/vedic_astrology_web /usr/bin/vedic_astrology_web
COPY --from=build --chown=65532:65532 /home/nonroot/knowledge /opt/vedic_astrology/knowledge
# Vedic Astrology and the Swiss Ephemeris it includes are AGPL-3.0: the licence travels with the image.
COPY vendor/swisseph/LICENSE vendor/swisseph/agpl-3.0.txt /opt/vedic_astrology/licenses/
ENV VA_KNOWLEDGE=/opt/vedic_astrology/knowledge \
    VA_LISTEN=0.0.0.0:8080
USER 65532
EXPOSE 8080
HEALTHCHECK --interval=30s --timeout=5s --start-period=5s CMD ["/usr/bin/vedic_astrology_web", "--health-check"]
ENTRYPOINT ["/usr/bin/vedic_astrology_web"]
