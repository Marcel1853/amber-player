//! HTTP-API zum Fernsteuern (Stream Deck, TouchDeck, Home Assistant, curl) und MCP-Endpunkt.
//!
//! Jede Aktion ist per GET und POST erreichbar, damit auch einfache „URL aufrufen“-Tasten
//! funktionieren. Aktionen antworten mit dem Player-Status als JSON.

use std::net::SocketAddr;
use std::sync::Arc;

use anyhow::Context;
use axum::extract::{ConnectInfo, Path, Query, Request, State};
use axum::http::{header, HeaderMap, HeaderValue, StatusCode};
use axum::middleware::{self, Next};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, MethodRouter};
use axum::{Json, Router};
use serde::Deserialize;
use serde_json::{json, Value};

use crate::library::{Sort, TrackQuery};
use crate::model::{Repeat, Status};
use crate::player::{Command, Player};

pub const DEFAULT_PORT: u16 = 7878;

#[derive(Clone)]
pub struct ApiState {
    pub player: Arc<Player>,
    /// Ist ein Token gesetzt, muss jede Anfrage es mitschicken (`?token=` oder Bearer-Header).
    pub token: Option<Arc<str>>,
}

pub struct ApiError(StatusCode, String);

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (self.0, Json(json!({ "error": self.1 }))).into_response()
    }
}

impl From<anyhow::Error> for ApiError {
    fn from(e: anyhow::Error) -> Self {
        ApiError(StatusCode::BAD_REQUEST, format!("{e:#}"))
    }
}

fn not_found(what: &str) -> ApiError {
    ApiError(StatusCode::NOT_FOUND, format!("{what} nicht gefunden"))
}

type ApiResult<T> = Result<T, ApiError>;

/// Führt blockierende Player-/Datenbankaufrufe außerhalb der async-Threads aus.
pub(crate) async fn blocking<T: Send + 'static>(f: impl FnOnce() -> T + Send + 'static) -> T {
    tokio::task::spawn_blocking(f)
        .await
        .expect("Hintergrundaufgabe abgestürzt")
}

async fn run(st: &ApiState, cmd: Command) -> Json<Status> {
    let p = st.player.clone();
    Json(blocking(move || p.send(cmd)).await)
}

/// Route für einen Befehl ohne Parameter, per GET und POST.
fn action(cmd: Command) -> MethodRouter<ApiState> {
    let h = move |State(st): State<ApiState>| async move { run(&st, cmd).await };
    get(h.clone()).post(h)
}

