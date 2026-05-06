#!/usr/bin/env bash
set -euo pipefail

REPO_DIR="${1:-/work}"
OUTPUT_DIR="${OUTPUT_DIR:-$REPO_DIR/out/debian-source}"
SIGN_SOURCE_PACKAGE="${SIGN_SOURCE_PACKAGE:-0}"
DPUT_TARGET="${DPUT_TARGET:-}"

if [[ ! -f "$REPO_DIR/package/debian/changelog" ]]; then
  echo "package/debian/changelog was not found under: $REPO_DIR" >&2
  exit 1
fi

apt-get update
mk-build-deps \
  --install \
  --remove \
  --tool 'apt-get --no-install-recommends -y' \
  "$REPO_DIR/package/debian/control"

VERSION="$(dpkg-parsechangelog -l"$REPO_DIR/package/debian/changelog" -SVersion | sed 's/-[^-]*$//')"
WORKDIR="$(mktemp -d)"
SRCDIR="$WORKDIR/hwatch-$VERSION"
ORIG_TAR="$WORKDIR/hwatch_${VERSION}.orig.tar.gz"

cleanup() {
  rm -rf "$WORKDIR"
}
trap cleanup EXIT

mkdir -p "$SRCDIR" "$OUTPUT_DIR"

rsync -a \
  --exclude '.git' \
  --exclude 'debian' \
  --exclude 'out' \
  --exclude 'package' \
  --exclude 'rpmbuild' \
  --exclude 'target' \
  "$REPO_DIR"/ "$SRCDIR"/

tar -czf "$ORIG_TAR" -C "$WORKDIR" "hwatch-$VERSION"
cp -a "$REPO_DIR/package/debian" "$SRCDIR/debian"

(
  cd "$SRCDIR"
  dpkg-buildpackage -S -sa -us -uc
)

cp -a "$WORKDIR"/hwatch_* "$OUTPUT_DIR"/

if [[ "$SIGN_SOURCE_PACKAGE" == "1" ]]; then
  changes_file="$(find "$OUTPUT_DIR" -maxdepth 1 -type f -name 'hwatch_*.changes' | sort | tail -n1)"
  debsign "$changes_file"
fi

if [[ -n "$DPUT_TARGET" ]]; then
  changes_file="$(find "$OUTPUT_DIR" -maxdepth 1 -type f -name 'hwatch_*.changes' | sort | tail -n1)"
  dput "$DPUT_TARGET" "$changes_file"
fi

echo "Debian source package files are available in: $OUTPUT_DIR"
