//! Audio-Engine: eigener Thread mit rodio, gesteuert über Befehle.

use std::sync::mpsc::{self, RecvTimeoutError};
use std::sync::{Arc, Mutex, RwLock};
use std::thread;
use std::time::{Duration, Instant};

use anyhow::{anyhow, Context, Result};
#[cfg(not(target_os = "android"))]
use rodio::OutputStream;
use rodio::{OutputStreamHandle, Sink, Source};

use crate::decoder::open_audio;
use crate::netstream::{NetSource, StreamTitle};
use crate::radio::{self, Station};
use crate::sources::{self, Sources};
use serde::{Deserialize, Serialize};
use tokio::sync::broadcast;

use crate::analyzer::{Analyzer, VisualFrame};
use crate::effects::{EffectChain, PluginHost};
use crate::fade::{Controlled, TrackCtl};
use crate::library::{Library, Sort, TrackQuery};
use crate::model::{PlayState, RadioNow, Repeat, Status, Track};
use crate::queue::Queue;

const TICK: Duration = Duration::from_millis(200);
/// Takt, solange nichts spielt (spart Akku: die Engine wacht seltener auf)
const IDLE_TICK: Duration = Duration::from_secs(1);
/// Handy: so lange nach dem Anhalten bleibt die Tonausgabe offen, danach wird sie pausiert –
/// sonst schickt sie ununterbrochen Stille an den Audio-Chip und das Handy schläft nie.
#[cfg(target_os = "android")]
const OUTPUT_IDLE: Duration = Duration::from_secs(8);
/// Bewegt sich die Position so lange nicht weiter, gilt der Titel als hängen geblieben.
const STALL_LOCAL: Duration = Duration::from_secs(8);
const STALL_NET: Duration = Duration::from_secs(25);

/// Android beendet die Audioausgabe, wenn das Gerät wechselt (Kopfhörer ein/aus, Bluetooth).
/// Das meldet der Fehler-Callback hier; die Engine öffnet die Ausgabe dann neu.
#[cfg(target_os = "android")]
static OUTPUT_LOST: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
const SAVE_EVERY: Duration = Duration::from_secs(5);
/// Unterhalb dieser Position springt „Zurück“ zum vorigen Titel, sonst an den Anfang.
const PREV_RESTART: Duration = Duration::from_secs(3);
const SEEK_STEP_MS: i64 = 10_000;
const VOLUME_STEP: i16 = 5;
/// So lange vor dem Ende wird der nächste Titel für die lückenlose Wiedergabe angehängt.
const PRELOAD_BEFORE: Duration = Duration::from_secs(8);
/// Höchste einstellbare Überblendung in Sekunden.
pub const MAX_CROSSFADE: u8 = 12;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "cmd", rename_all = "snake_case")]
pub enum Command {
    /// Spielt eine Liste ab Index `start`.
    PlayList { ids: Vec<i64>, start: usize },
    Play,
    Pause,
    Toggle,
    Stop,
    Next,
    Prev,
    Seek { ms: u64 },
    SeekBy { ms: i64 },
    SeekForward,
    SeekBack,
    SetVolume { volume: u8 },
    VolumeUp,
    VolumeDown,
    ToggleMute,
    /// `None` schaltet um.
    SetShuffle { on: Option<bool> },
    /// `None` schaltet weiter (aus → alle → einen).
    SetRepeat { mode: Option<Repeat> },
    /// Hängt an die Warteschlange an.
    Enqueue { id: i64 },
    /// Spielt als Nächstes.
    PlayNext { id: i64 },
    ClearUpNext,
    /// Minuten bis zur Pause; `None` schaltet den Timer aus.
    SleepTimer { minutes: Option<u64> },
    /// Metadaten des aktuellen Titels neu laden (z. B. nach Favoriten-Änderung).
    Refresh,
    /// Internetradio abspielen.
    PlayStation { station: Station },
    /// Überblendung in Sekunden (0 = aus, dann lückenlos).
    SetCrossfade { secs: u8 },
    /// Einen Titel sofort spielen und danach mit der bisherigen Warteschlange weitermachen
    /// (z. B. aus der Suche). Ist die Warteschlange leer, geht es mit der Bibliothek weiter.
    PlayNow { id: i64 },
}

/// Schon an die laufende Spur angehängter nächster Titel (lückenlose Wiedergabe).
struct Preload {
    track: Track,
    ctl: Arc<TrackCtl>,
    /// Warteschlange, wie sie nach dem Wechsel aussieht.
    queue: Queue,
}

/// Folgen zwei Titel auf einem Album direkt aufeinander? Dann wird nicht übergeblendet,
/// sondern lückenlos weitergespielt (Live- und Konzeptalben).
fn album_continues(a: &Track, b: &Track) -> bool {
    if a.album.is_empty() || a.album != b.album || a.album_artist != b.album_artist {
        return false;
    }
    match (a.track_no, b.track_no) {
        (Some(x), Some(y)) if a.disc_no == b.disc_no => y == x + 1,
        (Some(_), Some(1)) => matches!((a.disc_no, b.disc_no), (Some(d1), Some(d2)) if d2 == d1 + 1),
        _ => false,
    }
}

#[derive(Serialize, Deserialize, Default)]
struct Session {
    items: Vec<i64>,
    index: usize,
    position_ms: u64,
}

type Envelope = (Command, Option<mpsc::Sender<Status>>);