pub fn router(player: Arc<Player>, token: Option<String>) -> Router {
    let state = ApiState {
        player,
        token: token.filter(|t| !t.is_empty()).map(Into::into),
    };
    Router::new()
        .route("/", get(index))
        .route("/api", get(index))
        .route("/api/status", get(status))
        .route("/api/play", action(Command::Play))
        .route("/api/pause", action(Command::Pause))
        .route("/api/toggle", action(Command::Toggle))
        .route("/api/stop", action(Command::Stop))
        .route("/api/next", action(Command::Next))
        .route("/api/prev", action(Command::Prev))
        .route("/api/volume", get(volume).post(volume))
        .route("/api/volume/up", action(Command::VolumeUp))
        .route("/api/volume/down", action(Command::VolumeDown))
        .route("/api/mute", action(Command::ToggleMute))
        .route("/api/seek", get(seek).post(seek))
        .route("/api/seek/forward", action(Command::SeekForward))
        .route("/api/seek/back", action(Command::SeekBack))
        .route("/api/shuffle", get(shuffle).post(shuffle))
        .route("/api/repeat", get(repeat).post(repeat))
        .route("/api/sleep", get(sleep).post(sleep))
        .route("/api/favorite", get(favorite).post(favorite))
        .route("/api/search", get(search))
        .route("/api/play/track/:id", get(play_track).post(play_track))
        .route("/api/play/music", get(play_music).post(play_music))
        .route("/api/queue", get(queue))
        .route("/api/queue/add/:id", get(queue_add).post(queue_add))
        .route("/api/queue/next/:id", get(queue_next).post(queue_next))
        .route("/api/queue/clear", action(Command::ClearUpNext))
        .route("/api/playlists", get(playlists))
        .route("/api/playlists/:id", get(playlist))
        .route("/api/playlists/:id/play", get(playlist_play).post(playlist_play))
        .route("/api/effects", get(effects))
        .route("/api/effects/:id", get(effect).post(effect))
        .route("/api/effects/:id/toggle", get(effect_toggle).post(effect_toggle))
        .route(
            "/api/effects/:id/param/:param",
            get(effect_param).post(effect_param),
        )
        .route("/api/cover/current", get(cover_current))
        .route("/api/cover/:id", get(cover))
        .route("/api/library", get(library_stats))
        .route("/api/library/scan", get(library_scan).post(library_scan))
        .route("/api/online", get(online_list))
        .route("/api/online/search", get(online_search))
        .route("/api/online/play", get(online_play).post(online_play))
        .route("/api/online/import", get(online_import).post(online_import))
        .route("/api/radio", get(radio_list))
        .route("/api/radio/search", get(radio_search))
        .route("/api/radio/play", get(radio_play_url).post(radio_play_url))
        .route("/api/radio/play/:id", get(radio_play).post(radio_play))
        .route("/stream/all.m3u", get(stream_all))
        .route("/stream/favorites.m3u", get(stream_favorites))
        .route("/stream/playlist/:file", get(stream_playlist))
        .route("/stream/track/:file", get(stream_track))
        .route(
            "/mcp",
            get(|| async { StatusCode::METHOD_NOT_ALLOWED })
                .post(crate::mcp::handle)
                .delete(|| async { StatusCode::METHOD_NOT_ALLOWED }),
        )
        .layer(middleware::from_fn_with_state(state.clone(), guard))
        .with_state(state)
}

/// Startet den Server und läuft, bis er beendet wird.
pub async fn serve(player: Arc<Player>, addr: SocketAddr, token: Option<String>) -> anyhow::Result<()> {
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .with_context(|| format!("Port {addr} ist belegt oder nicht erreichbar"))?;
    log::info!("API läuft auf http://{addr}");
    axum::serve(listener, router(player, token).into_make_service_with_connect_info::<SocketAddr>()).await?;
    Ok(())
}

// ---------- Zugriffsschutz ----------

fn query_param<'a>(query: &'a str, key: &str) -> Option<&'a str> {
    query
        .split('&')
        .filter_map(|kv| kv.split_once('='))
        .find(|(k, _)| *k == key)
        .map(|(_, v)| v)
}

fn is_local_origin(origin: &HeaderValue) -> bool {
    let Ok(o) = origin.to_str() else { return false };
    if o.starts_with("tauri://") {
        return true;
    }
    let host = o
        .split_once("://")
        .map_or(o, |(_, rest)| rest)
        .split('/')
        .next()
        .unwrap_or("");
    let host = match host.rsplit_once(':') {
        Some((h, port)) if port.chars().all(|c| c.is_ascii_digit()) => h,
        _ => host,
    };
    matches!(host, "localhost" | "127.0.0.1" | "[::1]" | "tauri.localhost")
}

/// Mit Token: Geräte im Netz müssen es mitschicken (Programme auf diesem PC nicht).
/// Außerdem werden Anfragen fremder Webseiten abgewiesen, damit keine Seite im Browser
/// heimlich den Player steuert.
async fn guard(State(st): State<ApiState>, req: Request, next: Next) -> Response {
    let local = req
        .extensions()
        .get::<ConnectInfo<SocketAddr>>()
        .is_none_or(|ConnectInfo(addr)| addr.ip().is_loopback());
    if let Some(origin) = req.headers().get(header::ORIGIN) {
        if !is_local_origin(origin) {
            return ApiError(StatusCode::FORBIDDEN, "Fremde Herkunft nicht erlaubt".into()).into_response();
        }
    }
    if let (Some(token), false) = (&st.token, local) {
        let bearer = req
            .headers()
            .get(header::AUTHORIZATION)
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.strip_prefix("Bearer "));
        let query = req.uri().query().and_then(|q| query_param(q, "token"));
        if bearer.or(query) != Some(token.as_ref()) {
            return ApiError(StatusCode::UNAUTHORIZED, "Token fehlt oder ist falsch".into())
                .into_response();
        }
    }
    next.run(req).await
}

