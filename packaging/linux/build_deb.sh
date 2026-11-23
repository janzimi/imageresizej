#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
VERSION="$(sed -n 's/^version = "\([^"]*\)"/\1/p' "$ROOT/Cargo.toml" | head -n1)"
if [[ -z "$VERSION" ]]; then
    echo "Failed to read version from Cargo.toml" >&2
    exit 1
fi

ARCH="$(dpkg --print-architecture)"
TARGET_DIR="${CARGO_TARGET_DIR:-$ROOT/target}"
STAGE_DIR="$TARGET_DIR/debbuild"
PKG_DIR="$STAGE_DIR/imageresizej_${VERSION}_${ARCH}"
OUT_DEB="$ROOT/dist/imageresizej_${VERSION}_${ARCH}.deb"

cd "$ROOT"
cargo build --release -p resizer-worker -p resizer-ui

rm -rf "$STAGE_DIR"
mkdir -p "$PKG_DIR/DEBIAN"

install -Dm0755 "$TARGET_DIR/release/imageresizej" \
    "$PKG_DIR/usr/bin/imageresizej"
install -Dm0755 "$TARGET_DIR/release/resizer-worker" \
    "$PKG_DIR/usr/bin/resizer-worker"
install -Dm0644 "$ROOT/assets/imageresizej.svg" \
    "$PKG_DIR/usr/share/icons/hicolor/scalable/apps/imageresizej.svg"
install -Dm0644 "$ROOT/packaging/linux/imageresizej.desktop" \
    "$PKG_DIR/usr/share/applications/imageresizej.desktop"
install -Dm0644 "$ROOT/packaging/linux/imageresizej-servicemenu.desktop" \
    "$PKG_DIR/usr/share/kio/servicemenus/imageresizej.desktop"

strip "$PKG_DIR/usr/bin/imageresizej" "$PKG_DIR/usr/bin/resizer-worker" 2>/dev/null || true

INSTALLED_SIZE="$(du -sk "$PKG_DIR/usr" | cut -f1)"

cat > "$PKG_DIR/DEBIAN/control" <<EOF
Package: imageresizej
Version: ${VERSION}
Section: graphics
Priority: optional
Architecture: ${ARCH}
Maintainer: Vladimir Piskov <1oblivka@mail.ru>
Installed-Size: ${INSTALLED_SIZE}
Depends: libc6, libgtk-3-0t64 | libgtk-3-0, libwebkit2gtk-4.1-0, libheif1, libheif-plugin-libde265, libheif-plugin-dav1d | libheif-plugin-aomdec
Homepage: https://github.com/janzimi/imageresizej
Description: Fast batch image resizer and converter
 ImageResizerJ is a fast desktop utility for batch resizing and converting
 images (JPG, PNG, WebP, AVIF, HEIC) with KDE Dolphin context menu integration.
EOF

for hook in postinst postrm; do
    cat > "$PKG_DIR/DEBIAN/$hook" <<'EOF'
#!/bin/sh
set -e
if command -v gtk-update-icon-cache >/dev/null 2>&1; then
    gtk-update-icon-cache -f -t /usr/share/icons/hicolor >/dev/null 2>&1 || true
fi
if command -v update-desktop-database >/dev/null 2>&1; then
    update-desktop-database -q /usr/share/applications >/dev/null 2>&1 || true
fi
EOF
    chmod 0755 "$PKG_DIR/DEBIAN/$hook"
done

mkdir -p "$ROOT/dist"
dpkg-deb --build --root-owner-group "$PKG_DIR" "$OUT_DEB"

ls -lh "$OUT_DEB"