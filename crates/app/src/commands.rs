//! Befehle, die die Oberfläche über `invoke` aufruft.

use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};

use amber_core::effects::EffectInfo;
use amber_core::{Album, Command, Library, NamedCount, Player, Playlist, Status, Track, TrackQuery};
use serde_json::{json, Value};
use tauri::http::{Request, Response};
use tauri::{AppHandle, Emitter, Manager, Runtime, State, UriSchemeContext, UriSchemeResponder};
use tauri_plugin_dialog::DialogExt;

use crate::{ApiStatus, Ctx};

type R<T> = Result<T, String>;

fn err(e: impl std::fmt::Display) -> String {
    e.to_string()
}

/// Führt blockierende Arbeit (Datenbank, Player-Befehle) außerhalb der async-Threads aus.
async fn bg<T: Send + 'static>(f: impl FnOnce() -> anyhow::Result<T> + Send + 'static) -> R<T> {
    tauri::async_runtime::spawn_blocking(f)
        .await
        .map_err(err)?
        .map_err(|e| format!("{e:#}"))
}

fn lib(ctx: &State<'_, Ctx>) -> Arc<Library> {
    ctx.player.library().clone()
}

fn player(ctx: &State<'_, Ctx>) -> Arc<Player> {
    ctx.player.clone()
}

// ---------- Wiedergabe ----------

#[tauri::command]
pub fn get_status(ctx: State<'_, Ctx>) -> Status {
    ctx.player.status()
}

#[tauri::command]
pub async fn player_command(ctx: State<'_, Ctx>, cmd: Command) -> R<Status> {
    let p = player(&ctx);
    bg(move || Ok(p.send(cmd))).await
}

/// Spielt eine Liste ab `start`; `shuffle` setzt vorher die Zufallswiedergabe.
#[tauri::command]
pub async fn play_ids(ctx: State<'_, Ctx>, ids: Vec<i64>, start: usize, shuffle: Option<bool>) -> R<Status> {
    let p = player(&ctx);
    bg(move || {
        if let Some(on) = shuffle {
            p.send(Command::SetShuffle { on: Some(on) });
        }
        Ok(p.send(Command::PlayList { ids, start }))
    })
    .await
}

#[tauri::command]
pub async fn upcoming(ctx: State<'_, Ctx>, limit: Option<usize>) -> R<Vec<Track>> {
    let p = player(&ctx);
    bg(move || Ok(p.upcoming(limit.unwrap_or(100)))).await
}

// ---------- Bibliothek ----------

#[tauri::command]
pub async fn tracks(ctx: State<'_, Ctx>, query: TrackQuery) -> R<Vec<Track>> {
    let l = lib(&ctx);
    bg(move || l.tracks(&query)).await
}

#[tauri::command]
pub async fn albums(ctx: State<'_, Ctx>) -> R<Vec<Album>> {
    let l = lib(&ctx);
    bg(move || l.albums()).await
}

#[tauri::command]
pub async fn artists(ctx: State<'_, Ctx>) -> R<Vec<NamedCount>> {
    let l = lib(&ctx);
    bg(move || l.artists()).await
}

#[tauri::command]
pub async fn genres(ctx: State<'_, Ctx>) -> R<Vec<NamedCount>> {
    let l = lib(&ctx);
    bg(move || l.genres()).await
}

#[tauri::command]
pub async fn set_favorite(ctx: State<'_, Ctx>, id: i64, favorite: bool) -> R<()> {
    let p = player(&ctx);
    bg(move || {
        p.library().set_favorite(id, favorite)?;
        p.send(Command::Refresh);
        Ok(())
    })
    .await
}

#[tauri::command]
pub async fn library_stats(ctx: State<'_, Ctx>) -> R<Value> {
    let l = lib(&ctx);
    bg(move || {
        Ok(json!({
            "tracks": l.track_count()?,
            "folders": l.folders()?,
            "scanning": l.is_scanning(),
        }))
    })
    .await
}

// ---------- Playlisten ----------

#[tauri::command]
pub async fn playlists(ctx: State<'_, Ctx>) -> R<Vec<Playlist>> {
    let l = lib(&ctx);
    bg(move || l.playlists()).await
}

#[tauri::command]
pub async fn playlist_tracks(ctx: State<'_, Ctx>, id: i64) -> R<Vec<Track>> {
    let l = lib(&ctx);
    bg(move || l.playlist_tracks(id)).await
}

#[tauri::command]
pub async fn create_playlist(ctx: State<'_, Ctx>, name: String, track_ids: Option<Vec<i64>>) -> R<i64> {
    let l = lib(&ctx);
    bg(move || {
        let id = l.create_playlist(&name)?;
        if let Some(ids) = track_ids {
            l.add_to_playlist(id, &ids)?;
        }
        Ok(id)
    })
    .await
}

#[tauri::command]
pub async fn rename_playlist(ctx: State<'_, Ctx>, id: i64, name: String) -> R<()> {
    let l = lib(&ctx);
    bg(move || l.rename_playlist(id, &name)).await
}