// ---------- Handler ----------

async fn index() -> Json<Value> {
    Json(json!({
        "name": "Amber Player",
        "version": env!("CARGO_PKG_VERSION"),
        "endpoints": [
            "/api/status", "/api/play", "/api/pause", "/api/toggle", "/api/stop",
            "/api/next", "/api/prev", "/api/volume?value=50", "/api/volume/up",
            "/api/volume/down", "/api/mute", "/api/seek?seconds=60", "/api/seek/forward",
            "/api/seek/back", "/api/shuffle", "/api/repeat", "/api/sleep?minutes=30",
            "/api/favorite", "/api/search?q=", "/api/play/track/{id}",
            "/api/play/music?artist=&album=&genre=&favorites=true&shuffle=true",
            "/api/queue", "/api/queue/add/{id}", "/api/queue/next/{id}", "/api/queue/clear",
            "/api/playlists", "/api/playlists/{id}", "/api/playlists/{id}/play?shuffle=true",
            "/api/effects", "/api/effects/{id}?enabled=true&preset=Rock",
            "/api/effects/{id}/toggle", "/api/effects/{id}/param/{param}?value=3",
            "/api/cover/current", "/api/cover/{id}", "/api/library", "/api/library/scan",
            "/api/online", "/api/online/search?q=&source=", "/api/online/play?q=",
            "/api/online/import?url=", "/api/radio", "/api/radio/search?q=&country=DE", "/api/radio/play/{id}",
            "/api/radio/play?url=&name=", "/stream/all.m3u", "/stream/favorites.m3u",
            "/stream/playlist/{id}.m3u", "/stream/track/{id}.mp3",
            "/mcp"
        ]
    }))
}

async fn status(State(st): State<ApiState>) -> Json<Status> {
    Json(st.player.status())
}

#[derive(Deserialize)]
struct ValueQ {
    value: Option<u8>,
}

async fn volume(State(st): State<ApiState>, Query(q): Query<ValueQ>) -> Json<Status> {
    match q.value {
        Some(volume) => run(&st, Command::SetVolume { volume }).await,
        None => Json(st.player.status()),
    }
}

#[derive(Deserialize)]
struct SeekQ {
    seconds: Option<f64>,
}

async fn seek(State(st): State<ApiState>, Query(q): Query<SeekQ>) -> Json<Status> {
    match q.seconds {
        Some(s) => {
            let ms = (s.max(0.0) * 1000.0) as u64;
            run(&st, Command::Seek { ms }).await
        }
        None => Json(st.player.status()),
    }
}

#[derive(Deserialize)]
struct OnQ {
    on: Option<bool>,
}

async fn shuffle(State(st): State<ApiState>, Query(q): Query<OnQ>) -> Json<Status> {
    run(&st, Command::SetShuffle { on: q.on }).await
}

#[derive(Deserialize)]
struct RepeatQ {
    mode: Option<String>,
}

async fn repeat(State(st): State<ApiState>, Query(q): Query<RepeatQ>) -> ApiResult<Json<Status>> {
    let mode = match q.mode.as_deref() {
        None => None,
        Some(m) => Some(Repeat::parse(m).ok_or_else(|| {
            ApiError(StatusCode::BAD_REQUEST, "mode muss off, all oder one sein".into())
        })?),
    };
    Ok(run(&st, Command::SetRepeat { mode }).await)
}

#[derive(Deserialize)]
struct SleepQ {
    minutes: Option<u64>,
}

async fn sleep(State(st): State<ApiState>, Query(q): Query<SleepQ>) -> Json<Status> {
    run(&st, Command::SleepTimer { minutes: q.minutes }).await
}

/// Schaltet den Favoriten-Status des laufenden Titels um.
async fn favorite(State(st): State<ApiState>) -> ApiResult<Json<Status>> {
    let p = st.player.clone();
    let status = blocking(move || -> anyhow::Result<Status> {
        let Some(t) = p.status().track else {
            anyhow::bail!("Es läuft gerade kein Titel");
        };
        p.library().set_favorite(t.id, !t.favorite)?;
        Ok(p.send(Command::Refresh))
    })
    .await?;
    Ok(Json(status))
}

