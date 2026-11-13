//! Amber Player als Tauri-App – am PC (Fenster) und auf Android.

#[cfg(desktop)]
mod background;
mod commands;
mod config;
#[cfg(desktop)]
mod media;

use std::sync::{Arc, Mutex};

use amber_core::{Library, Player};
use tauri::{Emitter, Manager};

use config::ApiConfig;

/// Gemeinsamer Zustand für alle Tauri-Befehle.
pub struct Ctx {
    pub player: Arc<Player>,
    pub api: Mutex<ApiStatus>,
}

#[derive(Clone, serde::Serialize, Default)]
pub struct ApiStatus {
    pub config: ApiConfig,
    pub running: bool,
    pub error: Option<String>,
}

fn open_player() -> anyhow::Result<Arc<Player>> {
    let lib = Arc::new(Library::open(&Library::default_path())?);
    Ok(Arc::new(Player::new(lib)))
}

/// Hilfetext für die Kommandozeile (nur am PC).
pub const HELP: &str = "amber-player                     Player mit Fenster starten
amber-player --headless          Ohne Fenster: nur Wiedergabe, HTTP-API und MCP
           --music <ORDNER>   Musikordner hinzufügen (mehrfach möglich)
           --port <PORT>      Port der API (Standard 7878)
           --lan              API im Heimnetz freigeben (mit Token)";

#[cfg(desktop)]
fn arg_value<'a>(args: &'a [String], name: &str) -> Vec<&'a str> {
    args.windows(2)
        .filter(|w| w[0] == name)
        .map(|w| w[1].as_str())
        .collect()
}

/// Ohne Fenster: nur Wiedergabe, HTTP-API und MCP (z. B. für einen Rechner am Fernseher).
#[cfg(desktop)]
pub fn run_headless(args: &[String]) -> anyhow::Result<()> {
    let player = open_player()?;
    let lib = player.library().clone();
    for dir in arg_value(args, "--music") {
        lib.add_folder(dir)?;
    }
    let mut cfg = ApiConfig::load(&lib);
    if let Some(port) = arg_value(args, "--port").first() {
        cfg.port = port.parse()?;
    }
    if args.iter().any(|a| a == "--lan") {
        cfg.lan = true;
    }
    cfg.ensure_token();
    cfg.save(&lib);

    let scan_lib = lib.clone();
    let cover_player = player.clone();
    std::thread::spawn(move || {
        match scan_lib.scan_all(&mut |_, _| {}) {
            Ok(r) => log::info!("Bibliothek eingelesen: {} Titel", r.total),
            Err(e) => log::warn!("Einlesen fehlgeschlagen: {e:#}"),
        }
        if commands::covers_enabled(&scan_lib) {
            match amber_core::covers::fetch_missing(&scan_lib, &mut |_, _| {}) {
                Ok(n) => log::info!("{n} Cover aus dem Internet geladen"),
                Err(e) => log::warn!("Cover-Suche: {e:#}"),
            }
            cover_player.send(amber_core::Command::Refresh);
        }
    });

    println!("Amber Player läuft ohne Fenster.");
    println!("  API: {}", cfg.url());
    println!("  MCP: {}", cfg.mcp_url());
    if cfg.lan {
        println!("  Token: {}", cfg.token);
    }
    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(amber_core::api::serve(player, cfg.bind_addr(), cfg.required_token()))
}