#[tauri::command]
pub async fn delete_playlist(ctx: State<'_, Ctx>, id: i64) -> R<()> {
    let l = lib(&ctx);
    bg(move || l.delete_playlist(id)).await
}

#[tauri::command]
pub async fn add_to_playlist(ctx: State<'_, Ctx>, id: i64, track_ids: Vec<i64>) -> R<()> {
    let l = lib(&ctx);
    bg(move || l.add_to_playlist(id, &track_ids)).await
}

#[tauri::command]
pub async fn remove_from_playlist(ctx: State<'_, Ctx>, id: i64, position: usize) -> R<()> {
    let l = lib(&ctx);
    bg(move || l.remove_from_playlist(id, position)).await
}

#[tauri::command]
pub async fn move_in_playlist(ctx: State<'_, Ctx>, id: i64, from: usize, to: usize) -> R<()> {
    let l = lib(&ctx);
    bg(move || l.move_in_playlist(id, from, to)).await
}

#[tauri::command]
pub async fn import_playlist<R2: Runtime>(app: AppHandle<R2>, ctx: State<'_, Ctx>) -> R<Option<i64>> {
    let Some(file) = app
        .dialog()
        .file()
        .set_title("Playlist importieren")
        .add_filter("Playlist", &["m3u", "m3u8"])
        .blocking_pick_file()
    else {
        return Ok(None);
    };
    let path = file.into_path().map_err(err)?;
    let l = lib(&ctx);
    bg(move || l.import_m3u(&path).map(Some)).await
}

#[tauri::command]
pub async fn export_playlist<R2: Runtime>(app: AppHandle<R2>, ctx: State<'_, Ctx>, id: i64) -> R<bool> {
    let l = lib(&ctx);
    let name = {
        let l = l.clone();
        bg(move || l.playlist(id)).await?
    }
    .map(|p| p.name)
    .unwrap_or_else(|| "Playlist".into());
    let Some(file) = app
        .dialog()
        .file()
        .set_title("Playlist exportieren")
        .set_file_name(format!("{name}.m3u8"))
        .add_filter("Playlist", &["m3u8", "m3u"])
        .blocking_save_file()
    else {
        return Ok(false);
    };
    let path: PathBuf = file.into_path().map_err(err)?;
    bg(move || l.export_m3u(id, &path).map(|_| true)).await
}

// ---------- Musikordner ----------

#[tauri::command]
pub async fn folders(ctx: State<'_, Ctx>) -> R<Vec<String>> {
    let l = lib(&ctx);
    bg(move || l.folders()).await
}

#[tauri::command]
pub async fn add_folder<R2: Runtime>(app: AppHandle<R2>, ctx: State<'_, Ctx>) -> R<Option<String>> {
    #[cfg(desktop)]
    let path = {
        let Some(dir) = app
            .dialog()
            .file()
            .set_title("Musikordner hinzufügen")
            .blocking_pick_folder()
        else {
            return Ok(None);
        };
        dir.into_path().map_err(err)?.to_string_lossy().into_owned()
    };
    // Android: vorerst der Musikordner des Handys (eigene Ordnerwahl folgt).
    #[cfg(mobile)]
    let path = default_music_dir().ok_or("Kein Musikordner gefunden")?;
    let l = lib(&ctx);
    let p2 = path.clone();
    bg(move || l.add_folder(&p2)).await?;
    start_scan(app);
    Ok(Some(path))
}

/// Fügt den Standard-Musikordner des Systems hinzu (z. B. ~/Musik).
/// Standard-Musikordner: ~/Musik am PC, „Music“ im internen Speicher auf Android.
fn default_music_dir() -> Option<String> {
    #[cfg(mobile)]
    return Some("/storage/emulated/0/Music".to_string());
    #[cfg(desktop)]
    return dirs::audio_dir().map(|d| d.to_string_lossy().into_owned());
}

#[tauri::command]
pub async fn add_default_folder<R2: Runtime>(app: AppHandle<R2>, ctx: State<'_, Ctx>) -> R<Option<String>> {
    let Some(path) = default_music_dir() else {
        return Ok(None);
    };
    let l = lib(&ctx);
    let p2 = path.clone();
    bg(move || l.add_folder(&p2)).await?;
    start_scan(app);
    Ok(Some(path))
}

#[tauri::command]
pub async fn remove_folder<R2: Runtime>(app: AppHandle<R2>, ctx: State<'_, Ctx>, path: String) -> R<()> {
    let l = lib(&ctx);
    bg(move || l.remove_folder(&path)).await?;
    let _ = app.emit("library-changed", ());
    Ok(())
}

#[tauri::command]
pub fn rescan<R2: Runtime>(app: AppHandle<R2>) {
    start_scan(app);
}