#[derive(Deserialize)]
struct SearchQ {
    q: String,
    limit: Option<u32>,
}

async fn search(State(st): State<ApiState>, Query(q): Query<SearchQ>) -> ApiResult<Json<Value>> {
    let lib = st.player.library().clone();
    let tracks = blocking(move || {
        lib.tracks(&TrackQuery {
            search: Some(q.q),
            sort: Sort::Artist,
            limit: Some(q.limit.unwrap_or(50)),
            ..Default::default()
        })
    })
    .await?;
    Ok(Json(json!(tracks)))
}

async fn play_track(State(st): State<ApiState>, Path(id): Path<i64>) -> ApiResult<Json<Status>> {
    let lib = st.player.library().clone();
    if blocking(move || lib.track(id)).await?.is_none() {
        return Err(not_found("Titel"));
    }
    Ok(run(&st, Command::PlayList { ids: vec![id], start: 0 }).await)
}

#[derive(Deserialize, Default)]
#[serde(default)]
pub(crate) struct MusicQ {
    pub artist: Option<String>,
    pub album: Option<String>,
    pub genre: Option<String>,
    pub search: Option<String>,
    pub favorites: bool,
    pub shuffle: Option<bool>,
}

/// Spielt eine Auswahl aus der Bibliothek (ohne Filter: alles).
pub(crate) fn play_selection(player: &Player, q: MusicQ) -> anyhow::Result<Status> {
    let sort = if q.album.is_some() { Sort::Album } else { Sort::Artist };
    let ids: Vec<i64> = player
        .library()
        .tracks(&TrackQuery {
            search: q.search,
            artist: q.artist,
            album: q.album,
            genre: q.genre,
            favorites: q.favorites,
            sort,
            limit: None,
        })?
        .iter()
        .map(|t| t.id)
        .collect();
    if ids.is_empty() {
        anyhow::bail!("Keine passenden Titel gefunden");
    }
    play_ids(player, ids, q.shuffle)
}

/// Spielt eine Liste; mit Shuffle beginnt sie an einer zufälligen Stelle.
pub(crate) fn play_ids(player: &Player, ids: Vec<i64>, shuffle: Option<bool>) -> anyhow::Result<Status> {
    if ids.is_empty() {
        anyhow::bail!("Die Liste ist leer");
    }
    if let Some(on) = shuffle {
        player.send(Command::SetShuffle { on: Some(on) });
    }
    let start = if shuffle == Some(true) {
        rand::Rng::gen_range(&mut rand::thread_rng(), 0..ids.len())
    } else {
        0
    };
    Ok(player.send(Command::PlayList { ids, start }))
}

async fn play_music(State(st): State<ApiState>, Query(q): Query<MusicQ>) -> ApiResult<Json<Status>> {
    let p = st.player.clone();
    Ok(Json(blocking(move || play_selection(&p, q)).await?))
}

async fn queue(State(st): State<ApiState>) -> Json<Value> {
    let p = st.player.clone();
    Json(json!(blocking(move || p.upcoming(100)).await))
}

async fn queue_add(State(st): State<ApiState>, Path(id): Path<i64>) -> Json<Status> {
    run(&st, Command::Enqueue { id }).await
}

async fn queue_next(State(st): State<ApiState>, Path(id): Path<i64>) -> Json<Status> {
    run(&st, Command::PlayNext { id }).await
}

async fn playlists(State(st): State<ApiState>) -> ApiResult<Json<Value>> {
    let lib = st.player.library().clone();
    Ok(Json(json!(blocking(move || lib.playlists()).await?)))
}

async fn playlist(State(st): State<ApiState>, Path(id): Path<i64>) -> ApiResult<Json<Value>> {
    let lib = st.player.library().clone();
    let (pl, tracks) = blocking(move || -> anyhow::Result<_> {
        Ok((lib.playlist(id)?, lib.playlist_tracks(id)?))
    })
    .await?;
    let pl = pl.ok_or_else(|| not_found("Playlist"))?;
    Ok(Json(json!({ "playlist": pl, "tracks": tracks })))
}