pub struct Player {
    tx: mpsc::Sender<Envelope>,
    status: Arc<RwLock<Status>>,
    queue: Arc<Mutex<Queue>>,
    events: broadcast::Sender<Status>,
    library: Arc<Library>,
    effects: Arc<PluginHost>,
    analyzer: Arc<Analyzer>,
    sources: Arc<Sources>,
}

impl Player {
    pub fn new(library: Arc<Library>) -> Self {
        let (tx, rx) = mpsc::channel::<Envelope>();
        let status = Arc::new(RwLock::new(Status::default()));
        let queue = Arc::new(Mutex::new(Queue::new()));
        let (events, _) = broadcast::channel(64);
        let effects = PluginHost::load(Some(&Library::plugin_dir("effects")), Some(library.clone()));
        let analyzer = Arc::new(Analyzer::default());
        let mut sources = Sources::load(&Library::plugin_dir("sources"));
        // Online-Quellen aus Plugin-Paketen nur, wenn „Erweiterte Plugins erlauben“ an ist.
        if library.setting_as::<bool>(crate::packages::EXTENDED_SETTING).unwrap_or(false) {
            let (parts, errors) = crate::packages::source_parts();
            sources.errors.extend(errors);
            sources.add_js(parts);
        }
        let sources = Arc::new(sources);
        match library.prune_online() {
            Ok(n) if n > 0 => log::info!("{n} unbenutzte Online-Titel aufgeräumt"),
            Err(e) => log::warn!("Aufräumen der Online-Titel: {e:#}"),
            _ => {}
        }

        let engine = Engine::restore(
            library.clone(),
            queue.clone(),
            status.clone(),
            events.clone(),
            effects.clone(),
            analyzer.clone(),
            sources.clone(),
        );
        thread::Builder::new()
            .name("amber-player-audio".into())
            .spawn(move || engine.run(rx))
            .expect("Audio-Thread starten");

        Self {
            tx,
            status,
            queue,
            events,
            library,
            effects,
            analyzer,
            sources,
        }
    }

    /// Installierte Quellen-Plugins.
    pub fn sources(&self) -> &Arc<Sources> {
        &self.sources
    }

    /// Frequenzbänder und Wellenform des gerade Gespielten (für die Visualisierung).
    pub fn visual_frame(&self, bands: usize, wave_len: usize) -> VisualFrame {
        self.analyzer.frame(bands, wave_len)
    }

    pub fn library(&self) -> &Arc<Library> {
        &self.library
    }

    /// Effekt-Plugins (Equalizer usw.); Änderungen wirken sofort.
    pub fn effects(&self) -> &Arc<PluginHost> {
        &self.effects
    }

    /// Führt einen Befehl aus und liefert den Status danach.
    pub fn send(&self, cmd: Command) -> Status {
        let (reply_tx, reply_rx) = mpsc::channel();
        if self.tx.send((cmd, Some(reply_tx))).is_err() {
            return self.status();
        }
        reply_rx
            .recv_timeout(Duration::from_secs(5))
            .unwrap_or_else(|_| self.status())
    }

    pub fn status(&self) -> Status {
        self.status.read().unwrap_or_else(|e| e.into_inner()).clone()
    }

    /// Statusänderungen (bei jedem Befehl und etwa sekündlich während der Wiedergabe).
    pub fn subscribe(&self) -> broadcast::Receiver<Status> {
        self.events.subscribe()
    }

    /// Die nächsten `n` Titel in Abspielreihenfolge.
    pub fn upcoming(&self, n: usize) -> Vec<Track> {
        let ids = self.queue.lock().unwrap_or_else(|e| e.into_inner()).upcoming(n);
        self.library.tracks_by_ids(&ids).unwrap_or_default()
    }
}

struct Engine {
    lib: Arc<Library>,
    queue: Arc<Mutex<Queue>>,
    shared: Arc<RwLock<Status>>,
    events: broadcast::Sender<Status>,
    effects: Arc<PluginHost>,
    analyzer: Arc<Analyzer>,
    sources: Arc<Sources>,

    handle: Option<Output>,
    sink: Option<Sink>,
    current: Option<Track>,
    /// Läuft Internetradio, steht hier der Sender (dann ist `current` leer).
    radio: Option<Station>,
    radio_title: StreamTitle,
    last_radio_title: Option<String>,
    state: PlayState,
    volume: u8,
    muted: bool,
    /// Position, an der beim nächsten Start begonnen wird (wiederhergestellte Sitzung).
    pending_seek: Option<Duration>,
    sleep_deadline: Option<Instant>,
    error: Option<String>,
    last_published_sec: u64,
    last_save: Instant,
    /// Überblendung in Sekunden (0 = aus, lückenlos).
    crossfade: u8,
    /// Steuerung des laufenden Titels (zum Ausblenden).
    cur_ctl: Option<Arc<TrackCtl>>,
    preload: Option<Preload>,
    /// Ausblendende Spuren einer Überblendung, mit Zeitpunkt zum Aufräumen.
    fading: Vec<(Sink, Instant)>,
    /// Der Übergang zum nächsten Titel ist für den laufenden schon vorbereitet (oder gescheitert).
    next_handled: bool,
    /// Letzte Position und seit wann sie sich nicht bewegt (Wächter gegen Hänger).
    stall: Option<(Duration, Instant)>,
}