/// Liest die Bibliothek im Hintergrund ein und meldet den Fortschritt als Event.
pub fn start_scan<R2: Runtime>(app: AppHandle<R2>) {
    let lib = app.state::<Ctx>().player.library().clone();
    if lib.is_scanning() {
        return;
    }
    std::thread::spawn(move || {
        let mut last = Instant::now() - Duration::from_secs(1);
        let _ = app.emit("scan-progress", json!({ "done": 0, "total": 0, "running": true }));
        let result = lib.scan_all(&mut |done, total| {
            if last.elapsed() >= Duration::from_millis(150) || done == total {
                last = Instant::now();
                let _ = app.emit("scan-progress", json!({ "done": done, "total": total, "running": true }));
            }
        });
        let payload = match result {
            Ok(r) => json!({ "running": false, "result": r }),
            Err(e) => json!({ "running": false, "error": format!("{e:#}") }),
        };
        let _ = app.emit("scan-progress", payload);
        let _ = app.emit("library-changed", ());
        run_cover_fetch(&app);
    });
}

static COVERS_RUNNING: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

pub fn covers_enabled(lib: &Library) -> bool {
    lib.setting_as("cover_download").unwrap_or(true)
}

/// Fehlende Cover online suchen (im aufrufenden Thread), Fortschritt als Event.
fn run_cover_fetch<R2: Runtime>(app: &AppHandle<R2>) {
    use std::sync::atomic::Ordering;
    let player = app.state::<Ctx>().player.clone();
    let lib = player.library().clone();
    if !covers_enabled(&lib) || COVERS_RUNNING.swap(true, Ordering::AcqRel) {
        return;
    }
    let mut last = Instant::now() - Duration::from_secs(1);
    let result = amber_core::covers::fetch_missing(&lib, &mut |done, total| {
        if total > 0 && (last.elapsed() >= Duration::from_millis(300) || done == total) {
            last = Instant::now();
            let _ = app.emit("cover-progress", json!({ "done": done, "total": total, "running": done < total }));
        }
    });
    COVERS_RUNNING.store(false, Ordering::Release);
    let payload = match result {
        Ok(found) => json!({ "running": false, "found": found }),
        Err(e) => json!({ "running": false, "error": format!("{e:#}") }),
    };
    let _ = app.emit("cover-progress", payload);
    let _ = app.emit("library-changed", ());
    player.send(Command::Refresh);
}

#[tauri::command]
pub fn cover_settings(ctx: State<'_, Ctx>) -> Value {
    json!({ "enabled": covers_enabled(ctx.player.library()) })
}

#[tauri::command]
pub fn set_cover_download<R2: Runtime>(app: AppHandle<R2>, ctx: State<'_, Ctx>, enabled: bool) -> R<()> {
    ctx.player.library().set_setting_json("cover_download", &enabled).map_err(err)?;
    if enabled {
        std::thread::spawn(move || run_cover_fetch(&app));
    }
    Ok(())
}

/// Alle Titel ohne Cover erneut online suchen.
#[tauri::command]
pub fn refetch_covers<R2: Runtime>(app: AppHandle<R2>, ctx: State<'_, Ctx>) -> R<()> {
    ctx.player.library().reset_cover_checks().map_err(err)?;
    std::thread::spawn(move || run_cover_fetch(&app));
    Ok(())
}

// ---------- Effekte ----------

#[tauri::command]
pub fn effects_list(ctx: State<'_, Ctx>) -> Vec<EffectInfo> {
    ctx.player.effects().list()
}

#[tauri::command]
pub fn effect_set(ctx: State<'_, Ctx>, id: String, enabled: Option<bool>, values: Option<Vec<f32>>) -> R<EffectInfo> {
    ctx.player
        .effects()
        .set(&id, enabled, values.as_deref())
        .map_err(|e| format!("{e:#}"))
}

#[tauri::command]
pub fn effect_preset(ctx: State<'_, Ctx>, id: String, preset: String) -> R<EffectInfo> {
    ctx.player
        .effects()
        .apply_preset(&id, &preset)
        .map_err(|e| format!("{e:#}"))
}

#[tauri::command]
pub fn effect_save_preset(ctx: State<'_, Ctx>, id: String, name: String) -> R<EffectInfo> {
    ctx.player.effects().save_preset(&id, &name).map_err(|e| format!("{e:#}"))
}

#[tauri::command]
pub fn effect_delete_preset(ctx: State<'_, Ctx>, id: String, name: String) -> R<EffectInfo> {
    ctx.player.effects().delete_preset(&id, &name).map_err(|e| format!("{e:#}"))
}

#[tauri::command]
pub fn effect_reset(ctx: State<'_, Ctx>, id: String) -> R<EffectInfo> {
    ctx.player.effects().reset(&id).map_err(|e| format!("{e:#}"))
}

// ---------- API / MCP ----------

fn api_json(api: &ApiStatus) -> Value {
    json!({
        "url": api.config.url(),
        "mcp_url": api.config.mcp_url(),
        "port": api.config.port,
        "lan": api.config.lan,
        "token": api.config.token,
        "running": api.running,
        "error": api.error,
    })
}

#[tauri::command]
pub fn api_info(ctx: State<'_, Ctx>) -> Value {
    api_json(&ctx.api.lock().unwrap())
}

