# Release guide

How to cut a versioned release of the Vedic Astrology web app and publish its container image.
Releases are small and repeatable; follow the steps in order.

## Versioning

Semantic versioning: `MAJOR.MINOR.PATCH`.

* **PATCH** — fixes and content tweaks, no behaviour change for users.
* **MINOR** — new features, backward compatible.
* **MAJOR** — a change that breaks existing URLs, config, or saved birth-detail JSON.

The version lives in `Cargo.toml` (`package.version`). Tags are `v<version>` (e.g. `v0.2.0`).

## 1. Pre-flight

```sh
./scripts/check.sh          # clippy, tests, structure caps — must be green
docker compose up -d --build
curl -fsS http://localhost:8080/health   # expect 200
```

Smoke-test by hand: generate a report, download the PDF, and (if an AI service is configured) ask a
question and export the Q&A. Then `docker compose down`.

## 2. Bump the version

* Edit `version` in `Cargo.toml`.
* Run `cargo build` so `Cargo.lock` updates.
* Update `CHANGELOG.md` (create it on the first release): a short list under the new version of
  Added / Changed / Fixed, newest on top.

```sh
git add Cargo.toml Cargo.lock CHANGELOG.md
git commit -m "chore(release): v<version>"
```

## 3. Tag and push

```sh
git tag -a v<version> -m "v<version>"
git push origin main
git push origin v<version>
```

## 4. Build and export the image

```sh
./scripts/package-web.sh          # builds vedic-astrology-web:latest and exports a tarball to dist/
# or, to tag the image with the version:
docker build -t vedic-astrology-web:<version> -t vedic-astrology-web:latest .
```

The image is built on a Chainguard base, runs as non-root with a read-only root filesystem, and
carries the AGPL licence files from `vendor/swisseph/` and the repo `LICENSE`.

If you push to a registry, tag both `:<version>` and `:latest`, and push both.

## 5. Publish the release

On Gitea, create a release from the `v<version>` tag, paste the changelog section, and attach the
`dist/vedic-astrology-web-<version>-<arch>.tar.gz` tarball (loadable with `docker load -i`).

## Deploying an update

```sh
git pull
docker compose up -d --build       # rebuilds and recreates the container
docker image prune -f              # optional: reclaim space
```

Runtime configuration (AI keys, listen address) is read from `.env`/the environment and is not baked
into the image, so an update never changes your settings. See [README.md](README.md) and
[docs/PACKAGING.md](docs/PACKAGING.md).

## Rollback

```sh
docker compose down
git checkout v<previous-version>
docker compose up -d --build
```

Saved birth-detail JSON and exported reports/Q&A are files on the user's machine and are unaffected
by a rollback.
