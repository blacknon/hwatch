#!/usr/bin/env bash
set -euo pipefail

REPO_DIR="${1:-/work}"
ACTION="${2:-build-source}"
OUTPUT_DIR="${OUTPUT_DIR:-$REPO_DIR/out/debian-source}"
SIGN_SOURCE_PACKAGE="${SIGN_SOURCE_PACKAGE:-0}"
DPUT_TARGET="${DPUT_TARGET:-}"

if [[ ! -f "$REPO_DIR/package/debian/changelog" ]]; then
  echo "package/debian/changelog was not found under: $REPO_DIR" >&2
  exit 1
fi

install_build_deps() {
  apt-get update
  mk-build-deps \
    --install \
    --remove \
    --tool 'apt-get --no-install-recommends -y' \
    "$REPO_DIR/package/debian/control"
}

latest_changes_file() {
  find "$OUTPUT_DIR" -maxdepth 1 -type f -name 'hwatch_*.changes' | sort | tail -n1
}

build_source_package() {
  install_build_deps

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
    debsign "$(latest_changes_file)"
  fi

  if [[ -n "$DPUT_TARGET" ]]; then
    dput "$DPUT_TARGET" "$(latest_changes_file)"
  fi

  echo "Debian source package files are available in: $OUTPUT_DIR"
}

run_lintian() {
  changes_file="$(latest_changes_file)"
  if [[ -z "$changes_file" ]]; then
    echo "no hwatch_*.changes file found in: $OUTPUT_DIR" >&2
    exit 1
  fi

  lintian -EvIL +pedantic "$changes_file"
}

case "$ACTION" in
  build-source)
    build_source_package
    ;;
  lintian)
    run_lintian
    ;;
  *)
    echo "unknown action: $ACTION" >&2
    echo "usage: hwatch-debian-build [repo-dir] [build-source|lintian]" >&2
    exit 1
    ;;
esac