async fn playlist_play(
    State(st): State<ApiState>,
    Path(id): Path<i64>,
    Query(q): Query<OnShuffleQ>,
) -> ApiResult<Json<Status>> {
    let p = st.player.clone();
    let status = blocking(move || {
        let ids = p.library().playlist_track_ids(id)?;
        play_ids(&p, ids, q.shuffle)
    })
    .await?;
    Ok(Json(status))
}

#[derive(Deserialize)]
struct OnShuffleQ {
    shuffle: Option<bool>,
}

async fn effects(State(st): State<ApiState>) -> Json<Value> {
    Json(json!(st.player.effects().list()))
}

#[derive(Deserialize)]
struct EffectQ {
    enabled: Option<bool>,
    preset: Option<String>,
    /// Kommagetrennte Werte, z. B. `values=3,2,0,0,0,0,0,1,2,3`
    values: Option<String>,
}

async fn effect(
    State(st): State<ApiState>,
    Path(id): Path<String>,
    Query(q): Query<EffectQ>,
) -> ApiResult<Json<Value>> {
    let host = st.player.effects();
    let mut info = host.get(&id)?;
    if let Some(preset) = &q.preset {
        info = host.apply_preset(&id, preset)?;
    }
    let values: Option<Vec<f32>> = match &q.values {
        Some(v) => Some(
            v.split(',')
                .map(|x| x.trim().parse::<f32>())
                .collect::<Result<_, _>>()
                .map_err(|_| ApiError(StatusCode::BAD_REQUEST, "values: Zahlen mit Komma trennen".into()))?,
        ),
        None => None,
    };
    if q.enabled.is_some() || values.is_some() {
        info = host.set(&id, q.enabled, values.as_deref())?;
    }
    Ok(Json(json!(info)))
}

async fn effect_toggle(State(st): State<ApiState>, Path(id): Path<String>) -> ApiResult<Json<Value>> {
    let host = st.player.effects();
    let on = !host.get(&id)?.enabled;
    Ok(Json(json!(host.set(&id, Some(on), None)?)))
}

async fn effect_param(
    State(st): State<ApiState>,
    Path((id, param)): Path<(String, String)>,
    Query(q): Query<FloatQ>,
) -> ApiResult<Json<Value>> {
    let value = q
        .value
        .ok_or_else(|| ApiError(StatusCode::BAD_REQUEST, "value fehlt".into()))?;
    Ok(Json(json!(st.player.effects().set_param(&id, &param, value)?)))
}

#[derive(Deserialize)]
struct FloatQ {
    value: Option<f32>,
}

async fn cover_response(st: &ApiState, id: i64) -> ApiResult<Response> {
    let lib = st.player.library().clone();
    let (data, mime) = blocking(move || lib.cover(id))
        .await?
        .ok_or_else(|| not_found("Cover"))?;
    let mut headers = HeaderMap::new();
    headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_str(&mime).unwrap_or(HeaderValue::from_static("image/jpeg")),
    );
    headers.insert(header::CACHE_CONTROL, HeaderValue::from_static("max-age=60"));
    Ok((headers, data).into_response())
}

async fn cover_current(State(st): State<ApiState>) -> ApiResult<Response> {
    let id = st
        .player
        .status()
        .track
        .map(|t| t.id)
        .ok_or_else(|| not_found("Laufender Titel"))?;
    cover_response(&st, id).await
}

async fn cover(State(st): State<ApiState>, Path(id): Path<i64>) -> ApiResult<Response> {
    cover_response(&st, id).await
}

async fn library_stats(State(st): State<ApiState>) -> ApiResult<Json<Value>> {
    let lib = st.player.library().clone();
    let v = blocking(move || -> anyhow::Result<Value> {
        Ok(json!({
            "tracks": lib.track_count()?,
            "albums": lib.albums()?.len(),
            "artists": lib.artists()?.len(),
            "playlists": lib.playlists()?.len(),
            "folders": lib.folders()?,
            "scanning": lib.is_scanning(),
        }))
    })
    .await?;
    Ok(Json(v))
}