impl Engine {
    fn restore(
        lib: Arc<Library>,
        queue: Arc<Mutex<Queue>>,
        shared: Arc<RwLock<Status>>,
        events: broadcast::Sender<Status>,
        effects: Arc<PluginHost>,
        analyzer: Arc<Analyzer>,
        sources: Arc<Sources>,
    ) -> Self {
        let session: Session = lib.setting_as("session").unwrap_or_default();
        let current = {
            let mut q = queue.lock().unwrap();
            q.set_repeat(lib.setting_as("repeat").unwrap_or_default());
            let items = lib
                .tracks_by_ids(&session.items)
                .unwrap_or_default()
                .into_iter()
                .map(|t| t.id)
                .collect();
            q.set(items, session.index);
            q.set_shuffle(lib.setting_as("shuffle").unwrap_or(false));
            q.current().and_then(|id| lib.track(id).ok().flatten())
        };

        let lib_crossfade = lib.setting_as::<u8>("crossfade").unwrap_or(0).min(MAX_CROSSFADE);
        Self {
            volume: lib.setting_as("volume").unwrap_or(70),
            muted: false,
            pending_seek: current
                .as_ref()
                .map(|_| Duration::from_millis(session.position_ms)),
            current,
            lib,
            queue,
            shared,
            events,
            effects,
            analyzer,
            sources,
            handle: None,
            sink: None,
            radio: None,
            radio_title: Default::default(),
            last_radio_title: None,
            state: PlayState::Stopped,
            sleep_deadline: None,
            error: None,
            last_published_sec: u64::MAX,
            last_save: Instant::now(),
            crossfade: lib_crossfade,
            cur_ctl: None,
            preload: None,
            fading: Vec::new(),
            next_handled: false,
            stall: None,
        }
    }

    fn run(mut self, rx: mpsc::Receiver<Envelope>) {
        // Der Ausgabe-Stream ist nicht `Send` und muss daher in diesem Thread leben.
        #[cfg(not(target_os = "android"))]
        let opened = OutputStream::try_default().map(|(stream, handle)| (stream, Output::Rodio(handle))).map_err(|e| e.to_string());
        #[cfg(target_os = "android")]
        let opened = android_output().map(|(stream, mixer)| (stream, Output::Mixer(mixer))).map_err(|e| format!("{e:#}"));
        #[allow(unused_mut)]
        let mut _stream = match opened {
            Ok((stream, handle)) => {
                self.handle = Some(handle);
                Some(stream)
            }
            Err(e) => {
                log::error!("Kein Audiogerät: {e}");
                self.error = Some(format!("Kein Audiogerät gefunden: {e}"));
                None
            }
        };
        self.publish();

        #[cfg(target_os = "android")]
        let mut idle_since: Option<Instant> = None;
        #[cfg(target_os = "android")]
        let mut output_paused = false;
        loop {
            let busy = self.state == PlayState::Playing || !self.fading.is_empty();
            match rx.recv_timeout(if busy { TICK } else { IDLE_TICK }) {
                Ok((cmd, reply)) => {
                    // Erst die Ausgabe wieder anwerfen: manche Befehle (z. B. Springen) warten
                    // darauf, dass der Audio-Thread sie ausführt – bei pausierter Ausgabe hinge
                    // die Engine sonst.
                    #[cfg(target_os = "android")]
                    if output_paused {
                        use rodio::cpal::traits::StreamTrait;
                        if let Some(st) = &_stream {
                            if let Err(e) = st.play() {
                                log::warn!("Audioausgabe fortsetzen: {e}");
                            }
                        }
                        output_paused = false;
                        idle_since = None;
                        log::info!("Audioausgabe wieder aktiv");
                    }
                    self.handle_command(cmd);
                    self.publish();
                    if let Some(r) = reply {
                        let _ = r.send(self.snapshot());
                    }
                }
                Err(RecvTimeoutError::Timeout) => {}
                Err(RecvTimeoutError::Disconnected) => break,
            }
            #[cfg(target_os = "android")]
            if OUTPUT_LOST.swap(false, std::sync::atomic::Ordering::Relaxed) {
                match android_output() {
                    Ok((stream, mixer)) => {
                        _stream = Some(stream);
                        output_paused = false;
                        self.handle = Some(Output::Mixer(mixer));
                        self.reattach();
                    }
                    Err(e) => log::error!("Audioausgabe neu öffnen: {e:#}"),
                }
            }
            self.tick();
            // Akku: Tonausgabe pausieren, wenn eine Weile nichts spielt; sofort weiter beim Abspielen
            #[cfg(target_os = "android")]
            {
                use rodio::cpal::traits::StreamTrait;
                let busy = self.state == PlayState::Playing || !self.fading.is_empty();
                if busy {
                    idle_since = None;
                    if output_paused {
                        if let Some(st) = &_stream {
                            if let Err(e) = st.play() {
                                log::warn!("Audioausgabe fortsetzen: {e}");
                            }
                        }
                        output_paused = false;
                        log::info!("Audioausgabe wieder aktiv");
                    }
                } else {
                    let since = *idle_since.get_or_insert_with(Instant::now);
                    if !output_paused && since.elapsed() >= OUTPUT_IDLE {
                        if let Some(st) = &_stream {
                            if st.pause().is_ok() {
                                output_paused = true;
                                log::info!("Audioausgabe pausiert (nichts spielt) – spart Akku");
                            }
                        }
                    }
                }
            }
        }
        self.save_session();
    }