/// Speichert neue API-Einstellungen; sie gelten ab dem nächsten Start.
#[tauri::command]
pub fn api_configure(ctx: State<'_, Ctx>, lan: bool, port: u16, new_token: bool) -> R<Value> {
    if port < 1024 {
        return Err("Bitte einen Port ab 1024 wählen".into());
    }
    let mut api = ctx.api.lock().unwrap();
    api.config.lan = lan;
    api.config.port = port;
    if new_token {
        api.config.regenerate_token();
    }
    api.config.save(ctx.player.library());
    let mut v = api_json(&api);
    v["restart_required"] = json!(true);
    Ok(v)
}

// ---------- Cover-Protokoll ----------

/// `cover://localhost/<id>` (Windows: `http://cover.localhost/<id>`) liefert das Cover eines Titels.
pub fn cover_protocol<R2: Runtime>(
    ctx: UriSchemeContext<'_, R2>,
    request: Request<Vec<u8>>,
    responder: UriSchemeResponder,
) {
    let lib = ctx.app_handle().state::<Ctx>().player.library().clone();
    let id = request
        .uri()
        .path()
        .trim_start_matches('/')
        .split(['/', '?'])
        .next()
        .and_then(|s| s.parse::<i64>().ok());
    std::thread::spawn(move || {
        let found = id.and_then(|id| lib.cover(id).ok().flatten());
        let response = match found {
            Some((data, mime)) => Response::builder()
                .status(200)
                .header("Content-Type", mime)
                .header("Cache-Control", "max-age=3600")
                .header("Access-Control-Allow-Origin", "*")
                .body(data),
            None => Response::builder().status(404).body(Vec::new()),
        };
        responder.respond(response.unwrap());
    });
}

// ---------- Visualisierung ----------

#[tauri::command]
pub fn visual_frame(ctx: State<'_, Ctx>, bands: usize, wave_len: usize) -> amber_core::analyzer::VisualFrame {
    ctx.player.visual_frame(bands, wave_len)
}

/// Quelltext aller eigenen Visualisierungs-Plugins (`*.js`, höchstens 512 KB je Datei).
#[tauri::command]
pub fn visualizer_plugins() -> Value {
    let dir = Library::plugin_dir("visualizers");
    let mut plugins: Vec<Value> = std::fs::read_dir(&dir)
        .map(|rd| {
            rd.filter_map(Result::ok)
                .map(|e| e.path())
                .filter(|p| p.extension().is_some_and(|x| x == "js"))
                .filter(|p| std::fs::metadata(p).is_ok_and(|m| m.len() <= 512 * 1024))
                .filter_map(|p| {
                    let code = std::fs::read_to_string(&p).ok()?;
                    let file = p.file_name()?.to_string_lossy().into_owned();
                    Some(json!({ "file": file, "code": code }))
                })
                .collect()
        })
        .unwrap_or_default();
    // Visualisierungen aus Plugin-Paketen
    let (parts, _) = amber_core::packages::contents(amber_core::packages::Kind::Visualizer);
    for (source, code) in parts {
        if let Ok(code) = code {
            plugins.push(json!({ "file": source, "code": code }));
        }
    }
    plugins.sort_by(|a, b| a["file"].as_str().cmp(&b["file"].as_str()));
    json!({ "dir": dir.to_string_lossy(), "plugins": plugins })
}

/// Plugin-Paket installieren (Dateien aus der .amberplugin-/ZIP-Datei, von der Oberfläche entpackt).
/// Klangeffekte wirken nach dem nächsten Start, Visualisierungen nach „Neu laden“.
#[tauri::command]
pub fn install_plugin_package(files: std::collections::HashMap<String, String>) -> R<Value> {
    use amber_core::packages::Kind;
    let m = amber_core::packages::install(&files).map_err(|e| format!("{e:#}"))?;
    Ok(json!({
        "id": m.id, "name": m.name, "version": m.version,
        "effects": m.count(Kind::Effect), "visualizers": m.count(Kind::Visualizer),
        "sources": m.count(Kind::Source),
    }))
}

/// „Erweiterte Plugins erlauben“: Online-Quellen aus Plugin-Paketen dürfen ins Internet
/// (nur zu den Adressen, die das Paket angibt). Gilt nach dem nächsten Start.
#[tauri::command]
pub fn extended_plugins(ctx: State<'_, Ctx>) -> bool {
    ctx.player.library().setting_as::<bool>(amber_core::packages::EXTENDED_SETTING).unwrap_or(false)
}

#[tauri::command]
pub fn set_extended_plugins(ctx: State<'_, Ctx>, on: bool) -> R<()> {
    ctx.player
        .library()
        .set_setting_json(amber_core::packages::EXTENDED_SETTING, &on)
        .map_err(|e| format!("{e:#}"))
}

/// Alle installierten Plugin-Pakete mit Version, Autor und Lizenz.
#[tauri::command]
pub fn plugin_packages() -> Value {
    let (all, errors) = amber_core::packages::list();
    json!({ "packages": all, "errors": errors })
}