async fn library_scan(State(st): State<ApiState>) -> ApiResult<Json<Value>> {
    let lib = st.player.library().clone();
    if lib.is_scanning() {
        return Ok(Json(json!({ "started": false, "scanning": true })));
    }
    tokio::task::spawn_blocking(move || {
        if let Err(e) = lib.scan_all(&mut |_, _| {}) {
            log::warn!("Scan fehlgeschlagen: {e:#}");
        }
    });
    Ok(Json(json!({ "started": true, "scanning": true })))
}

// ---------- Online-Quellen (Quellen-Plugins) ----------

async fn online_list(State(st): State<ApiState>) -> Json<Value> {
    Json(json!(st.player.sources().list()))
}

#[derive(Deserialize, Default)]
#[serde(default)]
struct OnlineQ {
    source: Option<String>,
    q: String,
    url: String,
    limit: Option<u32>,
    shuffle: Option<bool>,
}

async fn online_search(State(st): State<ApiState>, Query(q): Query<OnlineQ>) -> ApiResult<Json<Value>> {
    let p = st.player.clone();
    let tracks = blocking(move || {
        crate::sources::search_tracks(p.library(), p.sources(), q.source.as_deref(), &q.q, q.limit.unwrap_or(25))
    })
    .await?;
    Ok(Json(json!(tracks)))
}

/// Online suchen und die Treffer abspielen (für Stream-Deck-Tasten und Sprachbefehle).
async fn online_play(State(st): State<ApiState>, Query(q): Query<OnlineQ>) -> ApiResult<Json<Status>> {
    let p = st.player.clone();
    let status = blocking(move || {
        let tracks = if q.url.is_empty() {
            crate::sources::search_tracks(p.library(), p.sources(), q.source.as_deref(), &q.q, q.limit.unwrap_or(25))?
        } else {
            crate::sources::playlist_tracks(p.library(), p.sources(), q.source.as_deref(), &q.url)?.1
        };
        play_ids(&p, tracks.iter().map(|t| t.id).collect(), q.shuffle)
    })
    .await?;
    Ok(Json(status))
}

async fn online_import(State(st): State<ApiState>, Query(q): Query<OnlineQ>) -> ApiResult<Json<Value>> {
    let p = st.player.clone();
    let pl = blocking(move || {
        let id = crate::sources::import_playlist(p.library(), p.sources(), q.source.as_deref(), &q.url)?;
        p.library().playlist(id)
    })
    .await?;
    Ok(Json(json!(pl)))
}

// ---------- Internetradio ----------

async fn radio_list(State(st): State<ApiState>) -> ApiResult<Json<Value>> {
    let lib = st.player.library().clone();
    Ok(Json(json!(blocking(move || lib.stations()).await?)))
}

#[derive(Deserialize, Default)]
#[serde(default)]
struct RadioSearchQ {
    q: String,
    country: String,
    limit: Option<u32>,
}

async fn radio_search(Query(q): Query<RadioSearchQ>) -> ApiResult<Json<Value>> {
    let list = blocking(move || crate::radio::search(&q.q, &q.country, q.limit.unwrap_or(60))).await?;
    Ok(Json(json!(list)))
}

async fn radio_play(State(st): State<ApiState>, Path(id): Path<i64>) -> ApiResult<Json<Status>> {
    let lib = st.player.library().clone();
    let station = blocking(move || lib.station(id)).await?.ok_or_else(|| not_found("Sender"))?;
    Ok(run(&st, Command::PlayStation { station }).await)
}

#[derive(Deserialize)]
struct RadioUrlQ {
    url: String,
    name: Option<String>,
}

async fn radio_play_url(State(st): State<ApiState>, Query(q): Query<RadioUrlQ>) -> ApiResult<Json<Status>> {
    if !(q.url.starts_with("http://") || q.url.starts_with("https://")) {
        return Err(ApiError(StatusCode::BAD_REQUEST, "url muss mit http:// oder https:// beginnen".into()));
    }
    let station = crate::radio::Station {
        name: q.name.unwrap_or_else(|| q.url.clone()),
        url: q.url,
        ..Default::default()
    };
    Ok(run(&st, Command::PlayStation { station }).await)
}