    fn q(&self) -> std::sync::MutexGuard<'_, Queue> {
        self.queue.lock().unwrap_or_else(|e| e.into_inner())
    }

    fn handle_command(&mut self, cmd: Command) {
        // Alles, was die Reihenfolge ändern kann: angehängten nächsten Titel verwerfen, er wird
        // mit der neuen Reihenfolge wieder vorbereitet.
        if !matches!(
            cmd,
            Command::Seek { .. }
                | Command::SeekBy { .. }
                | Command::SeekForward
                | Command::SeekBack
                | Command::SetVolume { .. }
                | Command::VolumeUp
                | Command::VolumeDown
                | Command::ToggleMute
                | Command::SleepTimer { .. }
                | Command::Refresh
                | Command::Play
                | Command::Pause
                | Command::Toggle
                | Command::SetCrossfade { .. }
        ) {
            self.drop_preload();
        }
        match cmd {
            Command::PlayList { ids, start } => {
                let first = self.q().set(ids, start);
                match first {
                    Some(id) => self.play_id(id),
                    None => self.stop(),
                }
            }
            Command::Play => self.play(),
            Command::Pause => self.pause(),
            Command::Toggle => match self.state {
                PlayState::Playing => self.pause(),
                _ => self.play(),
            },
            Command::Stop => self.stop(),
            Command::Next => self.advance(false),
            Command::Prev => self.prev(),
            Command::Seek { ms } => self.seek(Duration::from_millis(ms)),
            Command::SeekBy { ms } => self.seek_by(ms),
            Command::SeekForward => self.seek_by(SEEK_STEP_MS),
            Command::SeekBack => self.seek_by(-SEEK_STEP_MS),
            Command::SetVolume { volume } => self.set_volume(volume as i16),
            Command::VolumeUp => self.set_volume(self.volume as i16 + VOLUME_STEP),
            Command::VolumeDown => self.set_volume(self.volume as i16 - VOLUME_STEP),
            Command::ToggleMute => {
                self.muted = !self.muted;
                self.apply_volume();
            }
            Command::SetShuffle { on } => {
                let mut q = self.q();
                let on = on.unwrap_or(!q.shuffle());
                q.set_shuffle(on);
                drop(q);
                let _ = self.lib.set_setting_json("shuffle", &on);
            }
            Command::SetRepeat { mode } => {
                let mut q = self.q();
                let mode = mode.unwrap_or(q.repeat().cycle());
                q.set_repeat(mode);
                drop(q);
                let _ = self.lib.set_setting_json("repeat", &mode);
            }
            Command::Enqueue { id } => self.q().enqueue(id),
            Command::PlayNext { id } => self.q().play_next(id),
            Command::ClearUpNext => self.q().clear_upcoming(),
            Command::SleepTimer { minutes } => {
                self.sleep_deadline = minutes
                    .filter(|&m| m > 0)
                    .map(|m| Instant::now() + Duration::from_secs(m * 60));
            }
            Command::PlayStation { station } => self.play_station(station),
            Command::PlayNow { id } => {
                let empty = self.q().is_empty();
                if empty {
                    let ids: Vec<i64> = self
                        .lib
                        .tracks(&TrackQuery { sort: Sort::Artist, ..Default::default() })
                        .unwrap_or_default()
                        .iter()
                        .map(|t| t.id)
                        .collect();
                    let start = ids.iter().position(|&x| x == id).unwrap_or(0);
                    if ids.is_empty() {
                        self.play_id(id);
                    } else {
                        let first = self.q().set(ids, start);
                        if let Some(first) = first {
                            self.play_id(first);
                        }
                    }
                } else {
                    self.q().play_next(id);
                    self.advance(false);
                }
            }
            Command::SetCrossfade { secs } => {
                self.crossfade = secs.min(MAX_CROSSFADE);
                let _ = self.lib.set_setting_json("crossfade", &self.crossfade);
                // Neu entscheiden, wie der nächste Wechsel läuft
                self.drop_preload();
            }
            Command::Refresh => {
                if let Some(t) = &self.current {
                    self.current = self.lib.track(t.id).ok().flatten();
                }
            }
        }
    }

    fn play(&mut self) {
        if let Some(sink) = &self.sink {
            sink.play();
            for (s, _) in &self.fading {
                s.play();
            }
            self.state = PlayState::Playing;
            return;
        }
        if let Some(station) = self.radio.clone() {
            self.play_station(station);
            return;
        }
        let current = self.q().current();
        match current {
            Some(id) => {
                let seek = self.pending_seek.take();
                if let Err(e) = self.start(id, seek) {
                    self.error = Some(e.to_string());
                    self.advance(true);
                }
            }
            None => {
                // Nichts geladen: ganze Bibliothek abspielen.
                let ids: Vec<i64> = self
                    .lib
                    .tracks(&TrackQuery {
                        sort: Sort::Artist,
                        ..Default::default()
                    })
                    .unwrap_or_default()
                    .iter()
                    .map(|t| t.id)
                    .collect();
                let first = self.q().set(ids, 0);
                if let Some(id) = first {
                    self.play_id(id);
                }
            }
        }
    }

    fn pause(&mut self) {
        if let Some(sink) = &self.sink {
            log::info!("Pause");
            sink.pause();
            for (s, _) in &self.fading {
                s.pause();
            }
            self.state = PlayState::Paused;
        }
    }

    /// Nach neu geöffneter Ausgabe: laufenden Titel bzw. Sender an derselben Stelle fortsetzen
    /// (pausiert bleibt pausiert).
    #[allow(dead_code)]
    fn reattach(&mut self) {
        log::info!("Audioausgabe neu geöffnet, setze Wiedergabe fort");
        if self.sink.is_none() {
            return;
        }
        let was = self.state;
        if let Some(station) = self.radio.clone() {
            if was == PlayState::Playing {
                self.play_station(station);
            } else {
                self.sink = None;
                self.state = PlayState::Stopped;
            }
        } else if let Some(id) = self.current.as_ref().map(|t| t.id) {
            let pos = self.position();
            if let Err(e) = self.start(id, Some(pos)) {
                self.error = Some(e.to_string());
                return;
            }
            if was == PlayState::Paused {
                self.pause();
            }
        }
        self.publish();
    }

    /// Hängt ein Titel (Position bewegt sich nicht, z. B. kaputte Datei oder abgebrochener
    /// Stream), wird zum nächsten gesprungen statt endlos zu warten.
    fn watch_stall(&mut self) {
        if self.state != PlayState::Playing || self.sink.is_none() {
            self.stall = None;
            return;
        }
        let pos = self.position();
        match self.stall {
            Some((p, since)) if p == pos => {
                let online = self.radio.is_some() || self.current.as_ref().is_some_and(|t| sources::parse_path(&t.path).is_some());
                let limit = if online { STALL_NET } else { STALL_LOCAL };
                if since.elapsed() >= limit {
                    let name = self.current.as_ref().map(|t| t.title.clone()).or_else(|| self.radio.as_ref().map(|s| s.name.clone())).unwrap_or_default();
                    log::warn!("„{name}“ hängt seit {} s – weiter zum nächsten", limit.as_secs());
                    self.stall = None;
                    self.advance(true);
                    self.error = Some(format!("„{name}“ ließ sich nicht abspielen"));
                    self.publish();
                }
            }
            _ => self.stall = Some((pos, Instant::now())),
        }
    }

    /// Vorgeladenen Titel verwerfen (er endet sofort, wenn er an der Reihe wäre).
    fn drop_preload(&mut self) {
        if let Some(p) = self.preload.take() {
            p.ctl.cancel();
        }
        self.next_handled = false;
    }

    /// Alles neben der laufenden Spur beenden: Vorgeladenes und Überblendungen.
    fn drop_extra(&mut self) {
        self.drop_preload();
        self.fading.clear();
        self.cur_ctl = None;
    }

    fn stop(&mut self) {
        log::info!("Wiedergabe gestoppt");
        self.drop_extra();
        self.sink = None;
        self.pending_seek = None;
        self.state = PlayState::Stopped;
    }

    /// Startet `id`; bei defekten Dateien wird bis zu 20-mal der nächste Titel versucht.
    fn play_id(&mut self, mut id: i64) {
        for _ in 0..20 {
            match self.start(id, None) {
                Ok(()) => return,
                Err(e) => {
                    log::warn!("Titel {id} nicht abspielbar: {e:#}");
                    self.error = Some(format!("{e:#}"));
                    if self.handle.is_none() {
                        break;
                    }
                    match self.q().next(false) {
                        Some(n) => id = n,
                        None => break,
                    }
                }
            }
        }
        self.stop();
    }

    /// Internetradio starten (verbindet sich und puffert vor).
    fn play_station(&mut self, station: Station) {
        self.drop_extra();
        self.sink = None;
        self.current = None;
        *self.radio_title.lock().unwrap_or_else(|e| e.into_inner()) = None;
        self.last_radio_title = None;
        self.radio = Some(station.clone());
        let result = (|| -> Result<()> {
            let handle = self.handle.as_ref().ok_or_else(|| anyhow!("Kein Audiogerät verfügbar"))?;
            let source = NetSource::open(&station.url, self.radio_title.clone())?;
            let chain = EffectChain::new(source, self.effects.clone()).with_analyzer(self.analyzer.clone());
            let sink = handle.new_sink()?;
            sink.append(chain);
            self.sink = Some(sink);
            Ok(())
        })();
        match result {
            Ok(()) => {
                self.apply_volume();
                self.state = PlayState::Playing;
                self.error = None;
                radio::count_click(&station.uuid);
            }
            Err(e) => {
                self.error = Some(format!("{}: {e:#}", station.name));
                self.state = PlayState::Stopped;
            }
        }
    }

    /// Bei Radio: zum nächsten bzw. vorigen gespeicherten Sender wechseln.
    fn step_station(&mut self, delta: i64) {
        let stations = self.lib.stations().unwrap_or_default();
        if stations.is_empty() {
            return;
        }
        let url = self.radio.as_ref().map(|s| s.url.clone()).unwrap_or_default();
        let idx = stations.iter().position(|s| s.url == url).map_or(0, |i| {
            (i as i64 + delta).rem_euclid(stations.len() as i64) as usize
        });
        self.play_station(stations[idx].clone());
    }

    /// Öffnet die Audioquelle eines Titels (Datei, PC-Stream oder Online-Quelle). Fehlt die
    /// Länge in der Bibliothek, wird sie aus der Quelle ergänzt.
    fn open_track(&self, track: &mut Track) -> Result<Box<dyn Source<Item = f32> + Send>> {
        let decoder: Box<dyn Source<Item = f32> + Send> = match sources::parse_path(&track.path) {
            // Online-Titel: frische Adresse beim Plugin holen (die laufen nach einigen Stunden ab).
            // Titel vom gekoppelten PC: direkt über dessen Stream-Adresse (spulbar).
            Some((crate::pc::SOURCE_ID, file)) => {
                let link = crate::pc::PcLink::load(&self.lib)
                    .ok_or_else(|| anyhow!("Kein PC gekoppelt – „{}“ liegt auf dem PC", track.title))?;
                if !link.reachable() {
                    return Err(anyhow!("PC nicht erreichbar – „{}“ liegt auf dem PC", track.title));
                }
                Box::new(
                    crate::httpsource::open_url(&link.stream_url(file), Default::default())
                        .with_context(|| format!("„{}“ lässt sich nicht vom PC laden", track.title))?,
                )
            }
            Some((plugin, id)) => {
                let resolved = self.sources.get(plugin)?.resolve(id)?;
                Box::new(
                    crate::httpsource::open_url(&resolved.url, resolved.headers)
                        .with_context(|| format!("„{}“ lässt sich nicht abspielen", track.title))?,
                )
            }
            None => {
                let src = open_audio(std::path::Path::new(&track.path))
                    .with_context(|| format!("„{}“ lässt sich nicht abspielen", track.title))?;
                // Handy: nicht im Audio-Takt dekodieren, sondern ein paar Sekunden im eigenen
                // Thread vorpuffern. Sonst reißt der Ton ab, wenn das Handy kurz langsamer ist
                // (hörbar als Knacken/Rauschen).
                #[cfg(target_os = "android")]
                let src: Box<dyn Source<Item = f32> + Send> = Box::new(crate::httpsource::Prefetch::new(src)?);
                src
            }
        };
        if track.duration_ms == 0 {
            if let Some(d) = decoder.total_duration() {
                track.duration_ms = d.as_millis() as u64;
            }
        }
        Ok(decoder)
    }

    /// Quelle mit Effekten, Visualisierungs-Abgriff und Steuerung (Ein-/Ausblenden).
    fn chain(&self, decoder: Box<dyn Source<Item = f32> + Send>, ctl: &Arc<TrackCtl>) -> impl Source<Item = f32> + Send + 'static {
        Controlled::new(EffectChain::new(decoder, self.effects.clone()).with_analyzer(self.analyzer.clone()), ctl.clone())
    }

    fn start(&mut self, id: i64, seek: Option<Duration>) -> Result<()> {
        self.drop_extra();
        self.sink = None;
        self.radio = None;
        let mut track = self
            .lib
            .track(id)?
            .ok_or_else(|| anyhow!("Titel {id} nicht in der Bibliothek"))?;
        self.current = Some(track.clone());
        if self.handle.is_none() {
            return Err(anyhow!("Kein Audiogerät verfügbar"));
        }
        let where_ = match sources::parse_path(&track.path) {
            Some((crate::pc::SOURCE_ID, _)) => "vom PC",
            Some(_) => "online",
            None => "Datei",
        };
        log::info!("Spiele „{}“ ({where_}, Titel {id}{})", track.title, seek.map(|p| format!(", ab {} s", p.as_secs())).unwrap_or_default());
        let decoder = self.open_track(&mut track)?;
        self.current = Some(track);
        let ctl = TrackCtl::new(1.0);
        let source = self.chain(decoder, &ctl);
        let sink = self.handle.as_ref().expect("geprüft").new_sink()?;
        sink.append(source);
        if let Some(pos) = seek.filter(|p| !p.is_zero()) {
            let _ = sink.try_seek(pos);
        }
        self.sink = Some(sink);
        self.cur_ctl = Some(ctl);
        self.next_handled = false;
        self.apply_volume();
        self.state = PlayState::Playing;
        self.error = None;
        let _ = self.lib.mark_played(id);
        Ok(())
    }

    /// Kurz vor dem Ende: nächsten Titel lückenlos anhängen oder überblenden.
    fn prepare_next(&mut self) {
        if self.next_handled || self.preload.is_some() || self.radio.is_some() || self.state != PlayState::Playing {
            return;
        }
        let Some(cur) = self.current.clone() else { return };
        if cur.duration_ms == 0 {
            return; // Länge unbekannt: wie bisher erst am Ende wechseln
        }
        let remaining = Duration::from_millis(cur.duration_ms).saturating_sub(self.position());
        let fade = Duration::from_secs(self.crossfade as u64);
        if remaining > PRELOAD_BEFORE.max(fade) {
            return;
        }
        // Wie sieht die Reihenfolge nach dem Wechsel aus? (auf einer Kopie)
        let mut queue = self.q().clone();
        let Some(next_id) = queue.next(true) else {
            self.next_handled = true;
            return;
        };
        let Some(next) = self.lib.track(next_id).ok().flatten() else {
            self.next_handled = true;
            return;
        };
        let crossfade = self.crossfade > 0 && !album_continues(&cur, &next);
        if crossfade && remaining > fade {
            return; // noch zu früh für die Überblendung
        }
        if !crossfade && remaining > PRELOAD_BEFORE {
            return;
        }
        self.next_handled = true;
        let mut next = next;
        let decoder = match self.open_track(&mut next) {
            Ok(d) => d,
            Err(e) => {
                // Dann wie bisher am Ende wechseln (mit Fehlerbehandlung in `advance`).
                log::warn!("Nächsten Titel vorbereiten: {e:#}");
                return;
            }
        };
        if crossfade {
            self.crossfade_to(next, decoder, queue);
        } else if let Some(sink) = &self.sink {
            let ctl = TrackCtl::new(1.0);
            sink.append(self.chain(decoder, &ctl));
            self.preload = Some(Preload { track: next, ctl, queue });
        }
    }

    /// Überblenden: neuer Titel in eigener Spur einblenden, der alte blendet aus.
    fn crossfade_to(&mut self, next: Track, decoder: Box<dyn Source<Item = f32> + Send>, queue: Queue) {
        let secs = self.crossfade as f32;
        log::info!("Überblende ({secs} s) zu „{}“", next.title);
        let Some(handle) = self.handle.as_ref() else { return };
        let ctl = TrackCtl::new(0.0);
        let source = self.chain(decoder, &ctl);
        let sink = match handle.new_sink() {
            Ok(s) => s,
            Err(e) => {
                log::warn!("Überblendung: {e:#}");
                return;
            }
        };
        sink.append(source);
        ctl.fade_to(1.0, secs);
        if let Some(old) = self.cur_ctl.take() {
            old.fade_to(0.0, secs);
        }
        if let Some(old) = self.sink.take() {
            self.fading.push((old, Instant::now() + Duration::from_secs_f32(secs + 0.5)));
        }
        self.sink = Some(sink);
        self.cur_ctl = Some(ctl);
        *self.q() = queue;
        let id = next.id;
        self.current = Some(next);
        self.next_handled = false;
        self.apply_volume();
        let _ = self.lib.mark_played(id);
        self.publish();
    }

    /// Lückenlos: Ist der angehängte Titel an der Reihe, gilt er ab jetzt als der laufende.
    fn gapless_switch(&mut self) {
        let switched = self.preload.is_some() && self.sink.as_ref().is_some_and(|s| s.len() == 1);
        if !switched {
            return;
        }
        let p = self.preload.take().expect("geprüft");
        log::info!("Lückenlos weiter mit „{}“", p.track.title);
        *self.q() = p.queue;
        let id = p.track.id;
        self.current = Some(p.track);
        self.cur_ctl = Some(p.ctl);
        self.next_handled = false;
        let _ = self.lib.mark_played(id);
        self.publish();
    }

    fn advance(&mut self, auto: bool) {
        log::info!("Nächster Titel ({})", if auto { "Titel zu Ende" } else { "Weiter gedrückt" });
        if self.radio.is_some() {
            if auto {
                self.stop();
                self.error = Some("Der Sender hat die Verbindung beendet".into());
            } else {
                self.step_station(1);
            }
            return;
        }
        let next = self.q().next(auto);
        match next {
            Some(id) => self.play_id(id),
            None => self.stop(),
        }
    }

    fn prev(&mut self) {
        if self.radio.is_some() {
            self.step_station(-1);
            return;
        }
        if self.position() > PREV_RESTART {
            self.seek(Duration::ZERO);
            return;
        }
        let prev = self.q().prev();
        if let Some(id) = prev {
            self.play_id(id);
        }
    }

    fn position(&self) -> Duration {
        match &self.sink {
            Some(s) => s.get_pos(),
            None => self.pending_seek.unwrap_or_default(),
        }
    }

    fn seek(&mut self, pos: Duration) {
        if self.radio.is_some() {
            return;
        }
        let max = self
            .current
            .as_ref()
            .map(|t| Duration::from_millis(t.duration_ms))
            .filter(|d| !d.is_zero());
        let pos = max.map_or(pos, |m| pos.min(m.saturating_sub(Duration::from_millis(500))));
        match &self.sink {
            Some(sink) => {
                if let Err(e) = sink.try_seek(pos) {
                    self.error = Some(format!("Springen nicht möglich: {e}"));
                }
            }
            None if self.current.is_some() => self.pending_seek = Some(pos),
            None => {}
        }
    }

    fn seek_by(&mut self, ms: i64) {
        let pos = self.position().as_millis() as i64 + ms;
        self.seek(Duration::from_millis(pos.max(0) as u64));
    }

    fn set_volume(&mut self, v: i16) {
        self.volume = v.clamp(0, 100) as u8;
        self.muted = false;
        self.apply_volume();
        let _ = self.lib.set_setting_json("volume", &self.volume);
    }

    fn apply_volume(&self) {
        let v = if self.muted { 0.0 } else { self.volume as f32 / 100.0 };
        // Quadratische Kurve klingt gleichmäßiger als linear.
        for sink in self.sink.iter().chain(self.fading.iter().map(|(s, _)| s)) {
            sink.set_volume(v * v);
        }
    }

    fn tick(&mut self) {
        let now = Instant::now();
        self.fading.retain(|(_, until)| now < *until);
        self.gapless_switch();
        self.prepare_next();
        self.watch_stall();
        if self.state == PlayState::Playing && self.sink.as_ref().is_some_and(|s| s.empty()) {
            self.advance(true);
            self.publish();
        }
        if self.sleep_deadline.is_some_and(|d| Instant::now() >= d) {
            self.sleep_deadline = None;
            self.pause();
            self.publish();
        }
        if self.radio.is_some() {
            let title = self.radio_title.lock().unwrap_or_else(|e| e.into_inner()).clone();
            if title != self.last_radio_title {
                self.last_radio_title = title;
                self.publish();
            }
        }
        let sec = self.position().as_secs();
        if self.state == PlayState::Playing && sec != self.last_published_sec {
            self.publish();
        }
        if self.last_save.elapsed() >= SAVE_EVERY {
            self.save_session();
        }
    }

    fn save_session(&mut self) {
        self.last_save = Instant::now();
        let session = {
            let q = self.q();
            Session {
                items: q.items().to_vec(),
                index: q.current_index().unwrap_or(0),
                position_ms: self.position().as_millis() as u64,
            }
        };
        let _ = self.lib.set_setting_json("session", &session);
    }

    fn snapshot(&self) -> Status {
        let q = self.q();
        Status {
            state: self.state,
            track: self.current.clone(),
            radio: self.radio.as_ref().map(|s| RadioNow {
                station: s.clone(),
                title: self.radio_title.lock().unwrap_or_else(|e| e.into_inner()).clone(),
            }),
            position_ms: self.position().as_millis() as u64,
            duration_ms: self.current.as_ref().map_or(0, |t| t.duration_ms),
            volume: self.volume,
            muted: self.muted,
            shuffle: q.shuffle(),
            repeat: q.repeat(),
            sleep_remaining_s: self
                .sleep_deadline
                .map(|d| d.saturating_duration_since(Instant::now()).as_secs()),
            queue_length: q.len(),
            error: self.error.clone(),
            crossfade: self.crossfade,
        }
    }

    fn publish(&mut self) {
        let s = self.snapshot();
        self.last_published_sec = s.position_ms / 1000;
        *self.shared.write().unwrap_or_else(|e| e.into_inner()) = s.clone();
        let _ = self.events.send(s);
    }
}

