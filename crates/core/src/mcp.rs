//! MCP-Server (Model Context Protocol) über „Streamable HTTP“ unter `/mcp`.
//!
//! Zustandslos: jede POST-Anfrage enthält eine JSON-RPC-Nachricht (oder einen Batch)
//! und wird direkt mit JSON beantwortet. Einrichtung in Claude Code:
//! `claude mcp add --transport http amber-player http://127.0.0.1:7878/mcp`

use std::sync::Arc;

use anyhow::{anyhow, bail, Result};
use axum::body::Bytes;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde_json::{json, Value};

use crate::api::{blocking, play_ids, play_selection, ApiState, MusicQ};
use crate::library::{Sort, TrackQuery};
use crate::model::{Repeat, Track};
use crate::player::{Command, Player};

const PROTOCOL_VERSIONS: &[&str] = &["2025-06-18", "2025-03-26", "2024-11-05"];

pub async fn handle(State(st): State<ApiState>, body: Bytes) -> Response {
    let msg: Value = match serde_json::from_slice(&body) {
        Ok(v) => v,
        Err(e) => {
            return Json(rpc_error(Value::Null, -32700, &format!("Ungültiges JSON: {e}")))
                .into_response()
        }
    };
    let player = st.player.clone();
    let replies = blocking(move || match msg {
        Value::Array(batch) => batch
            .into_iter()
            .filter_map(|m| handle_message(&player, m))
            .collect::<Vec<_>>(),
        m => handle_message(&player, m).into_iter().collect(),
    })
    .await;

    match replies.len() {
        0 => StatusCode::ACCEPTED.into_response(),
        1 if !body.trim_ascii_start().starts_with(b"[") => {
            Json(replies.into_iter().next().unwrap()).into_response()
        }
        _ => Json(Value::Array(replies)).into_response(),
    }
}

fn rpc_error(id: Value, code: i64, message: &str) -> Value {
    json!({ "jsonrpc": "2.0", "id": id, "error": { "code": code, "message": message } })
}

/// Liefert `None` für Benachrichtigungen (Nachrichten ohne `id`) und Antworten.
fn handle_message(player: &Arc<Player>, msg: Value) -> Option<Value> {
    let id = msg.get("id").cloned()?;
    let Some(method) = msg.get("method").and_then(Value::as_str) else {
        return Some(rpc_error(id, -32600, "Methode fehlt"));
    };
    let params = msg.get("params").cloned().unwrap_or(Value::Null);
    let result = match method {
        "initialize" => {
            let requested = params
                .get("protocolVersion")
                .and_then(Value::as_str)
                .unwrap_or("");
            let version = PROTOCOL_VERSIONS
                .iter()
                .find(|v| **v == requested)
                .unwrap_or(&PROTOCOL_VERSIONS[0]);
            json!({
                "protocolVersion": version,
                "capabilities": { "tools": { "listChanged": false } },
                "serverInfo": {
                    "name": "amber-player",
                    "title": "Amber Player Musik-Player",
                    "version": env!("CARGO_PKG_VERSION")
                },
                "instructions": "Steuert den lokalen Musik-Player Amber Player: Wiedergabe, Lautstärke, \
                    Suche in der Musikbibliothek, Playlisten, Warteschlange und Klangeffekte \
                    (Equalizer usw.). Titel-IDs bekommst du über search_library oder get_playlist."
            })
        }
        "ping" => json!({}),
        "tools/list" => json!({ "tools": tool_definitions() }),
        "tools/call" => {
            let name = params.get("name").and_then(Value::as_str).unwrap_or("");
            let args = params.get("arguments").cloned().unwrap_or(json!({}));
            match call_tool(player, name, &args) {
                Ok(v) => json!({
                    "content": [{ "type": "text", "text": serde_json::to_string_pretty(&v).unwrap_or_default() }],
                    "isError": false
                }),
                Err(e) => json!({
                    "content": [{ "type": "text", "text": format!("Fehler: {e:#}") }],
                    "isError": true
                }),
            }
        }
        other => return Some(rpc_error(id, -32601, &format!("Unbekannte Methode: {other}"))),
    };
    Some(json!({ "jsonrpc": "2.0", "id": id, "result": result }))
}

fn tool(name: &str, description: &str, properties: Value, required: &[&str]) -> Value {
    json!({
        "name": name,
        "description": description,
        "inputSchema": {
            "type": "object",
            "properties": properties,
            "required": required,
        }
    })
}

