# Vedic Astrology web app in Docker

The web app is built on Chainguard images (`cgr.dev/chainguard/rust` to build,
`cgr.dev/chainguard/glibc-dynamic` to run). Quick start:

```sh
docker compose up -d --build        # then open http://localhost:8080
```

Everything else is in [docs/PACKAGING.md](docs/PACKAGING.md): building and exporting the image,
the AI writer settings, mounting your own knowledge folder, security and licences.