/// Wohin die Wiedergabe geht: am PC der Standard-Stream von rodio, am Handy ein eigener
/// Stream mit größerem Puffer (siehe [`android_output`]).
enum Output {
    #[allow(dead_code)]
    Rodio(OutputStreamHandle),
    #[allow(dead_code)]
    Mixer(Arc<rodio::dynamic_mixer::DynamicMixerController<f32>>),
}

impl Output {
    fn new_sink(&self) -> Result<Sink> {
        match self {
            Output::Rodio(h) => Sink::try_new(h).context("Audioausgabe öffnen"),
            Output::Mixer(m) => {
                let (sink, out) = Sink::new_idle();
                m.add(out);
                Ok(sink)
            }
        }
    }
}

/// Handy: Ausgabe selbst öffnen. rodio nimmt dort den kleinsten Puffer des Geräts; reicht
/// die Zeit einmal nicht (Handy kurz beschäftigt), reißt der Ton ab – hörbar als leises
/// Knacken. Mit eigenem Stream lässt sich ein größerer Puffer anfordern (~85 ms).
#[cfg(target_os = "android")]
fn android_output() -> Result<(rodio::cpal::Stream, Arc<rodio::dynamic_mixer::DynamicMixerController<f32>>)> {
    use rodio::cpal::traits::{HostTrait, StreamTrait};
    use rodio::cpal::{BufferSize, SampleFormat, StreamConfig};
    use rodio::DeviceTrait;

    const FRAMES: u32 = 4096;
    let device = rodio::cpal::default_host()
        .default_output_device()
        .ok_or_else(|| anyhow!("Kein Audiogerät gefunden"))?;
    let def = device.default_output_config().context("Audioformat des Geräts lesen")?;
    let channels = def.channels();
    let rate = def.sample_rate();
    log::info!("Audioausgabe: {} Hz, {} Kanäle, {:?}, Puffer {FRAMES} Frames", rate.0, channels, def.sample_format());

    let build = |buffer: BufferSize| -> Result<(rodio::cpal::Stream, Arc<rodio::dynamic_mixer::DynamicMixerController<f32>>)> {
        let (ctl, mut mixer) = rodio::dynamic_mixer::mixer::<f32>(channels, rate.0);
        let config = StreamConfig { channels, sample_rate: rate, buffer_size: buffer };
        let on_err = |e| {
            log::warn!("Audioausgabe unterbrochen ({e}) – wird neu geöffnet");
            OUTPUT_LOST.store(true, std::sync::atomic::Ordering::Relaxed);
        };
        let stream = match def.sample_format() {
            SampleFormat::I16 => device.build_output_stream(
                &config,
                move |data: &mut [i16], _| {
                    for d in data.iter_mut() {
                        let v = mixer.next().unwrap_or(0.0).clamp(-1.0, 1.0);
                        *d = (v * i16::MAX as f32) as i16;
                    }
                },
                on_err,
                None,
            )?,
            _ => device.build_output_stream(
                &config,
                move |data: &mut [f32], _| {
                    for d in data.iter_mut() {
                        *d = mixer.next().unwrap_or(0.0);
                    }
                },
                on_err,
                None,
            )?,
        };
        stream.play()?;
        Ok((stream, ctl))
    };
    build(BufferSize::Fixed(FRAMES)).or_else(|e| {
        log::warn!("Größerer Audiopuffer nicht möglich ({e:#}), nehme den Standard");
        build(BufferSize::Default)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn t(album: &str, disc: Option<u32>, no: Option<u32>) -> Track {
        Track { album: album.into(), album_artist: "A".into(), disc_no: disc, track_no: no, ..Default::default() }
    }

    #[test]
    fn album_continues_only_for_direct_successors() {
        assert!(album_continues(&t("Live", None, Some(3)), &t("Live", None, Some(4))));
        assert!(!album_continues(&t("Live", None, Some(3)), &t("Live", None, Some(7))));
        assert!(!album_continues(&t("Live", None, Some(3)), &t("Anderes", None, Some(4))));
        assert!(album_continues(&t("Box", Some(1), Some(12)), &t("Box", Some(2), Some(1))));
        assert!(!album_continues(&t("", None, Some(1)), &t("", None, Some(2))));
    }
}
