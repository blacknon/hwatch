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

download_and_verify_upstream_tarball() {
  local version="$1"
  local workdir="$2"
  local tarball_name="hwatch-${version}.tar.gz"
  local tarball_url="https://github.com/blacknon/hwatch/releases/download/${version}/${tarball_name}"
  local sig_url="${tarball_url}.asc"
  local downloaded_tar="$workdir/${tarball_name}"
  local downloaded_sig="${downloaded_tar}.asc"
  local orig_tar="$workdir/hwatch_${version}.orig.tar.gz"
  local orig_sig="${orig_tar}.asc"
  local keyring="$workdir/upstream-signing-key.gpg"
  local extracted_root

  curl -fsSL -o "$downloaded_tar" "$tarball_url"
  curl -fsSL -o "$downloaded_sig" "$sig_url"

  gpg --batch --no-default-keyring --keyring "$keyring" \
    --import "$REPO_DIR/package/debian/upstream/signing-key.asc"
  gpgv --keyring "$keyring" "$downloaded_sig" "$downloaded_tar"

  cp "$downloaded_tar" "$orig_tar"
  cp "$downloaded_sig" "$orig_sig"

  extracted_root="$(tar -tzf "$downloaded_tar" | head -n1 | cut -d/ -f1)"
  tar -xzf "$downloaded_tar" -C "$workdir"

  if [[ -z "$extracted_root" || ! -d "$workdir/$extracted_root" ]]; then
    echo "failed to determine extracted upstream tarball root for: $downloaded_tar" >&2
    exit 1
  fi

  printf '%s\n' "$workdir/$extracted_root"
}

build_source_package() {
  install_build_deps

  VERSION="$(dpkg-parsechangelog -l"$REPO_DIR/package/debian/changelog" -SVersion | sed 's/-[^-]*$//')"
  WORKDIR="$(mktemp -d)"
  SRCDIR=""

  cleanup() {
    rm -rf "$WORKDIR"
  }
  trap cleanup EXIT

  mkdir -p "$OUTPUT_DIR"
  SRCDIR="$(download_and_verify_upstream_tarball "$VERSION" "$WORKDIR")"
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
