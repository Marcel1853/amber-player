fn main() {
    // Android: Die Audioausgabe (Oboe) ist C++ und braucht die C++-Standardbibliothek.
    // Ohne sie lässt sich die App-Bibliothek nicht laden und die App stürzt sofort ab.
    // Gelinkt wird gegen libc++_shared.so aus dem NDK, so wie es Google für Apps mit
    // C++-Anteil empfiehlt.
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("android") {
        android_cxx();
    }
    tauri_build::build()
}

fn android_cxx() {
    let ndk = std::env::var("NDK_HOME")
        .or_else(|_| std::env::var("ANDROID_NDK_HOME"))
        .expect("NDK_HOME muss für den Android-Build gesetzt sein");
    let triple = match std::env::var("CARGO_CFG_TARGET_ARCH").as_deref() {
        Ok("aarch64") => "aarch64-linux-android",
        Ok("arm") => "arm-linux-androideabi",
        Ok("x86_64") => "x86_64-linux-android",
        Ok("x86") => "i686-linux-android",
        other => panic!("Unbekannte Android-Architektur: {other:?}"),
    };
    let lib_dir = format!("{ndk}/toolchains/llvm/prebuilt/linux-x86_64/sysroot/usr/lib/{triple}");
    // Nur diese eine Datei in einen eigenen Suchpfad legen: Den ganzen NDK-Ordner als
    // Suchpfad anzugeben, zieht sonst die statische libc.a mit hinein (doppelte Symbole).
    let out = std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap()).join("cxx");
    std::fs::create_dir_all(&out).expect("Ordner für libc++ anlegen");
    std::fs::copy(format!("{lib_dir}/libc++_shared.so"), out.join("libc++_shared.so")).expect("libc++_shared.so kopieren");
    println!("cargo:rustc-link-search=native={}", out.display());
    println!("cargo:rustc-link-lib=dylib=c++_shared");
    println!("cargo:rerun-if-env-changed=NDK_HOME");

    // In die APK (jniLibs) bringt Tauri die Datei selbst, sobald die App sie braucht.
    // Nicht selbst dorthin kopieren: Dort liegt ein Verweis auf die NDK-Datei, eine Kopie
    // würde die Originaldatei im NDK leeren.
}
