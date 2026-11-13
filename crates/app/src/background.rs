//! Weiterspielen im Hintergrund: Wird das Fenster bei laufender Musik geschlossen, verschwindet
//! es nur, und ein Symbol in der Systemleiste erscheint. Spielt dann keine Musik mehr, beendet
//! sich Amber Player nach kurzer Zeit komplett. Ohne laufende Musik beendet Schließen sofort.

use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use amber_core::{Command, PlayState};
use tauri::menu::{MenuBuilder, MenuItemBuilder};
use tauri::tray::TrayIconBuilder;
use tauri::{AppHandle, Manager, Runtime, Window, WindowEvent};

use crate::Ctx;

const TRAY_ID: &str = "amber";
/// So lange darf im Hintergrund nichts spielen, bevor Amber Player sich beendet
/// (Zeit, um nach einer Pause per Medientaste oder TouchDeck weiterzumachen).
const IDLE_EXIT: Duration = Duration::from_secs(60);
const CHECK_EVERY: Duration = Duration::from_secs(5);

static HIDDEN: AtomicBool = AtomicBool::new(false);

/// Legt das (zunächst unsichtbare) Symbol in der Systemleiste an und startet die Überwachung.
pub fn setup<R: Runtime>(app: &AppHandle<R>) -> tauri::Result<()> {
    let open = MenuItemBuilder::with_id("open", "Amber Player öffnen").build(app)?;
    let toggle = MenuItemBuilder::with_id("toggle", "Abspielen / Pause").build(app)?;
    let next = MenuItemBuilder::with_id("next", "Nächster Titel").build(app)?;
    let quit = MenuItemBuilder::with_id("quit", "Beenden").build(app)?;
    let menu = MenuBuilder::new(app)
        .items(&[&open, &toggle, &next])
        .separator()
        .item(&quit)
        .build()?;

    let mut tray = TrayIconBuilder::with_id(TRAY_ID)
        .tooltip("Amber Player")
        .menu(&menu)
        .show_menu_on_left_click(true)
        .on_menu_event(|app, event| match event.id().as_ref() {
            "open" => show_main(app),
            "toggle" => send(app, Command::Toggle),
            "next" => send(app, Command::Next),
            "quit" => app.exit(0),
            _ => {}
        });
    if let Some(icon) = app.default_window_icon() {
        tray = tray.icon(icon.clone());
    }
    let tray = tray.build(app)?;
    tray.set_visible(false)?;

    let handle = app.clone();
    std::thread::Builder::new()
        .name("amber-player-hintergrund".into())
        .spawn(move || watch(handle))
        .ok();
    Ok(())
}

fn send<R: Runtime>(app: &AppHandle<R>, cmd: Command) {
    let player = app.state::<Ctx>().player.clone();
    std::thread::spawn(move || {
        player.send(cmd);
    });
}

/// Fenster schließen: bei laufender Musik nur verstecken, sonst normal beenden.
pub fn on_window_event<R: Runtime>(window: &Window<R>, event: &WindowEvent) {
    let WindowEvent::CloseRequested { api, .. } = event else {
        return;
    };
    let playing = window.state::<Ctx>().player.status().state == PlayState::Playing;
    if !playing {
        return;
    }
    api.prevent_close();
    let _ = window.hide();
    HIDDEN.store(true, Ordering::Release);
    if let Some(tray) = window.app_handle().tray_by_id(TRAY_ID) {
        let _ = tray.set_visible(true);
    }
    log::info!("Fenster geschlossen, Musik spielt im Hintergrund weiter");
}

/// Fenster wieder zeigen (Leisten-Menü oder erneuter Start aus dem Startmenü).
pub fn show_main<R: Runtime>(app: &AppHandle<R>) {
    if let Some(w) = app.get_webview_window("main") {
        let _ = w.show();
        let _ = w.unminimize();
        let _ = w.set_focus();
    }
    HIDDEN.store(false, Ordering::Release);
    if let Some(tray) = app.tray_by_id(TRAY_ID) {
        let _ = tray.set_visible(false);
    }
}

/// Beendet Amber Player, wenn das Fenster zu ist und eine Weile keine Musik mehr läuft.
fn watch<R: Runtime>(app: AppHandle<R>) {
    let player = app.state::<Ctx>().player.clone();
    let mut idle = Duration::ZERO;
    loop {
        std::thread::sleep(CHECK_EVERY);
        if !HIDDEN.load(Ordering::Acquire) || player.status().state == PlayState::Playing {
            idle = Duration::ZERO;
            continue;
        }
        idle += CHECK_EVERY;
        if idle >= IDLE_EXIT {
            log::info!("Im Hintergrund läuft keine Musik mehr – Amber Player wird beendet");
            app.exit(0);
            return;
        }
    }
}
