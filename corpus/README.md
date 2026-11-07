# corpus/ — classical texts baked into *this machine's* image

Drop classical texts here and `docker compose up -d --build` bakes them into the image instead of
downloading them during the build. Everything except this README is gitignored, so **nothing here
reaches the repository**: a fresh clone gets an empty folder and the build fetches the
public-domain corpus itself, exactly as before.

That split is deliberate. We run with the texts already in hand; other people get the importer and
fetch their own copy, so we are not redistributing someone else's corpus on their behalf.

## Filling it

```sh
cargo run --release --bin vedic_astrology_web -- --import-texts ./corpus-tmp
mv corpus-tmp/texts/jyotisha-corpus corpus/
mv corpus-tmp/texts/SOURCES.md      corpus/
```

The layout the Dockerfile looks for:

```
corpus/
  jyotisha-corpus/     passages + rules.json  (public-domain editions, CC BY 4.0)
  SOURCES.md           what is here and under what terms
```

## What must never go here

`texts/personal/` — the BPHS chapters from indike.org. Its Sanskrit e-text comes from
sanskritdocuments.org, whose terms allow personal study but not redistribution. It is excluded by
`.dockerignore` and deleted again inside the build, so it cannot reach an image even by accident.
Mount it at runtime instead:

```yaml
volumes:
  - ${HOME}/.vedic_astrology/knowledge:/opt/vedic_astrology/knowledge
```

Build without any texts at all with `docker compose build --build-arg WITH_TEXTS=0`.
