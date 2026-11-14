//! Quellen-Plugin (JavaScript) ausprobieren, ohne Amber Player zu starten.
//!
//! ```sh
//! cargo run -p amber-core --example source-probe -- <Paket-Ordner> search "suchbegriff"
//! cargo run -p amber-core --example source-probe -- <Paket-Ordner> playlist "https://…"
//! cargo run -p amber-core --example source-probe -- <Paket-Ordner> resolve <id> [--decode]
//! ```
//!
//! Der Paket-Ordner enthält plugin.json (mit „permissions.network“) und die Quelle. Mit
//! `--decode` wird die gelieferte Adresse geöffnet und ein Stück dekodiert (ohne Tonausgabe) –
//! so sieht man, ob der Player die Adresse wirklich abspielen kann.

use amber_core::jssource::JsSource;
use amber_core::packages::{Kind, Manifest};
use rodio::Source;
use serde_json::json;

fn main() -> anyhow::Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let (dir, action, arg) = match args.as_slice() {
        [d, a, x, ..] => (std::path::PathBuf::from(d), a.as_str(), x.clone()),
        _ => anyhow::bail!("Aufruf: source-probe <Paket-Ordner> search|playlist|resolve <Wert> [--decode]"),
    };
    let manifest = Manifest::parse(&std::fs::read_to_string(dir.join("plugin.json"))?)?;
    let part = manifest
        .contents
        .iter()
        .find(|c| c.kind == Kind::Source)
        .ok_or_else(|| anyhow::anyhow!("Das Paket enthält keine Online-Quelle"))?;
    let code = std::fs::read_to_string(dir.join(&part.file))?;
    let start = std::time::Instant::now();
    let source = JsSource::start(&part.file, code, manifest.permissions.network.clone())?;
    eprintln!("Quelle „{}“ geladen ({:?})", source.info.name, start.elapsed());

    let start = std::time::Instant::now();
    let result = match action {
        "search" => source.call("search", json!([arg, 10]))?,
        "playlist" => source.call("playlist", json!([arg]))?,
        "resolve" => source.call("resolve", json!([arg]))?,
        other => anyhow::bail!("Unbekannte Aktion {other}"),
    };
    eprintln!("{action}: {:?}", start.elapsed());
    match action {
        "playlist" => {
            let items = result["items"].as_array().map(Vec::len).unwrap_or(0);
            println!("{} – {items} Titel", result["name"]);
            for it in result["items"].as_array().into_iter().flatten().take(5) {
                println!("  {} | {} – {} ({} ms)", it["id"], it["artist"], it["title"], it["duration_ms"]);
            }
        }
        "search" => {
            for it in result.as_array().into_iter().flatten() {
                println!("{} | {} – {} ({} ms)", it["id"], it["artist"], it["title"], it["duration_ms"]);
            }
        }
        _ => {
            let url = result["url"].as_str().unwrap_or("");
            println!("Adresse: {}…", &url[..url.len().min(90)]);
            if args.iter().any(|a| a == "--decode") {
                let headers = serde_json::from_value(result["headers"].clone()).unwrap_or_default();
                let start = std::time::Instant::now();
                let mut src = amber_core::httpsource::open_url(url, headers)?;
                println!(
                    "Geöffnet nach {:?}: {} Hz, {} Kanäle, Länge {:?}",
                    start.elapsed(),
                    src.sample_rate(),
                    src.channels(),
                    src.total_duration()
                );
                let three_seconds = src.sample_rate() as usize * src.channels() as usize * 3;
                let n = src.by_ref().take(three_seconds).count();
                println!("3 Sekunden dekodiert ({n} Samples)");
                src.try_seek(std::time::Duration::from_secs(60)).map_err(|e| anyhow::anyhow!("{e:?}"))?;
                let n = src.take(1000).count();
                println!("Spulen auf 1:00 ok ({n} Samples danach)");
            }
        }
    }
    Ok(())
}