// ---------- Streams fürs Handy ----------
// Einzelne Titel lassen sich in Poweramp unter „Streams › URL hinzufügen“ eintragen. Ganze Listen
// lädt man als .m3u aufs Handy und öffnet sie mit Poweramp (Poweramp liest keine Listen aus dem Netz).

/// „http://<Adresse wie vom Handy aufgerufen>“ – so stimmen die Links auch im Heimnetz.
fn base_url(headers: &HeaderMap) -> String {
    let host = headers.get(header::HOST).and_then(|h| h.to_str().ok()).unwrap_or("127.0.0.1");
    format!("http://{host}")
}

fn track_ext(path: &str) -> String {
    std::path::Path::new(path)
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_ascii_lowercase())
        .unwrap_or_else(|| "mp3".into())
}

/// M3U-Datei (Download) mit Stream-Links auf die einzelnen Titel.
fn m3u_response(st: &ApiState, headers: &HeaderMap, name: &str, tracks: &[crate::model::Track]) -> Response {
    let base = base_url(headers);
    let token = st.token.as_deref().map(|t| format!("?token={t}")).unwrap_or_default();
    let mut body = format!("#EXTM3U\n#PLAYLIST:{name}\n");
    // Online-Titel lassen sich nicht weiterreichen – nur eigene Dateien.
    for t in tracks.iter().filter(|t| !t.path.starts_with(crate::sources::PATH_PREFIX)) {
        let label = if t.artist.is_empty() { t.title.clone() } else { format!("{} - {}", t.artist, t.title) };
        body.push_str(&format!(
            "#EXTINF:{},{}\n{base}/stream/track/{}.{}{token}\n",
            t.duration_ms / 1000,
            label.replace(['\n', '\r'], " "),
            t.id,
            track_ext(&t.path)
        ));
    }
    let mut h = HeaderMap::new();
    h.insert(header::CONTENT_TYPE, HeaderValue::from_static("audio/x-mpegurl; charset=utf-8"));
    // Als Datei mit dem Namen der Playlist herunterladen (Umlaute per RFC 5987).
    let safe: String = name.chars().map(|c| if c.is_alphanumeric() || c == ' ' || c == '-' { c } else { '_' }).collect();
    let ascii: String = safe.chars().map(|c| if c.is_ascii() { c } else { '_' }).collect();
    let encoded: String = safe
        .bytes()
        .map(|b| if b.is_ascii_alphanumeric() || b == b'-' { (b as char).to_string() } else { format!("%{b:02X}") })
        .collect();
    if let Ok(v) = HeaderValue::from_str(&format!("attachment; filename=\"{ascii}.m3u\"; filename*=UTF-8''{encoded}.m3u")) {
        h.insert(header::CONTENT_DISPOSITION, v);
    }
    (h, body).into_response()
}

async fn stream_all(State(st): State<ApiState>, headers: HeaderMap) -> ApiResult<Response> {
    let lib = st.player.library().clone();
    let tracks = blocking(move || lib.tracks(&TrackQuery { sort: Sort::Artist, ..Default::default() })).await?;
    Ok(m3u_response(&st, &headers, "Alle Titel", &tracks))
}

async fn stream_favorites(State(st): State<ApiState>, headers: HeaderMap) -> ApiResult<Response> {
    let lib = st.player.library().clone();
    let tracks = blocking(move || {
        lib.tracks(&TrackQuery { favorites: true, sort: Sort::Artist, ..Default::default() })
    })
    .await?;
    Ok(m3u_response(&st, &headers, "Favoriten", &tracks))
}

/// `{id}` oder `{id}.endung` → id
fn id_from_file(file: &str) -> ApiResult<i64> {
    file.split('.').next().and_then(|s| s.parse().ok()).ok_or_else(|| not_found("Eintrag"))
}

async fn stream_playlist(State(st): State<ApiState>, headers: HeaderMap, Path(file): Path<String>) -> ApiResult<Response> {
    let id = id_from_file(&file)?;
    let lib = st.player.library().clone();
    let (pl, tracks) = blocking(move || -> anyhow::Result<_> { Ok((lib.playlist(id)?, lib.playlist_tracks(id)?)) }).await?;
    let pl = pl.ok_or_else(|| not_found("Playlist"))?;
    Ok(m3u_response(&st, &headers, &pl.name, &tracks))
}