fn tool_definitions() -> Vec<Value> {
    let none = json!({});
    let ids = json!({ "type": "array", "items": { "type": "integer" } });
    vec![
        tool("get_status", "Aktueller Status: laufender Titel, Position, Lautstärke, Shuffle, Wiederholen.", none.clone(), &[]),
        tool("play", "Wiedergabe starten oder fortsetzen. Ist nichts geladen, wird die ganze Bibliothek gespielt.", none.clone(), &[]),
        tool("pause", "Wiedergabe pausieren.", none.clone(), &[]),
        tool("toggle", "Zwischen Abspielen und Pause umschalten.", none.clone(), &[]),
        tool("stop", "Wiedergabe stoppen.", none.clone(), &[]),
        tool("next", "Zum nächsten Titel springen.", none.clone(), &[]),
        tool("previous", "Zum vorigen Titel springen (oder an den Anfang, wenn schon >3 s gespielt).", none.clone(), &[]),
        tool("set_volume", "Lautstärke setzen.",
            json!({ "volume": { "type": "integer", "minimum": 0, "maximum": 100, "description": "0–100" } }), &["volume"]),
        tool("seek", "Im laufenden Titel an eine Stelle springen.",
            json!({ "seconds": { "type": "number", "minimum": 0 } }), &["seconds"]),
        tool("set_shuffle", "Zufallswiedergabe an- oder ausschalten.",
            json!({ "on": { "type": "boolean" } }), &["on"]),
        tool("set_repeat", "Wiederholen: off, all (ganze Liste) oder one (einen Titel).",
            json!({ "mode": { "type": "string", "enum": ["off", "all", "one"] } }), &["mode"]),
        tool("set_sleep_timer", "Pausiert die Wiedergabe nach X Minuten. 0 schaltet den Timer aus.",
            json!({ "minutes": { "type": "integer", "minimum": 0 } }), &["minutes"]),
        tool("search_library", "Sucht Titel nach Titel, Interpret oder Album. Liefert Titel mit IDs.",
            json!({
                "query": { "type": "string" },
                "limit": { "type": "integer", "minimum": 1, "maximum": 500, "default": 25 }
            }), &["query"]),
        tool("list_artists", "Alle Interpreten mit Titelanzahl.", none.clone(), &[]),
        tool("list_albums", "Alle Alben mit Interpret und Titelanzahl.", none.clone(), &[]),
        tool("play_music", "Spielt eine Auswahl aus der Bibliothek: nach Interpret, Album, Genre, Suchbegriff oder Favoriten. Ohne Filter: alles.",
            json!({
                "artist": { "type": "string" },
                "album": { "type": "string" },
                "genre": { "type": "string" },
                "search": { "type": "string" },
                "favorites": { "type": "boolean" },
                "shuffle": { "type": "boolean" }
            }), &[]),
        tool("play_tracks", "Spielt bestimmte Titel (IDs) in dieser Reihenfolge.",
            json!({ "track_ids": ids.clone(), "shuffle": { "type": "boolean" } }), &["track_ids"]),
        tool("add_to_queue", "Reiht einen Titel in die Warteschlange ein.",
            json!({
                "track_id": { "type": "integer" },
                "play_next": { "type": "boolean", "description": "true = direkt als Nächstes spielen" }
            }), &["track_id"]),
        tool("get_queue", "Die nächsten Titel in der Warteschlange.", none.clone(), &[]),
        tool("set_favorite", "Markiert einen Titel als Favorit (ohne track_id: der laufende Titel).",
            json!({ "track_id": { "type": "integer" }, "favorite": { "type": "boolean" } }), &["favorite"]),
        tool("list_playlists", "Alle Playlisten mit ID, Name und Titelanzahl.", none.clone(), &[]),
        tool("get_playlist", "Titel einer Playlist.",
            json!({ "playlist_id": { "type": "integer" } }), &["playlist_id"]),
        tool("create_playlist", "Legt eine neue Playlist an, optional gleich mit Titeln.",
            json!({ "name": { "type": "string" }, "track_ids": ids.clone() }), &["name"]),
        tool("add_to_playlist", "Hängt Titel an eine Playlist an.",
            json!({ "playlist_id": { "type": "integer" }, "track_ids": ids }), &["playlist_id", "track_ids"]),
        tool("play_playlist", "Spielt eine Playlist ab.",
            json!({ "playlist_id": { "type": "integer" }, "shuffle": { "type": "boolean" } }), &["playlist_id"]),
        tool("search_online", "Sucht Musik online über ein installiertes Quellen-Plugin. Liefert Titel mit IDs, die sich mit play_tracks, add_to_queue und add_to_playlist nutzen lassen.",
            json!({
                "query": { "type": "string" },
                "source": { "type": "string", "description": "Plugin-ID, z. B. meine-quelle (leer = erstes Plugin)" },
                "limit": { "type": "integer", "minimum": 1, "maximum": 50, "default": 10 }
            }), &["query"]),
        tool("play_online", "Sucht online über ein Quellen-Plugin und spielt die Treffer ab – oder spielt eine Online-Playlist per url.",
            json!({
                "query": { "type": "string" },
                "url": { "type": "string", "description": "Link einer Online-Playlist" },
                "source": { "type": "string" },
                "shuffle": { "type": "boolean" }
            }), &[]),
        tool("import_online_playlist", "Übernimmt eine Online-Playlist (Link einer Playlist beim Online-Dienst) als eigene Playlist.",
            json!({ "url": { "type": "string" }, "source": { "type": "string" } }), &["url"]),
        tool("search_radio", "Sucht Internetradio-Sender (radio-browser.info). Ohne Suchbegriff: beliebteste Sender.",
            json!({
                "query": { "type": "string", "description": "Name des Senders, z. B. „1Live“ oder „Rock Antenne“" },
                "country": { "type": "string", "description": "Ländercode, z. B. DE, AT, CH (leer = alle)" },
                "limit": { "type": "integer", "minimum": 1, "maximum": 100, "default": 20 }
            }), &[]),
        tool("list_stations", "Die gespeicherten Lieblingssender („Meine Sender“) mit ID.", json!({}), &[]),
        tool("play_radio", "Spielt einen Radiosender: gespeicherten Sender per station_id oder beliebige Stream-Adresse per url.",
            json!({
                "station_id": { "type": "integer" },
                "url": { "type": "string" },
                "name": { "type": "string" }
            }), &[]),
        tool("save_station", "Speichert einen Sender in „Meine Sender“.",
            json!({ "name": { "type": "string" }, "url": { "type": "string" } }), &["name", "url"]),
        tool("list_effects", "Alle Klangeffekt-Plugins (Equalizer, Crossfeed, Stereobreite, Limiter) mit Parametern, Werten und Presets.", none, &[]),
        tool("set_effect", "Schaltet ein Effekt-Plugin an/aus, setzt Werte oder lädt ein Preset (z. B. Equalizer-Preset „Rock“).",
            json!({
                "effect_id": { "type": "string", "description": "z. B. equalizer, crossfeed, stereo_width, limiter" },
                "enabled": { "type": "boolean" },
                "preset": { "type": "string" },
                "values": { "type": "array", "items": { "type": "number" }, "description": "Alle Parameterwerte in der Reihenfolge aus list_effects" }
            }), &["effect_id"]),
    ]
}

