# ImageResizerJ

Быстрая нативная утилита на **Rust** для пакетного сжатия, изменения размера и конвертации изображений прямо из контекстного меню файлового менеджера.

![Version](https://img.shields.io/badge/version-0.3.1-orange)
![Platform](https://img.shields.io/badge/platform-Windows%2010%2F11%20%7C%20ALT%20Linux-blue)
![License](https://img.shields.io/badge/license-MIT-green)

![Настройки ImageResizerJ](assets/screenshot.png)

## Возможности

* **Запуск в один клик:** выделяете одно или несколько изображений → ПКМ → **«Сжать с ImageResizerJ»**.
* **Поддерживаемые форматы:**
  * **Чтение:** JPG, PNG, WebP, HEIC/HEIF, AVIF, BMP, TIFF.
  * **Сохранение:** JPG, PNG, WebP, AVIF.
* **Гибкие настройки:** выбор целевой ширины с сохранением пропорций, защита от увеличения маленьких фото, регулировка качества, опция удаления или сохранения оригиналов.
* **Нативная интеграция с ОС:**
  * **Windows 10 / 11:** современное контекстное меню Проводника (Sparse MSIX + Shell Extension), интерфейс на WebView2, компактные всплывающие уведомления.
  * **Альт Linux (KDE Plasma 6):** сервисное меню Dolphin верхнего уровня, интерфейс на WebKitGTK (X11 и Wayland), нативные уведомления через D-Bus.
* **Минимальная нагрузка:** компактные нативные бинарники без тяжелых рантаймов; поддержка светлой, тёмной и системной темы.

---

## Установка

Готовые установочные файлы доступны в разделе [Releases](https://github.com/janzimi/imageresizej/releases/latest) (зеркало на [GitVerse](https://gitverse.ru/1oblivka/imageresizej/releases)).

### Windows 10 / Windows 11 (x64)

1. Скачайте `ImageResizerJ_Setup_0.3.1.exe` со страницы релизов.
2. Запустите установщик — он установит приложение, зарегистрирует пункт контекстного меню Проводника и добавит ярлык настроек в меню «Пуск».

### Альт Образование 11.2 / Альт Рабочая станция K (x86_64, KDE Plasma 6)

Скачайте `.rpm`-пакет и установите через `apt-get` (системные библиотеки `libheif` и `webkit2gtk` подтянутся автоматически):

```bash
su -
apt-get install -y /путь/к/imageresizej-0.3.1-alt1.x86_64.rpm
```

После установки:
* Контекстное меню **«Сжать с ImageResizerJ»** доступно в Dolphin при выделении файлов изображений.
* Настройки открываются из меню приложений → **Графика → ImageResizerJ**.

---

## Настройки

При первом запуске создаётся файл конфигурации со следующими параметрами по умолчанию:

| Параметр | Значение по умолчанию |
| --- | --- |
| Формат сохранения | JPG |
| Качество сжатия | 90% |
| Целевая ширина | 1024 px |
| Не увеличивать маленькие | Включено |
| Удалять оригиналы | Включено |
| Уведомления о завершении | Включено |
| Язык интерфейса | Авто (RU / EN по языку ОС) |
| Тема оформления | Авто (системная) |

**Расположение файла конфигурации:**
* **Windows:** `%APPDATA%\ImageResizerJ\config\config.json`
* **Linux:** `~/.config/imageresizerj/config.json`

---

## Структура проекта

* `crates/resizer-core` — чтение/кодирование изображений (включая HEIC через `libheif-rs`), ресайз и управление конфигурацией.
* `crates/resizer-worker` — фоновый обработчик пакетных задач и отправка уведомлений.
* `crates/resizer-ui` — окно настроек (и toast-карточка для Windows) на базе `wry` / `tao`.
* `crates/resizer-win-shell` — COM-расширение `IExplorerCommand` для контекстного меню Windows 11.
* `packaging/windows` — скрипты сборки Sparse MSIX и установщика Inno Setup 6.
* `packaging/linux` — `.spec`-файл, `.desktop`-интеграция для KDE/Dolphin и скрипт сборки `.rpm`.

---

## Сборка из исходников

### Windows 10 / 11

**Требования:** Rust (MSVC toolchain), Visual Studio Build Tools (C++ и Windows SDK), `vcpkg`, Inno Setup 6.

```powershell
# Установка статической библиотеки libheif (с декодерами HEIC и AVIF)
vcpkg install "libheif[libde265,dav1d]:x64-windows-static-md" --recurse

# Сборка всех компонентов
cargo build --release --workspace

# Сборка MSIX и установщика
powershell -ExecutionPolicy Bypass -File .\packaging\windows\build_msix.ps1
& "C:\Program Files (x86)\Inno Setup 6\ISCC.exe" .\packaging\windows\installer.iss
```

### Альт Linux (x86_64)

**Требования:** Rust, `gcc`, `pkg-config`, `rpm-build`, пакеты разработки `libheif`, `gtk+3` и `webkit2gtk4.1`.

```bash
# Сборка бинарников и RPM-пакета в каталог dist/
./packaging/linux/build_rpm.sh
```

---

## Лицензия

Проект распространяется под лицензией [MIT](LICENSE).