/// Plugin-Paket entfernen.
#[tauri::command]
pub fn remove_plugin_package(id: String) -> R<()> {
    amber_core::packages::remove(&id).map_err(|e| format!("{e:#}"))
}

/// Ordner und Ladefehler der Klang-Plugins.
#[tauri::command]
pub fn effects_info(ctx: State<'_, Ctx>) -> Value {
    json!({
        "dir": Library::plugin_dir("effects").to_string_lossy(),
        "errors": ctx.player.effects().load_errors(),
    })
}

/// Plugin-Datei übernehmen (am Handy statt „Ordner öffnen“): Klangeffekte als `.json`,
/// Visualisierungen als `.js`. Effekte werden vorher geprüft, damit kaputte Dateien gar
/// nicht erst im Ordner landen. Gibt den gespeicherten Dateinamen zurück.
#[tauri::command]
pub fn import_plugin(kind: String, file_name: String, content: String) -> R<String> {
    let ext = match kind.as_str() {
        "effects" => "json",
        "visualizers" => "js",
        _ => return Err("Diese Plugin-Art lässt sich nicht importieren".into()),
    };
    if content.len() > 512 * 1024 {
        return Err("Die Datei ist zu groß (höchstens 512 KB)".into());
    }
    // Nur den reinen Dateinamen nehmen und auf harmlose Zeichen beschränken.
    let base = std::path::Path::new(&file_name)
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_default();
    let clean: String = base
        .chars()
        .map(|c| if c.is_alphanumeric() || c == '-' || c == '_' { c } else { '-' })
        .collect();
    let clean = clean.trim_matches('-');
    let name = format!("{}.{ext}", if clean.is_empty() { "plugin" } else { clean });
    let given_ext = std::path::Path::new(&file_name)
        .extension()
        .map(|e| e.to_string_lossy().to_lowercase())
        .unwrap_or_default();
    if given_ext != ext {
        return Err(format!("Erwartet wird eine .{ext}-Datei").into());
    }
    if kind == "effects" {
        amber_core::effects::custom::CustomPlugin::parse(&content, &name).map_err(|e| format!("Das Plugin ist fehlerhaft: {e:#}"))?;
    }
    std::fs::write(Library::plugin_dir(&kind).join(&name), content).map_err(err)?;
    Ok(name)
}

/// Öffnet den Plugin-Ordner (`effects` oder `visualizers`) im Dateimanager.
#[tauri::command]
pub fn open_plugin_dir(kind: String) -> R<()> {
    if !matches!(kind.as_str(), "effects" | "visualizers" | "sources") {
        return Err("Unbekannte Plugin-Art".into());
    }
    let dir = Library::plugin_dir(&kind);
    #[cfg(target_os = "windows")]
    let opener = "explorer";
    #[cfg(target_os = "macos")]
    let opener = "open";
    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    let opener = "xdg-open";
    std::process::Command::new(opener).arg(&dir).spawn().map_err(err)?;
    Ok(())
}

// ---------- Internetradio ----------

#[tauri::command]
pub async fn radio_search(query: String, country: String) -> R<Vec<amber_core::radio::Station>> {
    bg(move || amber_core::radio::search(&query, &country, 80)).await
}

#[tauri::command]
pub async fn stations(ctx: State<'_, Ctx>) -> R<Vec<amber_core::radio::Station>> {
    let l = lib(&ctx);
    bg(move || l.stations()).await
}

#[tauri::command]
pub async fn add_station(ctx: State<'_, Ctx>, station: amber_core::radio::Station) -> R<i64> {
    let l = lib(&ctx);
    bg(move || l.add_station(&station)).await
}

#[tauri::command]
pub async fn remove_station(ctx: State<'_, Ctx>, id: i64) -> R<()> {
    let l = lib(&ctx);
    bg(move || l.remove_station(id)).await
}

// ---------- Streams fürs Handy ----------

/// Adressen, die z. B. Poweramp unter „Streams › URL hinzufügen“ abspielen kann.
#[tauri::command]
pub async fn stream_links(ctx: State<'_, Ctx>) -> R<Value> {
    let cfg = ctx.api.lock().unwrap().config.clone();
    let l = lib(&ctx);
    let playlists = bg(move || l.playlists()).await?;
    let ip = crate::config::lan_ip();
    let base = format!("http://{}:{}", ip.as_deref().unwrap_or("127.0.0.1"), cfg.port);
    let token = format!("?token={}", cfg.token);
    let mut links = vec![
        json!({ "name": "Alle Titel", "url": format!("{base}/stream/all.m3u{token}") }),
        json!({ "name": "Favoriten", "url": format!("{base}/stream/favorites.m3u{token}") }),
    ];
    links.extend(playlists.iter().map(|p| {
        json!({ "name": p.name, "playlist_id": p.id, "url": format!("{base}/stream/playlist/{}.m3u{token}", p.id) })
    }));
    Ok(json!({ "lan": cfg.lan, "ip": ip, "links": links }))
}

