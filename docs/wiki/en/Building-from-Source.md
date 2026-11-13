# Building from source

> Note the [license](https://github.com/Marcel1853/amber-player/blob/main/LICENSE): building for
> your own use and for contributing is fine; distributing your own builds is not permitted
> without permission.

## Requirements

**Rust** via [rustup](https://rustup.rs) (stable).

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

**Tauri CLI** (for packages and the Android app):

```sh
cargo install tauri-cli --version "^2" --locked
```

## PC: develop and test

```sh
git clone https://github.com/Marcel1853/amber-player.git
cd amber-player
cargo test --workspace          # all tests
cargo run -p amber-player       # start Amber Player
```

A **test instance next to the installed player** – separate data folder, your real library
stays untouched:

```sh
AMBER_TEST_INSTANCE=1 XDG_DATA_HOME=$PWD/target/test-data XDG_CONFIG_HOME=$PWD/target/test-data \
  cargo run -p amber-player
```

`AMBER_TEST_INSTANCE=1` disables the “only one player at a time” lock.

The interface (`crates/app/ui/`) is plain HTML/CSS/JavaScript without a build step – changes
show up after restarting the app.

## PC: build packages

```sh
cd crates/app
cargo tauri build                  # .deb, .rpm and AppImage
cargo tauri build --bundles deb    # .deb only
```

The packages end up in `target/release/bundle/`.

## Android: build the app

**Additionally required:**

- Android SDK with build tools, platform tools and the **NDK** (e.g. via Android Studio or
  `sdkmanager`),
- Java 17 or newer,
- the Rust target: `rustup target add aarch64-linux-android`.

**Environment:**

```sh
export ANDROID_HOME=$HOME/Android/Sdk
export NDK_HOME=$ANDROID_HOME/ndk/<version>
export JAVA_HOME=/usr/lib/jvm/java-21-openjdk-amd64
```

**Build, sign, install** – all in one script:

```sh
bash scripts/android-apk.sh             # → target/android/AmberPlayer-<version>.apk
bash scripts/android-apk.sh --install   # also install via adb (main profile only)
```

The script builds a release APK for arm64, aligns it, signs it (currently with the debug key from
`~/.android/debug.keystore`) and afterwards stops the Gradle and Kotlin background processes so
they don't hold on to memory.

Manually:

```sh
cd crates/app
cargo tauri android build --target aarch64 --apk
```

**Log from the phone** (troubleshooting):

```sh
adb logcat -v time --pid=$(adb shell pidof de.amberplayer.app)
```

## Visual Studio Code

The repository contains matching settings (`.vscode/`):

- Java, Gradle, C++ and .NET do **not** preload the Android projects – VS Code stays light.
- **Run and Debug** offers ready-made entries: test instance, tests, build `.deb`, build and
  install APK, phone log.

## Version number

The version is in two places and must match:

- `Cargo.toml` (workspace) → `version = "…"`
- `crates/app/tauri.conf.json` → `"version": "…"`

Raise both for every release – otherwise package managers don't recognise the update.