fn arg_i64(args: &Value, key: &str) -> Result<i64> {
    args.get(key)
        .and_then(Value::as_i64)
        .ok_or_else(|| anyhow!("Parameter „{key}“ fehlt oder ist keine Zahl"))
}

fn arg_bool(args: &Value, key: &str) -> Option<bool> {
    args.get(key).and_then(Value::as_bool)
}

fn arg_str(args: &Value, key: &str) -> Option<String> {
    args.get(key)
        .and_then(Value::as_str)
        .filter(|s| !s.is_empty())
        .map(str::to_owned)
}

fn arg_ids(args: &Value, key: &str) -> Result<Vec<i64>> {
    args.get(key)
        .and_then(Value::as_array)
        .ok_or_else(|| anyhow!("Parameter „{key}“ fehlt"))?
        .iter()
        .map(|v| v.as_i64().ok_or_else(|| anyhow!("„{key}“ darf nur Zahlen enthalten")))
        .collect()
}

/// Kurzform eines Titels, damit Antworten für das Modell übersichtlich bleiben.
fn brief(t: &Track) -> Value {
    json!({
        "id": t.id,
        "title": t.title,
        "artist": t.artist,
        "album": t.album,
        "duration_s": t.duration_ms / 1000,
        "favorite": t.favorite,
    })
}