/// QR-Code (SVG) für eine Adresse – zum Abscannen mit der Handykamera.
#[tauri::command]
pub fn qr_svg(text: String) -> R<String> {
    let code = qrcode::QrCode::new(text.as_bytes()).map_err(err)?;
    Ok(code
        .render::<qrcode::render::svg::Color>()
        .min_dimensions(220, 220)
        .quiet_zone(true)
        .dark_color(qrcode::render::svg::Color("#1b130a"))
        .light_color(qrcode::render::svg::Color("#ffffff"))
        .build())
}

/// Amber Player neu starten (z. B. nachdem die Freigabe im Heimnetz geändert wurde).
#[tauri::command]
pub fn restart_app<R2: Runtime>(app: AppHandle<R2>) {
    app.restart();
}

// ---------- Firewall (Linux, ufw) ----------

fn ufw_active() -> bool {
    std::process::Command::new("systemctl")
        .args(["is-active", "ufw"])
        .output()
        .is_ok_and(|o| String::from_utf8_lossy(&o.stdout).trim() == "active")
}

/// Ist eine Firewall aktiv, und wurde der Port schon freigegeben?
#[tauri::command]
pub fn firewall_status(ctx: State<'_, Ctx>) -> Value {
    let port = ctx.api.lock().unwrap().config.port;
    let opened: Option<u16> = ctx.player.library().setting_as("firewall_port");
    json!({ "active": ufw_active(), "opened": opened == Some(port), "port": port })
}

/// Gibt den Port der API in der Firewall frei – nur für private Netze (Heimnetz), nicht fürs
/// Internet. pkexec fragt dabei grafisch nach dem Passwort.
#[tauri::command]
pub async fn open_firewall(ctx: State<'_, Ctx>) -> R<()> {
    let port = ctx.api.lock().unwrap().config.port;
    let l = lib(&ctx);
    bg(move || {
        let rules: Vec<String> = ["192.168.0.0/16", "10.0.0.0/8", "172.16.0.0/12"]
            .iter()
            .map(|net| format!("ufw allow from {net} to any port {port} proto tcp comment 'Amber Player'"))
            .collect();
        let out = std::process::Command::new("pkexec")
            .args(["sh", "-c", &rules.join(" && ")])
            .output()
            .map_err(|e| anyhow::anyhow!("pkexec nicht gefunden: {e}"))?;
        if !out.status.success() {
            let code = out.status.code().unwrap_or(-1);
            if code == 126 || code == 127 {
                anyhow::bail!("Abgebrochen – die Firewall wurde nicht geändert");
            }
            anyhow::bail!("Firewall-Freigabe fehlgeschlagen: {}", String::from_utf8_lossy(&out.stderr).trim());
        }
        l.set_setting_json("firewall_port", &port)?;
        Ok(())
    })
    .await
}

// ---------- Online-Quellen (Quellen-Plugins) ----------

#[tauri::command]
pub fn sources_list(ctx: State<'_, Ctx>) -> Value {
    json!({ "sources": ctx.player.sources().list(), "errors": ctx.player.sources().errors })
}

#[tauri::command]
pub async fn source_search(ctx: State<'_, Ctx>, source: String, query: String) -> R<Vec<Track>> {
    let p = player(&ctx);
    bg(move || amber_core::sources::search_tracks(p.library(), p.sources(), Some(&source), &query, 30)).await
}

#[tauri::command]
pub async fn source_playlist(ctx: State<'_, Ctx>, source: String, url: String) -> R<Value> {
    let p = player(&ctx);
    bg(move || {
        let (name, tracks) = amber_core::sources::playlist_tracks(p.library(), p.sources(), Some(&source), &url)?;
        Ok(json!({ "name": name, "tracks": tracks }))
    })
    .await
}

#[tauri::command]
pub async fn source_import(ctx: State<'_, Ctx>, source: String, url: String) -> R<i64> {
    let p = player(&ctx);
    bg(move || amber_core::sources::import_playlist(p.library(), p.sources(), Some(&source), &url)).await
}

// ---------- Verbindung zum PC (vom Handy aus) ----------

use amber_core::pc::PcLink;

fn pc_link_of(lib: &Library) -> anyhow::Result<PcLink> {
    PcLink::load(lib).ok_or_else(|| anyhow::anyhow!("Noch kein PC gekoppelt"))
}

/// Nur Pfade der Player-API zulassen.
fn api_path(path: &str) -> anyhow::Result<&str> {
    if path.starts_with("/api/") && !path.contains("..") {
        Ok(path)
    } else {
        anyhow::bail!("Ungültige Anfrage an den PC")
    }
}

/// Gekoppelter PC (Adresse) oder `null`.
#[tauri::command]
pub fn pc_info(ctx: State<'_, Ctx>) -> Value {
    match PcLink::load(&lib(&ctx)) {
        Some(l) => json!({ "url": l.url }),
        None => Value::Null,
    }
}

