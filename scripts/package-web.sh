#!/bin/sh
# Build the web app image on the Chainguard base and export it for deployment.
#   dist/vedic-astrology-web-<version>-<arch>.tar.gz   (load on a server with: docker load -i <file>)
# Options (environment):
#   WITH_TEXTS=0      skip downloading the public-domain classical texts into the image
#   PLATFORMS=...     e.g. linux/amd64,linux/arm64 to build a multi-architecture image with buildx
#                     and push it (also set IMAGE to your registry/name)
set -eu
cd "$(dirname "$0")/.."
VERSION=$(sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -1)
IMAGE=${IMAGE:-vedic-astrology-web}
WITH_TEXTS=${WITH_TEXTS:-1}

if [ -n "${PLATFORMS:-}" ]; then
  docker buildx build --platform "$PLATFORMS" --build-arg WITH_TEXTS="$WITH_TEXTS" \
    -t "$IMAGE:$VERSION" -t "$IMAGE:latest" --push .
  echo "Pushed $IMAGE:$VERSION for $PLATFORMS"
  exit 0
fi

docker build --build-arg WITH_TEXTS="$WITH_TEXTS" -t "$IMAGE:$VERSION" -t "$IMAGE:latest" .
mkdir -p dist
ARCH=$(docker image inspect "$IMAGE:$VERSION" --format '{{.Architecture}}')
OUT="dist/vedic-astrology-web-${VERSION}-${ARCH}.tar.gz"
docker save "$IMAGE:$VERSION" | gzip > "$OUT"
echo "Web image: $IMAGE:$VERSION  (exported to $OUT)"
