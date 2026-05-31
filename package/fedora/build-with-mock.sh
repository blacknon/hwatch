#!/usr/bin/env bash
set -euo pipefail

REPO_DIR="${1:-/work}"
CHROOT="${2:-fedora-rawhide-x86_64}"
OUTPUT_DIR="${OUTPUT_DIR:-$REPO_DIR/out/fedora/mock-${CHROOT}}"
TOPDIR="${TOPDIR:-$REPO_DIR/rpmbuild-mock}"
SPEC_FILE="${SPEC_FILE:-$REPO_DIR/package/fedora/hwatch.spec}"

if [[ ! -f "$SPEC_FILE" ]]; then
  echo "spec file was not found: $SPEC_FILE" >&2
  exit 1
fi

VERSION="$(sed -n 's/^Version:[[:space:]]*//p' "$SPEC_FILE" | head -n1)"
TARBALL="$REPO_DIR/out/fedora/hwatch-${VERSION}.tar.gz"

if [[ -z "$VERSION" ]]; then
  echo "failed to read Version from $SPEC_FILE" >&2
  exit 1
fi

if [[ ! -f "$TARBALL" ]]; then
  echo "source tarball was not found: $TARBALL" >&2
  echo "run 'mise run fedora_source_tarball' on the host first" >&2
  exit 1
fi

SOURCEDIR="$TOPDIR/SOURCES"
SPECDIR="$TOPDIR/SPECS"
SRPMDIR="$TOPDIR/SRPMS"

mkdir -p "$OUTPUT_DIR"
rm -rf "$TOPDIR"
mkdir -p "$SOURCEDIR" "$SPECDIR" "$SRPMDIR"

cp "$TARBALL" "$SOURCEDIR/"
cp "$SPEC_FILE" "$SPECDIR/"

rpmbuild -bs "$SPECDIR/$(basename "$SPEC_FILE")" --define "_topdir $TOPDIR"

SRPM_PATH="$(find "$SRPMDIR" -maxdepth 1 -type f -name 'hwatch-*.src.rpm' | sort | tail -n1)"
if [[ -z "$SRPM_PATH" ]]; then
  echo "failed to locate the generated SRPM under $SRPMDIR" >&2
  exit 1
fi

mock -r "$CHROOT" --rebuild "$SRPM_PATH" --resultdir "$OUTPUT_DIR"

echo "Mock rebuild finished. Results are available at: $OUTPUT_DIR"
