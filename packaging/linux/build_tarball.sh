#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
VERSION="$(sed -n 's/^version = "\([^"]*\)"/\1/p' "$ROOT/Cargo.toml" | head -n1)"
if [[ -z "$VERSION" ]]; then
    echo "Failed to read version from Cargo.toml" >&2
    exit 1
fi

ARCH="$(uname -m)"
TARGET_DIR="${CARGO_TARGET_DIR:-$ROOT/target}"
STAGE_DIR="$TARGET_DIR/tarbuild"
PKG_NAME="imageresizej-${VERSION}-linux-${ARCH}"
PKG_DIR="$STAGE_DIR/$PKG_NAME"
OUT_TAR="$ROOT/dist/${PKG_NAME}.tar.xz"

cd "$ROOT"
cargo build --release -p resizer-worker -p resizer-ui

rm -rf "$STAGE_DIR"
mkdir -p "$PKG_DIR"

install -Dm0755 "$TARGET_DIR/release/imageresizej" \
    "$PKG_DIR/bin/imageresizej"
install -Dm0755 "$TARGET_DIR/release/resizer-worker" \
    "$PKG_DIR/bin/resizer-worker"
install -Dm0644 "$ROOT/assets/imageresizej.svg" \
    "$PKG_DIR/share/icons/hicolor/scalable/apps/imageresizej.svg"
install -Dm0644 "$ROOT/packaging/linux/imageresizej.desktop" \
    "$PKG_DIR/share/applications/imageresizej.desktop"
install -Dm0644 "$ROOT/packaging/linux/imageresizej-servicemenu.desktop" \
    "$PKG_DIR/share/kio/servicemenus/imageresizej.desktop"

strip "$PKG_DIR/bin/imageresizej" "$PKG_DIR/bin/resizer-worker" 2>/dev/null || true

cat > "$PKG_DIR/install.sh" <<'EOF'
#!/bin/sh
set -eu

SCRIPT_DIR="$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)"

if [ "$(id -u)" -eq 0 ]; then
    PREFIX="${PREFIX:-/usr/local}"
else
    PREFIX="${PREFIX:-$HOME/.local}"
fi

install -Dm0755 "$SCRIPT_DIR/bin/imageresizej" \
    "$PREFIX/bin/imageresizej"
install -Dm0755 "$SCRIPT_DIR/bin/resizer-worker" \
    "$PREFIX/bin/resizer-worker"
install -Dm0644 "$SCRIPT_DIR/share/icons/hicolor/scalable/apps/imageresizej.svg" \
    "$PREFIX/share/icons/hicolor/scalable/apps/imageresizej.svg"

mkdir -p "$PREFIX/share/applications" "$PREFIX/share/kio/servicemenus"

sed -E "s|^Exec=(.*/)?imageresizej|Exec=${PREFIX}/bin/imageresizej|" \
    "$SCRIPT_DIR/share/applications/imageresizej.desktop" \
    > "$PREFIX/share/applications/imageresizej.desktop"
chmod 0644 "$PREFIX/share/applications/imageresizej.desktop"

sed -E "s|^Exec=(.*/)?resizer-worker|Exec=${PREFIX}/bin/resizer-worker|" \
    "$SCRIPT_DIR/share/kio/servicemenus/imageresizej.desktop" \
    > "$PREFIX/share/kio/servicemenus/imageresizej.desktop"
chmod 0644 "$PREFIX/share/kio/servicemenus/imageresizej.desktop"

if command -v gtk-update-icon-cache >/dev/null 2>&1; then
    gtk-update-icon-cache -f -t "$PREFIX/share/icons/hicolor" >/dev/null 2>&1 || true
fi
if command -v update-desktop-database >/dev/null 2>&1; then
    update-desktop-database -q "$PREFIX/share/applications" >/dev/null 2>&1 || true
fi
if [ "$(id -u)" -ne 0 ] && command -v kbuildsycoca6 >/dev/null 2>&1; then
    kbuildsycoca6 >/dev/null 2>&1 || true
fi

echo "Installed ImageResizerJ to $PREFIX"
EOF
chmod 0755 "$PKG_DIR/install.sh"

cat > "$PKG_DIR/uninstall.sh" <<'EOF'
#!/bin/sh
set -eu

if [ "$(id -u)" -eq 0 ]; then
    PREFIX="${PREFIX:-/usr/local}"
else
    PREFIX="${PREFIX:-$HOME/.local}"
fi

rm -f \
    "$PREFIX/bin/imageresizej" \
    "$PREFIX/bin/resizer-worker" \
    "$PREFIX/share/icons/hicolor/scalable/apps/imageresizej.svg" \
    "$PREFIX/share/applications/imageresizej.desktop" \
    "$PREFIX/share/kio/servicemenus/imageresizej.desktop"

if command -v gtk-update-icon-cache >/dev/null 2>&1; then
    gtk-update-icon-cache -f -t "$PREFIX/share/icons/hicolor" >/dev/null 2>&1 || true
fi
if command -v update-desktop-database >/dev/null 2>&1; then
    update-desktop-database -q "$PREFIX/share/applications" >/dev/null 2>&1 || true
fi
if [ "$(id -u)" -ne 0 ] && command -v kbuildsycoca6 >/dev/null 2>&1; then
    kbuildsycoca6 >/dev/null 2>&1 || true
fi

echo "Removed ImageResizerJ from $PREFIX"
EOF
chmod 0755 "$PKG_DIR/uninstall.sh"

mkdir -p "$ROOT/dist"
tar --owner=0 --group=0 -C "$STAGE_DIR" -cJf "$OUT_TAR" "$PKG_NAME"

ls -lh "$OUT_TAR"