/// Mit einem PC koppeln (Link aus dem QR-Code oder Adresse) und die Verbindung prüfen.
#[tauri::command]
pub async fn pc_pair(ctx: State<'_, Ctx>, link: String) -> R<Value> {
    let l = lib(&ctx);
    bg(move || {
        let pc = PcLink::parse(&link)?;
        let status = pc.get("/api/status")?;
        pc.save(&l)?;
        Ok(json!({ "url": pc.url, "status": status }))
    })
    .await
}

#[tauri::command]
pub fn pc_unpair(ctx: State<'_, Ctx>) -> R<()> {
    PcLink::remove(&lib(&ctx)).map_err(|e| format!("{e:#}"))
}

/// GET an den PC, z. B. `/api/status` oder `/api/playlists`.
#[tauri::command]
pub async fn pc_get(ctx: State<'_, Ctx>, path: String) -> R<Value> {
    let l = lib(&ctx);
    bg(move || pc_link_of(&l)?.get(api_path(&path)?)).await
}

/// Befehl an den PC, z. B. `/api/toggle` oder `/api/playlists/3/play`.
#[tauri::command]
pub async fn pc_post(ctx: State<'_, Ctx>, path: String) -> R<Value> {
    let l = lib(&ctx);
    bg(move || pc_link_of(&l)?.post(api_path(&path)?)).await
}

/// Plugin-Paket zum Teilen als `.amberplugin` speichern. Am PC mit „Speichern unter“ (gibt `null`
/// zurück, wenn abgebrochen), am Handy im Cache-Ordner – den Pfad reicht die Oberfläche an
/// das Android-Teilen-Menü weiter.
#[tauri::command]
pub async fn export_plugin_package(app: tauri::AppHandle, id: String) -> R<Option<String>> {
    let data = {
        let id = id.clone();
        bg(move || amber_core::packages::export(&id)).await?
    };
    #[cfg(mobile)]
    let path: PathBuf = {
        use tauri::Manager;
        let dir = app.path().app_cache_dir().map_err(err)?.join("share");
        std::fs::create_dir_all(&dir).map_err(err)?;
        dir.join(format!("{id}.amberplugin"))
    };
    #[cfg(not(mobile))]
    let path: PathBuf = {
        let Some(file) = app
            .dialog()
            .file()
            .set_title("Plugin-Paket exportieren")
            .set_file_name(format!("{id}.amberplugin"))
            .add_filter("Amber-Player-Plugin", &["amberplugin"])
            .blocking_save_file()
        else {
            return Ok(None);
        };
        file.into_path().map_err(err)?
    };
    std::fs::write(&path, data).map_err(err)?;
    Ok(Some(path.to_string_lossy().into_owned()))
}

