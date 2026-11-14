#!/usr/bin/env bash
# Baut die Android-App (arm64, Release), signiert sie und legt sie als
# target/android/AmberPlayer-<Version>.apk ab. Mit --install danach per adb aufs Handy.
#
# Erwartet das Android-SDK unter ~/projects/test/.android-sdk (anpassbar per ANDROID_HOME).
set -euo pipefail

cd "$(dirname "$0")/.."
export ANDROID_HOME="${ANDROID_HOME:-$HOME/projects/test/.android-sdk}"
export NDK_HOME="${NDK_HOME:-$(ls -d "$ANDROID_HOME"/ndk/* | sort -V | tail -1)}"
export JAVA_HOME="${JAVA_HOME:-/usr/lib/jvm/java-21-openjdk-amd64}"
export PATH="$HOME/.cargo/bin:$PATH"
# Für die JavaScript-Engine (rquickjs): Anbindung mit bindgen erzeugen, libclang und Header aus dem NDK
ndk_llvm="$NDK_HOME/toolchains/llvm/prebuilt/linux-x86_64"
export LIBCLANG_PATH="$ndk_llvm/lib"
# auch clang aus dem NDK – sonst mischt bindgen die Header eines System-clang dazu
export CLANG_PATH="$ndk_llvm/bin/clang"
clang_inc=$(ls -d "$ndk_llvm"/lib/clang/*/include | sort -V | tail -1)
export BINDGEN_EXTRA_CLANG_ARGS_aarch64_linux_android="--target=aarch64-linux-android24 --sysroot=$ndk_llvm/sysroot -isystem $clang_inc"
TAURI="$(command -v cargo-tauri || echo "$HOME/projects/test/.cargo/bin/cargo-tauri")"
BT="$(ls -d "$ANDROID_HOME"/build-tools/* | sort -V | tail -1)"

version=$(grep -m1 '^version' Cargo.toml | cut -d'"' -f2)
echo "== Amber Player $version für Android bauen"
(cd crates/app && "$TAURI" android build --target aarch64 --apk)

unsigned=crates/app/gen/android/app/build/outputs/apk/universal/release/app-universal-release-unsigned.apk
out="target/android/AmberPlayer-$version.apk"
mkdir -p target/android
rm -f "$out" target/android/aligned.apk
"$BT/zipalign" -p 4 "$unsigned" target/android/aligned.apk
# Signieren: mit dem Release-Schlüssel aus keystore/ (derselbe wie bei GitHub), sonst mit dem
# Debug-Schlüssel. Achtung: Apps mit verschiedenen Schlüsseln lassen sich nicht übereinander
# installieren.
props=keystore/keystore.properties
if [[ -f "$props" && -f keystore/rotation.lineage ]]; then
  # Schlüssel-Rotation: Die ersten Versionen waren mit dem Debug-Schlüssel signiert. Mit der
  # Rotations-Bestätigung (lineage) nimmt Android Updates mit dem neuen Schlüssel als normales
  # Update an – ohne Deinstallieren, alle Daten bleiben.
  prop() { sed -n "s/^$1=//p" "$props"; }
  echo "== Signieren mit dem Release-Schlüssel (Rotation vom Debug-Schlüssel)"
  STORE_PW=$(prop storePassword) KEY_PW=$(prop keyPassword) \
    "$BT/apksigner" sign \
    --ks "$HOME/.android/debug.keystore" --ks-key-alias androiddebugkey --ks-pass pass:android --key-pass pass:android \
    --next-signer --ks "keystore/$(prop storeFile)" --ks-key-alias "$(prop keyAlias)" --ks-pass env:STORE_PW --key-pass env:KEY_PW \
    --lineage keystore/rotation.lineage --out "$out" target/android/aligned.apk
elif [[ -f "$props" ]]; then
  prop() { sed -n "s/^$1=//p" "$props"; }
  echo "== Signieren mit dem Release-Schlüssel"
  STORE_PW=$(prop storePassword) KEY_PW=$(prop keyPassword) \
    "$BT/apksigner" sign --ks "keystore/$(prop storeFile)" --ks-key-alias "$(prop keyAlias)" \
    --ks-pass env:STORE_PW --key-pass env:KEY_PW --out "$out" target/android/aligned.apk
else
  echo "== Kein Release-Schlüssel (keystore/keystore.properties) – signiere mit dem Debug-Schlüssel"
  "$BT/apksigner" sign --ks "$HOME/.android/debug.keystore" --ks-key-alias androiddebugkey \
    --ks-pass pass:android --key-pass pass:android --out "$out" target/android/aligned.apk
fi
rm -f target/android/aligned.apk "$out.idsig"
echo "== Fertig: $out ($(du -h "$out" | cut -f1))"

# Gradle- und Kotlin-Hintergrundprozesse beenden – sie halten sonst mehrere GB Arbeitsspeicher fest.
(cd crates/app/gen/android && ./gradlew --stop >/dev/null 2>&1) || true
pkill -f org.jetbrains.kotlin.daemon.KotlinCompileDaemon 2>/dev/null || true

if [[ "${1:-}" == "--install" ]]; then
  adb="$ANDROID_HOME/platform-tools/adb"
  echo "== Aufs Handy installieren"
  # Nur fürs Hauptprofil – sonst landet die App auch im Samsung-Profil „Dual Messenger“ (doppeltes Symbol).
  "$adb" install -r --user 0 "$out"
fi
