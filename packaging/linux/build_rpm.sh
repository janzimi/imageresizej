#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
RPM_TOP="$ROOT/target/rpmbuild"

cd "$ROOT"
cargo build --release -p resizer-worker -p resizer-ui

rm -rf "$RPM_TOP"
mkdir -p "$RPM_TOP"/{BUILD,RPMS,SOURCES,SPECS,SRPMS,TMP}

rpmbuild -bb \
    --define "_topdir $RPM_TOP" \
    --define "_tmppath $RPM_TOP/TMP" \
    --define "proj_root $ROOT" \
    "$ROOT/packaging/linux/imageresizej.spec"

mkdir -p "$ROOT/dist"
cp -f "$RPM_TOP"/RPMS/x86_64/imageresizej-*.rpm "$ROOT/dist/"

echo "Готово:"
ls -lh "$ROOT"/dist/imageresizej-*.rpm