fn audio_mime(ext: &str) -> &'static str {
    match ext {
        "mp3" => "audio/mpeg",
        "flac" => "audio/flac",
        "ogg" | "oga" | "opus" => "audio/ogg",
        "m4a" | "mp4" | "aac" => "audio/mp4",
        "wav" => "audio/wav",
        "aif" | "aiff" => "audio/aiff",
        "webm" | "mka" => "audio/webm",
        _ => "application/octet-stream",
    }
}

/// `bytes=start-end` (end optional) → (start, end inklusive)
fn parse_range(h: &str, size: u64) -> Option<(u64, u64)> {
    let spec = h.strip_prefix("bytes=")?.split(',').next()?.trim();
    let (a, b) = spec.split_once('-')?;
    let (start, end) = if a.is_empty() {
        let n: u64 = b.parse().ok()?;
        (size.saturating_sub(n), size.checked_sub(1)?)
    } else {
        let start: u64 = a.parse().ok()?;
        let end = if b.is_empty() { size.checked_sub(1)? } else { b.parse::<u64>().ok()?.min(size.checked_sub(1)?) };
        (start, end)
    };
    (start <= end && start < size).then_some((start, end))
}

/// Liefert eine Audiodatei; mit Range-Anfragen kann das Handy im Titel springen.
async fn stream_track(State(st): State<ApiState>, headers: HeaderMap, Path(file): Path<String>) -> ApiResult<Response> {
    use tokio::io::{AsyncReadExt, AsyncSeekExt};
    let id = id_from_file(&file)?;
    let lib = st.player.library().clone();
    let track = blocking(move || lib.track(id)).await?.ok_or_else(|| not_found("Titel"))?;
    let mut f = tokio::fs::File::open(&track.path)
        .await
        .map_err(|_| not_found("Datei"))?;
    let size = f.metadata().await.map_err(|e| ApiError(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?.len();

    let mut h = HeaderMap::new();
    h.insert(header::CONTENT_TYPE, HeaderValue::from_static(audio_mime(&track_ext(&track.path))));
    h.insert(header::ACCEPT_RANGES, HeaderValue::from_static("bytes"));
    let range = headers.get(header::RANGE).and_then(|v| v.to_str().ok());
    let (status, start, len) = match range {
        Some(r) => match parse_range(r, size) {
            Some((a, b)) => {
                h.insert(header::CONTENT_RANGE, HeaderValue::from_str(&format!("bytes {a}-{b}/{size}")).unwrap());
                (StatusCode::PARTIAL_CONTENT, a, b - a + 1)
            }
            None => {
                h.insert(header::CONTENT_RANGE, HeaderValue::from_str(&format!("bytes */{size}")).unwrap());
                return Ok((StatusCode::RANGE_NOT_SATISFIABLE, h).into_response());
            }
        },
        None => (StatusCode::OK, 0, size),
    };
    h.insert(header::CONTENT_LENGTH, HeaderValue::from(len));
    f.seek(std::io::SeekFrom::Start(start))
        .await
        .map_err(|e| ApiError(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    let body = axum::body::Body::from_stream(tokio_util::io::ReaderStream::new(f.take(len)));
    Ok((status, h, body).into_response())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ranges() {
        assert_eq!(parse_range("bytes=0-", 100), Some((0, 99)));
        assert_eq!(parse_range("bytes=10-19", 100), Some((10, 19)));
        assert_eq!(parse_range("bytes=-10", 100), Some((90, 99)));
        assert_eq!(parse_range("bytes=50-500", 100), Some((50, 99)));
        assert_eq!(parse_range("bytes=200-", 100), None);
        assert_eq!(parse_range("items=0-1", 100), None);
    }

    #[test]
    fn origin_check() {
        let ok = ["http://localhost:1420", "http://127.0.0.1", "tauri://localhost", "http://tauri.localhost"];
        for o in ok {
            assert!(is_local_origin(&HeaderValue::from_static(o)), "{o}");
        }
        let bad = ["https://evil.example", "http://localhost.evil.com", "null"];
        for o in bad {
            assert!(!is_local_origin(&HeaderValue::from_static(o)), "{o}");
        }
    }
}