fn call_tool(player: &Player, name: &str, args: &Value) -> Result<Value> {
    let lib = player.library();
    let status = |c: Command| -> Result<Value> { Ok(json!(player.send(c))) };
    match name {
        "get_status" => Ok(json!(player.status())),
        "play" => status(Command::Play),
        "pause" => status(Command::Pause),
        "toggle" => status(Command::Toggle),
        "stop" => status(Command::Stop),
        "next" => status(Command::Next),
        "previous" => status(Command::Prev),
        "set_volume" => {
            let volume = arg_i64(args, "volume")?.clamp(0, 100) as u8;
            status(Command::SetVolume { volume })
        }
        "seek" => {
            let s = args
                .get("seconds")
                .and_then(Value::as_f64)
                .ok_or_else(|| anyhow!("Parameter „seconds“ fehlt"))?;
            status(Command::Seek {
                ms: (s.max(0.0) * 1000.0) as u64,
            })
        }
        "set_shuffle" => status(Command::SetShuffle {
            on: Some(arg_bool(args, "on").ok_or_else(|| anyhow!("Parameter „on“ fehlt"))?),
        }),
        "set_repeat" => {
            let mode = arg_str(args, "mode")
                .and_then(|m| Repeat::parse(&m))
                .ok_or_else(|| anyhow!("mode muss off, all oder one sein"))?;
            status(Command::SetRepeat { mode: Some(mode) })
        }
        "set_sleep_timer" => {
            let minutes = arg_i64(args, "minutes")?.max(0) as u64;
            status(Command::SleepTimer {
                minutes: (minutes > 0).then_some(minutes),
            })
        }
        "search_library" => {
            let tracks = lib.tracks(&TrackQuery {
                search: arg_str(args, "query"),
                sort: Sort::Artist,
                limit: Some(args.get("limit").and_then(Value::as_u64).unwrap_or(25).min(500) as u32),
                ..Default::default()
            })?;
            Ok(json!(tracks.iter().map(brief).collect::<Vec<_>>()))
        }
        "list_artists" => Ok(json!(lib.artists()?)),
        "list_albums" => Ok(json!(lib.albums()?)),
        "play_music" => {
            let q = MusicQ {
                artist: arg_str(args, "artist"),
                album: arg_str(args, "album"),
                genre: arg_str(args, "genre"),
                search: arg_str(args, "search"),
                favorites: arg_bool(args, "favorites").unwrap_or(false),
                shuffle: arg_bool(args, "shuffle"),
            };
            Ok(json!(play_selection(player, q)?))
        }
        "play_tracks" => Ok(json!(play_ids(
            player,
            arg_ids(args, "track_ids")?,
            arg_bool(args, "shuffle")
        )?)),
        "add_to_queue" => {
            let id = arg_i64(args, "track_id")?;
            if lib.track(id)?.is_none() {
                bail!("Titel {id} gibt es nicht");
            }
            if arg_bool(args, "play_next").unwrap_or(false) {
                status(Command::PlayNext { id })
            } else {
                status(Command::Enqueue { id })
            }
        }
        "get_queue" => Ok(json!(player.upcoming(50).iter().map(brief).collect::<Vec<_>>())),
        "set_favorite" => {
            let fav = arg_bool(args, "favorite").ok_or_else(|| anyhow!("Parameter „favorite“ fehlt"))?;
            let id = match args.get("track_id").and_then(Value::as_i64) {
                Some(id) => id,
                None => player
                    .status()
                    .track
                    .map(|t| t.id)
                    .ok_or_else(|| anyhow!("Es läuft kein Titel"))?,
            };
            lib.set_favorite(id, fav)?;
            player.send(Command::Refresh);
            Ok(json!({ "track_id": id, "favorite": fav }))
        }
        "list_playlists" => Ok(json!(lib.playlists()?)),
        "get_playlist" => {
            let id = arg_i64(args, "playlist_id")?;
            let pl = lib.playlist(id)?.ok_or_else(|| anyhow!("Playlist {id} gibt es nicht"))?;
            let tracks: Vec<Value> = lib.playlist_tracks(id)?.iter().map(brief).collect();
            Ok(json!({ "playlist": pl, "tracks": tracks }))
        }
        "create_playlist" => {
            let name = arg_str(args, "name").ok_or_else(|| anyhow!("Parameter „name“ fehlt"))?;
            let id = lib.create_playlist(&name)?;
            if args.get("track_ids").is_some() {
                lib.add_to_playlist(id, &arg_ids(args, "track_ids")?)?;
            }
            Ok(json!(lib.playlist(id)?))
        }
        "add_to_playlist" => {
            let id = arg_i64(args, "playlist_id")?;
            if lib.playlist(id)?.is_none() {
                bail!("Playlist {id} gibt es nicht");
            }
            lib.add_to_playlist(id, &arg_ids(args, "track_ids")?)?;
            Ok(json!(lib.playlist(id)?))
        }
        "play_playlist" => {
            let id = arg_i64(args, "playlist_id")?;
            Ok(json!(play_ids(
                player,
                lib.playlist_track_ids(id)?,
                arg_bool(args, "shuffle")
            )?))
        }
        "search_online" => {
            let tracks = crate::sources::search_tracks(
                lib,
                player.sources(),
                arg_str(args, "source").as_deref(),
                &arg_str(args, "query").ok_or_else(|| anyhow!("Parameter „query“ fehlt"))?,
                args.get("limit").and_then(Value::as_u64).unwrap_or(10) as u32,
            )?;
            Ok(json!(tracks.iter().map(brief).collect::<Vec<_>>()))
        }
        "play_online" => {
            let source = arg_str(args, "source");
            let tracks = match arg_str(args, "url") {
                Some(url) => crate::sources::playlist_tracks(lib, player.sources(), source.as_deref(), &url)?.1,
                None => crate::sources::search_tracks(
                    lib,
                    player.sources(),
                    source.as_deref(),
                    &arg_str(args, "query").ok_or_else(|| anyhow!("query oder url angeben"))?,
                    15,
                )?,
            };
            Ok(json!(play_ids(player, tracks.iter().map(|t| t.id).collect(), arg_bool(args, "shuffle"))?))
        }
        "import_online_playlist" => {
            let url = arg_str(args, "url").ok_or_else(|| anyhow!("Parameter „url“ fehlt"))?;
            let id = crate::sources::import_playlist(lib, player.sources(), arg_str(args, "source").as_deref(), &url)?;
            Ok(json!(lib.playlist(id)?))
        }
        "search_radio" => {
            let list = crate::radio::search(
                &arg_str(args, "query").unwrap_or_default(),
                &arg_str(args, "country").unwrap_or_default(),
                args.get("limit").and_then(Value::as_u64).unwrap_or(20) as u32,
            )?;
            Ok(json!(list
                .iter()
                .map(|s| json!({ "name": s.name, "url": s.url, "country": s.country, "tags": s.tags, "codec": s.codec, "bitrate": s.bitrate }))
                .collect::<Vec<_>>()))
        }
        "list_stations" => Ok(json!(lib.stations()?)),
        "play_radio" => {
            let station = match args.get("station_id").and_then(Value::as_i64) {
                Some(id) => lib.station(id)?.ok_or_else(|| anyhow!("Sender {id} gibt es nicht"))?,
                None => {
                    let url = arg_str(args, "url").ok_or_else(|| anyhow!("station_id oder url angeben"))?;
                    crate::radio::Station { name: arg_str(args, "name").unwrap_or_else(|| url.clone()), url, ..Default::default() }
                }
            };
            status(Command::PlayStation { station })
        }
        "save_station" => {
            let station = crate::radio::Station {
                name: arg_str(args, "name").unwrap_or_default(),
                url: arg_str(args, "url").ok_or_else(|| anyhow!("Parameter „url“ fehlt"))?,
                ..Default::default()
            };
            let id = lib.add_station(&station)?;
            Ok(json!(lib.station(id)?))
        }
        "list_effects" => Ok(json!(player.effects().list())),
        "set_effect" => {
            let id = arg_str(args, "effect_id").ok_or_else(|| anyhow!("Parameter „effect_id“ fehlt"))?;
            let host = player.effects();
            let mut info = host.get(&id)?;
            if let Some(preset) = arg_str(args, "preset") {
                info = host.apply_preset(&id, &preset)?;
            }
            let values: Option<Vec<f32>> = args
                .get("values")
                .and_then(Value::as_array)
                .map(|a| a.iter().filter_map(Value::as_f64).map(|v| v as f32).collect());
            let enabled = arg_bool(args, "enabled");
            if enabled.is_some() || values.is_some() {
                info = host.set(&id, enabled, values.as_deref())?;
            }
            Ok(json!(info))
        }
        other => bail!("Unbekanntes Werkzeug: {other}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tool_names_are_unique() {
        let defs = tool_definitions();
        let mut names: Vec<&str> = defs.iter().map(|t| t["name"].as_str().unwrap()).collect();
        names.sort();
        let len = names.len();
        names.dedup();
        assert_eq!(names.len(), len);
    }
}
