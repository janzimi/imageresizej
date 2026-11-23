%add_debuginfo_skiplist %_bindir/*

Name:           imageresizej
Version:        0.3.3
Release:        alt1
Summary:        Fast batch image resizer and converter
License:        MIT
Group:          Graphics
ExclusiveArch:  x86_64

%description
ImageResizerJ is a fast desktop utility for batch resizing and converting
images (JPG, PNG, WebP, AVIF, HEIC) with KDE Dolphin context menu integration.

%install
install -Dm0755 "%{proj_root}/target/release/imageresizej" \
    "%buildroot%_bindir/imageresizej"
install -Dm0755 "%{proj_root}/target/release/resizer-worker" \
    "%buildroot%_bindir/resizer-worker"
install -Dm0644 "%{proj_root}/assets/imageresizej.svg" \
    "%buildroot%_datadir/icons/hicolor/scalable/apps/imageresizej.svg"
install -Dm0644 "%{proj_root}/packaging/linux/imageresizej.desktop" \
    "%buildroot%_datadir/applications/imageresizej.desktop"
install -Dm0644 "%{proj_root}/packaging/linux/imageresizej-servicemenu.desktop" \
    "%buildroot%_datadir/kio/servicemenus/imageresizej.desktop"

%post
touch -c /usr/share/icons/hicolor &>/dev/null || :
gtk-update-icon-cache -f -t /usr/share/icons/hicolor &>/dev/null || :
update-desktop-database -q /usr/share/applications &>/dev/null || :

%postun
touch -c /usr/share/icons/hicolor &>/dev/null || :
gtk-update-icon-cache -f -t /usr/share/icons/hicolor &>/dev/null || :
update-desktop-database -q /usr/share/applications &>/dev/null || :

%files
%_bindir/imageresizej
%_bindir/resizer-worker
%_datadir/icons/hicolor/scalable/apps/imageresizej.svg
%_datadir/applications/imageresizej.desktop
%_datadir/kio/servicemenus/imageresizej.desktop

%changelog
* Mon Nov 23 2026 Vladimir Piskov <1oblivka@mail.ru> 0.3.3-alt1
- Add .deb and .tar.xz Linux packaging scripts and CI workflow
- Add .jfif, .jpe, and .hif context menu associations on Windows

* Sun Nov 22 2026 Vladimir Piskov <1oblivka@mail.ru> 0.3.2-alt1
- Add dismissible first-run usage hint in settings
- Fix frameless window minimize/restore on Windows and refine titlebar layout

* Sun Nov 22 2026 Vladimir Piskov <1oblivka@mail.ru> 0.3.1-alt1
- Add RU/EN localization with automatic OS language detection

* Sat Nov 21 2026 Vladimir Piskov <1oblivka@mail.ru> 0.3.0-alt1
- Initial ALT Education 11.2 (KDE Plasma 6) package
