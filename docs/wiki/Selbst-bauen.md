# Selbst bauen

## Voraussetzungen

**Rust** über [rustup](https://rustup.rs) (stabile Version).

**Ubuntu / Debian:**

```sh
sudo apt install build-essential libwebkit2gtk-4.1-dev libgtk-3-dev \
  libayatana-appindicator3-dev librsvg2-dev libasound2-dev libdbus-1-dev libssl-dev
```

**Fedora:**

```sh
sudo dnf install webkit2gtk4.1-devel gtk3-devel libappindicator-gtk3-devel \
  librsvg2-devel alsa-lib-devel dbus-devel openssl-devel
```

**Tauri-Werkzeug** (für Pakete und die Android-App):

```sh
cargo install tauri-cli --version "^2" --locked
```

## PC: entwickeln und testen

```sh
git clone https://github.com/Marcel1853/amber-player.git
cd amber-player
cargo test --workspace          # alle Tests
cargo run -p amber-player       # Amber Player starten
```

Eine **Test-Instanz neben dem installierten Player** – eigener Datenordner, die echte
Bibliothek bleibt unberührt:

```sh
AMBER_TEST_INSTANCE=1 XDG_DATA_HOME=$PWD/target/test-data XDG_CONFIG_HOME=$PWD/target/test-data \
  cargo run -p amber-player
```

`AMBER_TEST_INSTANCE=1` schaltet die Sperre „nur ein Player gleichzeitig“ ab.

Die Oberfläche (`crates/app/ui/`) ist reines HTML/CSS/JavaScript ohne Build-Schritt –
Änderungen sind nach einem Neustart der App sichtbar.

## PC: Pakete bauen

```sh
cd crates/app
cargo tauri build                  # .deb, .rpm und AppImage
cargo tauri build --bundles deb    # nur .deb
```

Die Pakete liegen danach in `target/release/bundle/`.

## Android: App bauen

**Zusätzlich nötig:**

- Android SDK mit Build-Tools, Platform-Tools und **NDK** (z. B. über Android Studio oder
  `sdkmanager`),
- Java 17 oder neuer,
- das Rust-Ziel: `rustup target add aarch64-linux-android`.

**Umgebung:**

```sh
export ANDROID_HOME=$HOME/Android/Sdk
export NDK_HOME=$ANDROID_HOME/ndk/<Version>
export JAVA_HOME=/usr/lib/jvm/java-21-openjdk-amd64
```

**Bauen, signieren, installieren** – alles in einem Skript:

```sh
bash scripts/android-apk.sh             # → target/android/AmberPlayer-<Version>.apk
bash scripts/android-apk.sh --install   # zusätzlich per adb aufs Handy (nur Hauptprofil)
```

Das Skript baut eine Release-APK für arm64, richtet sie aus, signiert sie (derzeit mit dem
Debug-Schlüssel aus `~/.android/debug.keystore`) und beendet danach die Gradle- und
Kotlin-Hintergrundprozesse, damit sie keinen Arbeitsspeicher festhalten.

Von Hand:

```sh
cd crates/app
cargo tauri android build --target aarch64 --apk
```

**Protokoll vom Handy** (Fehlersuche):

```sh
adb logcat -v time --pid=$(adb shell pidof de.amberplayer.app)
```

## Visual Studio Code

Im Repository liegen passende Einstellungen (`.vscode/`):

- Java, Gradle, C++ und .NET laden die Android-Projekte **nicht** vor – VS Code bleibt schlank.
- Unter **Ausführen und Debuggen** gibt es fertige Einträge: Test-Instanz, Tests, `.deb` bauen,
  APK bauen und installieren, Handy-Protokoll.

## Versionsnummer

Die Version steht an zwei Stellen und muss gleich sein:

- `Cargo.toml` (Workspace) → `version = "…"`
- `crates/app/tauri.conf.json` → `"version": "…"`

Bei jeder neuen Version beide erhöhen – sonst erkennt die Paketverwaltung das Update nicht.