/// Plugin-Paket vom PC holen und auf dem Handy installieren (mit allen Teilen und der Lizenz).
#[tauri::command]
pub async fn pc_install_plugin(ctx: State<'_, Ctx>, id: String) -> R<Value> {
    let l = lib(&ctx);
    let v = bg(move || {
        anyhow::ensure!(id.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_'), "Ungültige Plugin-ID");
        pc_link_of(&l)?.get(&format!("/api/plugins/{id}/files"))
    })
    .await?;
    let files: std::collections::HashMap<String, String> =
        serde_json::from_value(v["files"].clone()).map_err(|_| "Der PC hat kein gültiges Paket geschickt".to_string())?;
    install_plugin_package(files)
}

/// Cover eines PC-Titels als Bilddaten.
#[tauri::command]
pub async fn pc_cover(ctx: State<'_, Ctx>, id: i64) -> R<tauri::ipc::Response> {
    let l = lib(&ctx);
    let (data, _) = bg(move || pc_link_of(&l)?.bytes(&format!("/api/cover/{id}"))).await?;
    Ok(tauri::ipc::Response::new(data))
}

/// Titel vom PC auf dem Handy abspielen (gestreamt). `album` ist z. B. der Playlist-Name.
#[tauri::command]
pub async fn pc_play_here(ctx: State<'_, Ctx>, tracks: Vec<Track>, start: usize, album: Option<String>) -> R<Status> {
    let l = lib(&ctx);
    let p = player(&ctx);
    bg(move || {
        let pc = pc_link_of(&l)?;
        let ids = pc.import_tracks(&l, &tracks, album.as_deref().unwrap_or("Vom PC"))?;
        Ok(p.send(Command::PlayList { ids, start }))
    })
    .await
}

/// Kopplungs-Link für die Handy-App (steckt im QR-Code am PC).
#[tauri::command]
pub fn pair_link(ctx: State<'_, Ctx>) -> R<Value> {
    let cfg = ctx.api.lock().unwrap().config.clone();
    if !cfg.lan {
        return Ok(json!({ "lan": false }));
    }
    let ip = crate::config::lan_ip().ok_or("Keine Heimnetz-Adresse gefunden")?;
    let url = format!("http://{ip}:{}", cfg.port);
    let enc = |s: &str| s.replace('%', "%25").replace(':', "%3A").replace('/', "%2F");
    Ok(json!({
        "lan": true,
        "url": url,
        "link": format!("amberplayer://pair?url={}&token={}", enc(&url), cfg.token),
    }))
}

/// Senderlogo (Favicon) laden – für die Visualisierung beim Radio. Über den Kern statt direkt
/// in der Oberfläche, weil viele Sender-Server fremde Seiten per CORS aussperren.
#[tauri::command]
pub async fn station_logo(url: String) -> R<tauri::ipc::Response> {
    if !(url.starts_with("http://") || url.starts_with("https://")) {
        return Err("Ungültige Logo-Adresse".into());
    }
    let data = bg(move || {
        let resp = ureq_get(&url)?;
        let mut data = Vec::new();
        std::io::Read::read_to_end(&mut std::io::Read::take(resp.into_reader(), 1024 * 1024), &mut data)?;
        anyhow::ensure!(data.len() > 32, "Logo leer");
        Ok(data)
    })
    .await?;
    Ok(tauri::ipc::Response::new(data))
}

fn ureq_get(url: &str) -> anyhow::Result<ureq::Response> {
    Ok(ureq::get(url).timeout(Duration::from_secs(6)).call()?)
}

// ---------- Updates (Releases auf GitHub) ----------

const RELEASES_API: &str = "https://api.github.com/repos/Marcel1853/amber-player/releases/latest";
const RELEASES_PAGE: &str = "https://github.com/Marcel1853/amber-player/releases";

/// „0.6.2“ bzw. „v0.6.2“ → (0, 6, 2) zum Vergleichen.
fn version_tuple(v: &str) -> (u32, u32, u32) {
    let mut it = v.trim().trim_start_matches('v').split('.').map(|x| {
        x.chars().take_while(char::is_ascii_digit).collect::<String>().parse().unwrap_or(0)
    });
    (it.next().unwrap_or(0), it.next().unwrap_or(0), it.next().unwrap_or(0))
}

/// Fragt bei GitHub nach dem neuesten Release. Gesendet wird nur die Anfrage selbst
/// (mit „AmberPlayer/<Version>“ als Kennung), keine Daten über den Nutzer.
#[tauri::command]
pub async fn check_update() -> R<Value> {
    let current = env!("CARGO_PKG_VERSION").to_string();
    let agent = format!("AmberPlayer/{current}");
    bg(move || {
        let resp = ureq::get(RELEASES_API)
            .set("User-Agent", &agent)
            .set("Accept", "application/vnd.github+json")
            .timeout(Duration::from_secs(8))
            .call();
        let release: Value = match resp {
            Ok(r) => r.into_json()?,
            // Noch kein Release veröffentlicht
            Err(ureq::Error::Status(404, _)) => return Ok(json!({ "current": current, "newer": false })),
            Err(e) => return Err(anyhow::anyhow!("Update-Prüfung nicht möglich: {e}")),
        };
        let latest = release["tag_name"].as_str().unwrap_or("").trim_start_matches('v').to_string();
        let asset = |pred: &dyn Fn(&str) -> bool| {
            release["assets"].as_array().and_then(|a| {
                a.iter()
                    .find(|x| x["name"].as_str().is_some_and(pred))
                    .and_then(|x| x["browser_download_url"].as_str())
                    .map(str::to_string)
            })
        };
        Ok(json!({
            "current": current,
            "latest": latest,
            "newer": !latest.is_empty() && version_tuple(&latest) > version_tuple(&current),
            "page": release["html_url"].as_str().unwrap_or(RELEASES_PAGE),
            "apk": asset(&|n| n.ends_with(".apk")),
            "deb": asset(&|n| n.ends_with(".deb")),
            "rpm": asset(&|n| n.ends_with(".rpm")),
            "appimage": asset(&|n| n.ends_with(".AppImage")),
        }))
    })
    .await
}

/// Eine Adresse von Amber Player auf GitHub im Browser öffnen (am PC; am Handy übernimmt das
/// das Android-Plugin). Andere Adressen werden abgelehnt.
#[tauri::command]
pub fn open_release_url(url: String) -> R<()> {
    if !url.starts_with("https://github.com/Marcel1853/amber-player/") {
        return Err("Nur Adressen von Amber Player auf GitHub".into());
    }
    #[cfg(desktop)]
    {
        std::process::Command::new("xdg-open").arg(&url).spawn().map_err(err)?;
        Ok(())
    }
    #[cfg(mobile)]
    {
        let _ = url;
        Err("Am Handy über das Android-Plugin öffnen".into())
    }
}

#[cfg(test)]
mod update_tests {
    use super::version_tuple;

    #[test]
    fn versions() {
        assert!(version_tuple("v0.6.10") > version_tuple("0.6.9"));
        assert!(version_tuple("1.0.0") > version_tuple("0.99.99"));
        assert_eq!(version_tuple("0.6.2"), version_tuple("v0.6.2"));
        assert_eq!(version_tuple("kaputt"), (0, 0, 0));
    }
}