/// Startet die App mit Oberfläche (am PC und auf Android).
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    #[cfg(target_os = "android")]
    {
        android_logger::init_once(android_logger::Config::default().with_tag("AmberPlayer").with_max_level(log::LevelFilter::Info));
        // Abstürze mit Meldung ins Handy-Log schreiben, damit sie sich finden lassen.
        std::panic::set_hook(Box::new(|info| log::error!("Absturz: {info}")));
    }
    #[cfg(not(target_os = "android"))]
    let _ = env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).try_init();

    #[allow(unused_mut)]
    let mut builder = tauri::Builder::default();
    #[cfg(desktop)]
    {
        // Zweiter Start (z. B. aus dem Startmenü) zeigt das laufende Fenster statt eines zweiten Players.
        // AMBER_TEST_INSTANCE=1 schaltet das für Tests neben einem laufenden Player ab.
        if std::env::var_os("AMBER_TEST_INSTANCE").is_none() {
            builder = builder.plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| background::show_main(app)));
        }
        builder = builder.on_window_event(background::on_window_event);
    }
    builder
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_amber_mobile::init())
        .register_asynchronous_uri_scheme_protocol("cover", commands::cover_protocol)
        .invoke_handler(tauri::generate_handler![
            commands::get_status,
            commands::player_command,
            commands::play_ids,
            commands::upcoming,
            commands::tracks,
            commands::albums,
            commands::artists,
            commands::genres,
            commands::set_favorite,
            commands::playlists,
            commands::playlist_tracks,
            commands::create_playlist,
            commands::rename_playlist,
            commands::delete_playlist,
            commands::add_to_playlist,
            commands::remove_from_playlist,
            commands::move_in_playlist,
            commands::import_playlist,
            commands::export_playlist,
            commands::folders,
            commands::add_folder,
            commands::add_default_folder,
            commands::remove_folder,
            commands::rescan,
            commands::library_stats,
            commands::effects_list,
            commands::effect_set,
            commands::effect_preset,
            commands::api_info,
            commands::api_configure,
            commands::visual_frame,
            commands::visualizer_plugins,
            commands::effects_info,
            commands::cover_settings,
            commands::set_cover_download,
            commands::refetch_covers,
            commands::radio_search,
            commands::stations,
            commands::add_station,
            commands::remove_station,
            commands::stream_links,
            commands::qr_svg,
            commands::restart_app,
            commands::firewall_status,
            commands::open_firewall,
            commands::sources_list,
            commands::source_search,
            commands::source_playlist,
            commands::source_import,
            commands::open_plugin_dir,
            commands::import_plugin,
            commands::install_plugin_package,
            commands::plugin_packages,
            commands::remove_plugin_package,
            commands::pc_info,
            commands::pc_pair,
            commands::pc_unpair,
            commands::pc_get,
            commands::pc_post,
            commands::pc_cover,
            commands::pc_play_here,
            commands::pair_link,
            commands::station_logo,
        ])
        .setup(move |app| {
            let handle = app.handle().clone();

            // Android: Daten im App-Ordner (am PC bleibt es ~/.local/share/amber-player).
            log::info!("Start: Einrichtung beginnt");
            #[cfg(mobile)]
            {
                let dir = app.path().app_data_dir()?.join("amber-player");
                log::info!("Start: Datenordner {}", dir.display());
                amber_core::library::set_data_dir(dir);
            }

            let player = open_player().map_err(|e| {
                log::error!("Start: Bibliothek konnte nicht geöffnet werden: {e:#}");
                format!("Bibliothek konnte nicht geöffnet werden: {e:#}")
            })?;
            log::info!("Start: Bibliothek und Player bereit");
            let mut cfg = ApiConfig::load(player.library());
            cfg.ensure_token();
            cfg.save(player.library());
            app.manage(Ctx {
                player: player.clone(),
                api: Mutex::new(ApiStatus { config: cfg, running: false, error: None }),
            });

            // HTTP-API und MCP (nur am PC)
            #[cfg(desktop)]
            {
                let (addr, token) = {
                    let ctx = handle.state::<Ctx>();
                    let api = ctx.api.lock().unwrap();
                    (api.config.bind_addr(), api.config.required_token())
                };
                let api_handle = handle.clone();
                let api_player = player.clone();
                tauri::async_runtime::spawn(async move {
                    api_handle.state::<Ctx>().api.lock().unwrap().running = true;
                    let result = amber_core::api::serve(api_player, addr, token).await;
                    let ctx = api_handle.state::<Ctx>();
                    let mut api = ctx.api.lock().unwrap();
                    api.running = false;
                    if let Err(e) = result {
                        log::error!("API: {e:#}");
                        api.error = Some(format!("{e:#}"));
                    }
                });
            }

            // Statusänderungen an die Oberfläche weiterreichen
            let mut rx = player.subscribe();
            let ev_handle = handle.clone();
            tauri::async_runtime::spawn(async move {
                use tokio::sync::broadcast::error::RecvError;
                loop {
                    match rx.recv().await {
                        Ok(s) => {
                            let _ = ev_handle.emit("status", &s);
                        }
                        Err(RecvError::Lagged(_)) => continue,
                        Err(RecvError::Closed) => break,
                    }
                }
            });

            // Medientasten und Systemsteuerung (MPRIS / Windows / macOS)
            #[cfg(desktop)]
            if let Some(window) = app.get_webview_window("main") {
                media::start(player.clone(), &window);
            }

            // Bibliothek beim Start auf Änderungen prüfen
            if !player.library().folders().unwrap_or_default().is_empty() {
                commands::start_scan(handle.clone());
            }
            #[cfg(desktop)]
            background::setup(app.handle())?;
            log::info!("Start: fertig");
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("Amber Player konnte nicht gestartet werden");
}
