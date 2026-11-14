//! Musikbibliothek, Playlisten und Einstellungen in SQLite.

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, MutexGuard};
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{bail, Context, Result};
use rusqlite::{params, params_from_iter, Connection, OptionalExtension, Row};
use serde::{Deserialize, Serialize};

use crate::m3u;
use crate::model::{Album, NamedCount, Playlist, Track, TrackInfo};
use crate::radio::Station;
use crate::scanner;

const SCHEMA: &str = "
CREATE TABLE IF NOT EXISTS folders (
    path TEXT PRIMARY KEY
);
CREATE TABLE IF NOT EXISTS tracks (
    id INTEGER PRIMARY KEY,
    path TEXT NOT NULL UNIQUE,
    title TEXT NOT NULL,
    artist TEXT NOT NULL DEFAULT '',
    album TEXT NOT NULL DEFAULT '',
    album_artist TEXT NOT NULL DEFAULT '',
    genre TEXT NOT NULL DEFAULT '',
    year INTEGER,
    track_no INTEGER,
    disc_no INTEGER,
    duration_ms INTEGER NOT NULL DEFAULT 0,
    has_cover INTEGER NOT NULL DEFAULT 0,
    mtime INTEGER NOT NULL DEFAULT 0,
    added_at INTEGER NOT NULL,
    play_count INTEGER NOT NULL DEFAULT 0,
    last_played INTEGER,
    favorite INTEGER NOT NULL DEFAULT 0
);
CREATE INDEX IF NOT EXISTS tracks_album ON tracks(album);
CREATE INDEX IF NOT EXISTS tracks_artist ON tracks(artist);
CREATE TABLE IF NOT EXISTS playlists (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL,
    created_at INTEGER NOT NULL
);
CREATE TABLE IF NOT EXISTS playlist_tracks (
    playlist_id INTEGER NOT NULL REFERENCES playlists(id) ON DELETE CASCADE,
    position INTEGER NOT NULL,
    track_id INTEGER NOT NULL REFERENCES tracks(id) ON DELETE CASCADE,
    PRIMARY KEY (playlist_id, position)
);
CREATE TABLE IF NOT EXISTS stations (
    id INTEGER PRIMARY KEY,
    uuid TEXT NOT NULL DEFAULT '',
    name TEXT NOT NULL,
    url TEXT NOT NULL UNIQUE,
    favicon TEXT NOT NULL DEFAULT '',
    homepage TEXT NOT NULL DEFAULT '',
    country TEXT NOT NULL DEFAULT '',
    tags TEXT NOT NULL DEFAULT '',
    codec TEXT NOT NULL DEFAULT '',
    bitrate INTEGER NOT NULL DEFAULT 0,
    position INTEGER NOT NULL DEFAULT 0
);
CREATE TABLE IF NOT EXISTS settings (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL
);
";

const TRACK_COLUMNS: &str = "id, path, title, artist, album, album_artist, genre, year, \
    track_no, disc_no, duration_ms, has_cover, play_count, favorite";

/// Interpret für Gruppierungen: Album-Interpret, sonst Interpret.
const ARTIST_EXPR: &str = "COALESCE(NULLIF(album_artist, ''), artist)";

/// Eigene Dateien (keine Online-Titel aus Quellen-Plugins).
const LOCAL_ONLY: &str = "substr(path, 1, 9) != 'plugin://'";

/// Online-Titel erscheinen nur, wenn sie benutzt wurden (gespielt, Favorit oder in einer Playlist).
const VISIBLE: &str = "(substr(path, 1, 9) != 'plugin://' OR play_count > 0 OR favorite = 1
    OR id IN (SELECT track_id FROM playlist_tracks))";

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Sort {
    #[default]
    Artist,
    Title,
    Album,
    Added,
    Recent,
    MostPlayed,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct TrackQuery {
    pub search: Option<String>,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub genre: Option<String>,
    pub favorites: bool,
    pub sort: Sort,
    pub limit: Option<u32>,
}

/// Ein Titel, für den ein Cover gesucht werden soll.
#[derive(Debug, Clone)]
pub struct CoverJob {
    pub id: i64,
    pub title: String,
    pub artist: String,
    pub album: String,
    pub album_artist: String,
}

#[derive(Debug, Clone, Copy, Default, Serialize)]
pub struct ScanResult {
    pub added_or_updated: usize,
    pub removed: usize,
    pub total: usize,
}

static DATA_DIR: std::sync::OnceLock<PathBuf> = std::sync::OnceLock::new();

/// Datenordner festlegen (vor dem ersten Öffnen der Bibliothek), z. B. auf Android.
pub fn set_data_dir(dir: PathBuf) {
    let _ = DATA_DIR.set(dir);
}

pub struct Library {
    conn: Mutex<Connection>,
    scanning: AtomicBool,
}

fn now() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

fn track_from_row(r: &Row) -> rusqlite::Result<Track> {
    Ok(Track {
        id: r.get(0)?,
        path: r.get(1)?,
        title: r.get(2)?,
        artist: r.get(3)?,
        album: r.get(4)?,
        album_artist: r.get(5)?,
        genre: r.get(6)?,
        year: r.get(7)?,
        track_no: r.get(8)?,
        disc_no: r.get(9)?,
        duration_ms: r.get::<_, i64>(10)? as u64,
        has_cover: r.get(11)?,
        play_count: r.get(12)?,
        favorite: r.get(13)?,
    })
}

fn trim_folder(folder: &str) -> &str {
    folder.trim_end_matches(['/', '\\'])
}

/// Ordnerpfad immer mit abschließendem Trenner, damit Präfixvergleiche eindeutig sind.
fn folder_prefix(folder: &str) -> String {
    let mut p = trim_folder(folder).to_owned();
    p.push(std::path::MAIN_SEPARATOR);
    p
}

impl Library {
    /// Datenordner von Amber Player (z. B. `~/.local/share/amber-player`; auf Android per
    /// [`set_data_dir`] vom App-Ordner vorgegeben).
    pub fn data_dir() -> PathBuf {
        DATA_DIR
            .get()
            .cloned()
            .unwrap_or_else(|| dirs::data_dir().unwrap_or_else(|| PathBuf::from(".")).join("amber-player"))
    }

    /// Unterordner für Plugins einer Art (`effects`, `visualizers`), wird bei Bedarf angelegt.
    pub fn plugin_dir(kind: &str) -> PathBuf {
        let dir = Self::data_dir().join(kind);
        let _ = std::fs::create_dir_all(&dir);
        dir
    }

    pub fn default_path() -> PathBuf {
        Self::data_dir().join("library.db")
    }

    pub fn open(path: &Path) -> Result<Self> {
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir)
                .with_context(|| format!("Ordner {} anlegen", dir.display()))?;
        }
        Self::init(Connection::open(path)?)
    }

    pub fn open_in_memory() -> Result<Self> {
        Self::init(Connection::open_in_memory()?)
    }

    fn init(conn: Connection) -> Result<Self> {
        conn.execute_batch("PRAGMA foreign_keys = ON; PRAGMA journal_mode = WAL;")?;
        conn.execute_batch(SCHEMA)?;
        // Spalten neuerer Versionen nachrüsten (Fehler = gibt es schon).
        let _ = conn.execute("ALTER TABLE tracks ADD COLUMN cover_path TEXT", []);
        let _ = conn.execute("ALTER TABLE tracks ADD COLUMN cover_checked INTEGER NOT NULL DEFAULT 0", []);
        Ok(Self {
            conn: Mutex::new(conn),
            scanning: AtomicBool::new(false),
        })
    }

    fn db(&self) -> MutexGuard<'_, Connection> {
        self.conn.lock().unwrap_or_else(|e| e.into_inner())
    }

    // ---------- Ordner & Scan ----------

    pub fn folders(&self) -> Result<Vec<String>> {
        let db = self.db();
        let mut st = db.prepare("SELECT path FROM folders ORDER BY path")?;
        let rows = st.query_map([], |r| r.get(0))?;
        Ok(rows.collect::<rusqlite::Result<_>>()?)
    }

    pub fn add_folder(&self, path: &str) -> Result<()> {
        if !Path::new(path).is_dir() {
            bail!("Ordner nicht gefunden: {path}");
        }
        let path = trim_folder(path);
        self.db()
            .execute("INSERT OR IGNORE INTO folders(path) VALUES (?1)", [path])?;
        Ok(())
    }

    /// Entfernt den Ordner und alle seine Titel aus der Bibliothek (Dateien bleiben erhalten).
    pub fn remove_folder(&self, path: &str) -> Result<()> {
        let path = trim_folder(path);
        let prefix = folder_prefix(path);
        let db = self.db();
        db.execute("DELETE FROM folders WHERE path = ?1", [path])?;
        db.execute(
            "DELETE FROM tracks WHERE substr(path, 1, length(?1)) = ?1",
            [&prefix],
        )?;
        Ok(())
    }

    /// Liest einen Ordner neu ein. Unveränderte Dateien (gleiche mtime) werden übersprungen.
    pub fn scan_folder(
        &self,
        folder: &str,
        progress: &mut dyn FnMut(usize, usize),
    ) -> Result<ScanResult> {
        let prefix = folder_prefix(folder);
        let known: HashMap<String, i64> = {
            let db = self.db();
            let mut st = db.prepare(
                "SELECT path, mtime FROM tracks WHERE substr(path, 1, length(?1)) = ?1",
            )?;
            let rows = st.query_map([&prefix], |r| Ok((r.get(0)?, r.get(1)?)))?;
            rows.collect::<rusqlite::Result<_>>()?
        };

        let files = scanner::find_audio_files(Path::new(folder));
        let total = files.len();
        let present: HashSet<&str> = files.iter().map(|(p, _)| p.as_str()).collect();

        let mut batch = Vec::new();
        let mut changed = 0;
        for (i, (path, mtime)) in files.iter().enumerate() {
            if known.get(path) != Some(mtime) {
                batch.push(scanner::read_track_info(Path::new(path), *mtime));
            }
            if batch.len() >= 200 {
                changed += batch.len();
                self.upsert_tracks(&batch)?;
                batch.clear();
            }
            if i % 25 == 0 {
                progress(i, total);
            }
        }
        changed += batch.len();
        self.upsert_tracks(&batch)?;

        let gone: Vec<&String> = known.keys().filter(|p| !present.contains(p.as_str())).collect();
        {
            let mut db = self.db();
            let tx = db.transaction()?;
            for p in &gone {
                tx.execute("DELETE FROM tracks WHERE path = ?1", [p])?;
            }
            tx.commit()?;
        }
        progress(total, total);
        Ok(ScanResult {
            added_or_updated: changed,
            removed: gone.len(),
            total,
        })
    }

    pub fn is_scanning(&self) -> bool {
        self.scanning.load(Ordering::Relaxed)
    }

    /// Liest alle Ordner neu ein. Läuft schon ein Scan, wird abgebrochen.
    pub fn scan_all(&self, progress: &mut dyn FnMut(usize, usize)) -> Result<ScanResult> {
        if self.scanning.swap(true, Ordering::AcqRel) {
            bail!("Die Bibliothek wird bereits eingelesen");
        }
        let result = (|| {
            // Neue Scanner-Version: alles neu einlesen, auch unveränderte Dateien.
            if self.setting_as::<u32>("scanner_version") != Some(scanner::SCANNER_VERSION) {
                self.db().execute("UPDATE tracks SET mtime = 0", [])?;
                self.set_setting_json("scanner_version", &scanner::SCANNER_VERSION)?;
            }
            let mut sum = ScanResult::default();
            for folder in self.folders()? {
                let r = self.scan_folder(&folder, progress)?;
                sum.added_or_updated += r.added_or_updated;
                sum.removed += r.removed;
                sum.total += r.total;
            }
            Ok(sum)
        })();
        self.scanning.store(false, Ordering::Release);
        result
    }

    pub fn upsert_tracks(&self, tracks: &[TrackInfo]) -> Result<()> {
        if tracks.is_empty() {
            return Ok(());
        }
        let mut db = self.db();
        let tx = db.transaction()?;
        {
            let mut st = tx.prepare(
                "INSERT INTO tracks (path, title, artist, album, album_artist, genre, year,
                    track_no, disc_no, duration_ms, has_cover, mtime, added_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)
                 ON CONFLICT(path) DO UPDATE SET
                    title = excluded.title, artist = excluded.artist, album = excluded.album,
                    album_artist = excluded.album_artist, genre = excluded.genre,
                    year = excluded.year, track_no = excluded.track_no,
                    disc_no = excluded.disc_no, duration_ms = excluded.duration_ms,
                    has_cover = MAX(excluded.has_cover, tracks.cover_path IS NOT NULL),
                    mtime = excluded.mtime",
            )?;
            let ts = now();
            for t in tracks {
                st.execute(params![
                    t.path,
                    t.title,
                    t.artist,
                    t.album,
                    t.album_artist,
                    t.genre,
                    t.year,
                    t.track_no,
                    t.disc_no,
                    t.duration_ms as i64,
                    t.has_cover,
                    t.mtime,
                    ts
                ])?;
            }
        }
        tx.commit()?;
        Ok(())
    }

    // ---------- Titel ----------

    pub fn track(&self, id: i64) -> Result<Option<Track>> {
        let db = self.db();
        let sql = format!("SELECT {TRACK_COLUMNS} FROM tracks WHERE id = ?1");
        Ok(db.query_row(&sql, [id], track_from_row).optional()?)
    }

    pub fn track_by_path(&self, path: &str) -> Result<Option<Track>> {
        let db = self.db();
        let sql = format!("SELECT {TRACK_COLUMNS} FROM tracks WHERE path = ?1");
        Ok(db.query_row(&sql, [path], track_from_row).optional()?)
    }

    /// Titel zu den IDs in derselben Reihenfolge; unbekannte IDs fallen weg.
    pub fn tracks_by_ids(&self, ids: &[i64]) -> Result<Vec<Track>> {
        let mut out = Vec::with_capacity(ids.len());
        for &id in ids {
            if let Some(t) = self.track(id)? {
                out.push(t);
            }
        }
        Ok(out)
    }

    pub fn tracks(&self, q: &TrackQuery) -> Result<Vec<Track>> {
        let mut wheres: Vec<String> = Vec::new();
        let mut args: Vec<String> = Vec::new();

        if let Some(search) = q.search.as_deref() {
            for word in search.split_whitespace() {
                args.push(format!("%{}%", word.replace('%', "\\%").replace('_', "\\_")));
                let n = args.len();
                wheres.push(format!(
                    "(title LIKE ?{n} ESCAPE '\\' OR artist LIKE ?{n} ESCAPE '\\' \
                     OR album LIKE ?{n} ESCAPE '\\' OR album_artist LIKE ?{n} ESCAPE '\\')"
                ));
            }
        }
        if let Some(artist) = &q.artist {
            args.push(artist.clone());
            wheres.push(format!(
                "({ARTIST_EXPR} = ?{n} OR artist = ?{n})",
                n = args.len()
            ));
        }
        if let Some(album) = &q.album {
            args.push(album.clone());
            wheres.push(format!("album = ?{}", args.len()));
        }
        if let Some(genre) = &q.genre {
            args.push(genre.clone());
            wheres.push(format!("genre = ?{}", args.len()));
        }
        if q.favorites {
            wheres.push("favorite = 1".into());
        }
        wheres.push(VISIBLE.into());
        match q.sort {
            Sort::Recent => wheres.push("last_played IS NOT NULL".into()),
            Sort::MostPlayed => wheres.push("play_count > 0".into()),
            _ => {}
        }

        let album_order = "album COLLATE NOCASE, disc_no, track_no, title COLLATE NOCASE";
        let order = match q.sort {
            Sort::Artist => format!("{ARTIST_EXPR} COLLATE NOCASE, year, {album_order}"),
            Sort::Title => "title COLLATE NOCASE".into(),
            Sort::Album => album_order.into(),
            Sort::Added => "added_at DESC, id DESC".into(),
            Sort::Recent => "last_played DESC".into(),
            Sort::MostPlayed => "play_count DESC, last_played DESC".into(),
        };
        let mut sql = format!("SELECT {TRACK_COLUMNS} FROM tracks");
        if !wheres.is_empty() {
            sql.push_str(" WHERE ");
            sql.push_str(&wheres.join(" AND "));
        }
        sql.push_str(" ORDER BY ");
        sql.push_str(&order);
        if let Some(limit) = q.limit {
            sql.push_str(&format!(" LIMIT {limit}"));
        }

        let db = self.db();
        let mut st = db.prepare(&sql)?;
        let rows = st.query_map(params_from_iter(args.iter()), track_from_row)?;
        Ok(rows.collect::<rusqlite::Result<_>>()?)
    }

    pub fn albums(&self) -> Result<Vec<Album>> {
        let db = self.db();
        let sql = format!(
            "SELECT album, {ARTIST_EXPR} AS a, MAX(year), COUNT(*),
                    MIN(CASE WHEN has_cover THEN id END)
             FROM tracks WHERE album != '' AND {LOCAL_ONLY}
             GROUP BY album, a ORDER BY album COLLATE NOCASE"
        );
        let mut st = db.prepare(&sql)?;
        let rows = st.query_map([], |r| {
            Ok(Album {
                name: r.get(0)?,
                artist: r.get(1)?,
                year: r.get(2)?,
                track_count: r.get(3)?,
                cover_track_id: r.get(4)?,
            })
        })?;
        Ok(rows.collect::<rusqlite::Result<_>>()?)
    }

    fn grouped(&self, expr: &str) -> Result<Vec<NamedCount>> {
        let db = self.db();
        let sql = format!(
            "SELECT {expr} AS n, COUNT(*) FROM tracks WHERE n != '' AND {LOCAL_ONLY}
             GROUP BY n ORDER BY n COLLATE NOCASE"
        );
        let mut st = db.prepare(&sql)?;
        let rows = st.query_map([], |r| {
            Ok(NamedCount {
                name: r.get(0)?,
                track_count: r.get(1)?,
            })
        })?;
        Ok(rows.collect::<rusqlite::Result<_>>()?)
    }

    pub fn artists(&self) -> Result<Vec<NamedCount>> {
        self.grouped(ARTIST_EXPR)
    }

    pub fn genres(&self) -> Result<Vec<NamedCount>> {
        self.grouped("genre")
    }

    pub fn track_count(&self) -> Result<u64> {
        Ok(self
            .db()
            .query_row(&format!("SELECT COUNT(*) FROM tracks WHERE {LOCAL_ONLY}"), [], |r| r.get(0))?)
    }

    pub fn set_favorite(&self, id: i64, favorite: bool) -> Result<()> {
        self.db().execute(
            "UPDATE tracks SET favorite = ?2 WHERE id = ?1",
            params![id, favorite],
        )?;
        Ok(())
    }

    pub fn mark_played(&self, id: i64) -> Result<()> {
        self.db().execute(
            "UPDATE tracks SET play_count = play_count + 1, last_played = ?2 WHERE id = ?1",
            params![id, now()],
        )?;
        Ok(())
    }

    pub fn cover(&self, track_id: i64) -> Result<Option<(Vec<u8>, String)>> {
        let Some(t) = self.track(track_id)? else {
            return Ok(None);
        };
        if let Some(c) = scanner::read_cover(Path::new(&t.path)) {
            return Ok(Some(c));
        }
        // Aus dem Internet geladenes Cover
        let file: Option<String> = self
            .db()
            .query_row("SELECT cover_path FROM tracks WHERE id = ?1", [track_id], |r| r.get(0))
            .optional()?
            .flatten();
        let Some(file) = file else { return Ok(None) };
        if file.starts_with("http://") || file.starts_with("https://") {
            return Ok(self.fetch_online_cover(track_id, &file));
        }
        Ok(std::fs::read(file).ok().map(|d| (d, "image/jpeg".to_string())))
    }

    /// Vorschaubild eines Online-Titels einmal laden und als Datei zwischenspeichern.
    fn fetch_online_cover(&self, track_id: i64, url: &str) -> Option<(Vec<u8>, String)> {
        let resp = ureq::get(url).timeout(std::time::Duration::from_secs(10)).call().ok()?;
        let mut data = Vec::new();
        std::io::Read::read_to_end(&mut std::io::Read::take(resp.into_reader(), 4 * 1024 * 1024), &mut data).ok()?;
        if data.len() < 500 {
            return None;
        }
        let path = Self::plugin_dir("covers").join(format!("online-{track_id}.jpg"));
        if std::fs::write(&path, &data).is_ok() {
            let _ = self.db().execute(
                "UPDATE tracks SET cover_path = ?2 WHERE id = ?1",
                params![track_id, path.to_string_lossy()],
            );
        }
        Some((data, "image/jpeg".into()))
    }

    /// Online-Titel (aus Quellen-Plugins) anlegen oder aktualisieren; liefert die IDs in derselben Reihenfolge.
    pub fn upsert_online(&self, plugin: &str, album: &str, items: &[crate::sources::SourceItem]) -> Result<Vec<i64>> {
        let mut db = self.db();
        let tx = db.transaction()?;
        let mut ids = Vec::with_capacity(items.len());
        {
            let mut st = tx.prepare(
                "INSERT INTO tracks (path, title, artist, album, duration_ms, has_cover, cover_path, cover_checked, mtime, added_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 1, 0, ?8)
                 ON CONFLICT(path) DO UPDATE SET title = excluded.title, artist = excluded.artist,
                    duration_ms = MAX(excluded.duration_ms, tracks.duration_ms),
                    has_cover = MAX(tracks.has_cover, excluded.has_cover),
                    cover_path = COALESCE(tracks.cover_path, excluded.cover_path)",
            )?;
            let ts = now();
            for it in items {
                let path = crate::sources::make_path(plugin, &it.id);
                let thumb = (!it.thumbnail.is_empty()).then_some(it.thumbnail.as_str());
                st.execute(params![path, it.title, it.artist, album, it.duration_ms as i64, thumb.is_some(), thumb, ts])?;
                ids.push(tx.query_row("SELECT id FROM tracks WHERE path = ?1", [&path], |r| r.get(0))?);
            }
        }
        tx.commit()?;
        Ok(ids)
    }

    /// Unbenutzte Online-Titel (nur gesucht, nie gespielt) nach einem Tag entfernen.
    pub fn prune_online(&self) -> Result<usize> {
        let n = self.db().execute(
            "DELETE FROM tracks WHERE substr(path, 1, 9) = 'plugin://' AND play_count = 0 AND favorite = 0
             AND id NOT IN (SELECT track_id FROM playlist_tracks) AND added_at < ?1",
            [now() - 24 * 3600],
        )?;
        Ok(n)
    }

    /// Titel ohne Cover, die noch nicht online gesucht wurden.
    pub fn cover_jobs(&self) -> Result<Vec<CoverJob>> {
        let db = self.db();
        let mut st = db.prepare(
            "SELECT id, title, artist, album, album_artist FROM tracks
             WHERE has_cover = 0 AND cover_checked = 0 AND substr(path, 1, 9) != 'plugin://'
             ORDER BY album, id",
        )?;
        let rows = st.query_map([], |r| {
            Ok(CoverJob {
                id: r.get(0)?,
                title: r.get(1)?,
                artist: r.get(2)?,
                album: r.get(3)?,
                album_artist: r.get(4)?,
            })
        })?;
        Ok(rows.collect::<rusqlite::Result<_>>()?)
    }

    /// Ergebnis der Online-Suche speichern (`None` = nichts gefunden, nicht erneut suchen).
    pub fn set_cover_file(&self, id: i64, file: Option<&Path>) -> Result<()> {
        let path = file.map(|p| p.to_string_lossy().into_owned());
        self.db().execute(
            "UPDATE tracks SET cover_checked = 1, cover_path = ?2,
                has_cover = has_cover OR ?2 IS NOT NULL WHERE id = ?1",
            params![id, path],
        )?;
        Ok(())
    }

    /// Erlaubt eine neue Online-Suche für alle Titel ohne Cover.
    pub fn reset_cover_checks(&self) -> Result<()> {
        self.db().execute("UPDATE tracks SET cover_checked = 0 WHERE has_cover = 0", [])?;
        Ok(())
    }

    // ---------- Playlisten ----------

    pub fn playlists(&self) -> Result<Vec<Playlist>> {
        let db = self.db();
        let mut st = db.prepare(
            "SELECT p.id, p.name, COUNT(t.id), COALESCE(SUM(t.duration_ms), 0),
                    COALESCE(SUM(CASE WHEN substr(t.path, 1, 9) = 'plugin://' THEN 1 ELSE 0 END), 0)
             FROM playlists p
             LEFT JOIN playlist_tracks pt ON pt.playlist_id = p.id
             LEFT JOIN tracks t ON t.id = pt.track_id
             GROUP BY p.id ORDER BY p.name COLLATE NOCASE",
        )?;
        let rows = st.query_map([], |r| {
            Ok(Playlist {
                id: r.get(0)?,
                name: r.get(1)?,
                track_count: r.get(2)?,
                duration_ms: r.get::<_, i64>(3)? as u64,
                online_count: r.get(4)?,
            })
        })?;
        Ok(rows.collect::<rusqlite::Result<_>>()?)
    }

    pub fn playlist(&self, id: i64) -> Result<Option<Playlist>> {
        Ok(self.playlists()?.into_iter().find(|p| p.id == id))
    }

    pub fn create_playlist(&self, name: &str) -> Result<i64> {
        let name = name.trim();
        if name.is_empty() {
            bail!("Name der Playlist darf nicht leer sein");
        }
        let db = self.db();
        db.execute(
            "INSERT INTO playlists(name, created_at) VALUES (?1, ?2)",
            params![name, now()],
        )?;
        Ok(db.last_insert_rowid())
    }

    pub fn rename_playlist(&self, id: i64, name: &str) -> Result<()> {
        let name = name.trim();
        if name.is_empty() {
            bail!("Name der Playlist darf nicht leer sein");
        }
        self.db()
            .execute("UPDATE playlists SET name = ?2 WHERE id = ?1", params![id, name])?;
        Ok(())
    }

    pub fn delete_playlist(&self, id: i64) -> Result<()> {
        self.db().execute("DELETE FROM playlists WHERE id = ?1", [id])?;
        Ok(())
    }

    pub fn playlist_track_ids(&self, id: i64) -> Result<Vec<i64>> {
        let db = self.db();
        let mut st = db.prepare(
            "SELECT track_id FROM playlist_tracks WHERE playlist_id = ?1 ORDER BY position",
        )?;
        let rows = st.query_map([id], |r| r.get(0))?;
        Ok(rows.collect::<rusqlite::Result<_>>()?)
    }

    pub fn playlist_tracks(&self, id: i64) -> Result<Vec<Track>> {
        let db = self.db();
        let sql = format!(
            "SELECT {} FROM playlist_tracks pt JOIN tracks t ON t.id = pt.track_id
             WHERE pt.playlist_id = ?1 ORDER BY pt.position",
            TRACK_COLUMNS
                .split(", ")
                .map(|c| format!("t.{c}"))
                .collect::<Vec<_>>()
                .join(", ")
        );
        let mut st = db.prepare(&sql)?;
        let rows = st.query_map([id], track_from_row)?;
        Ok(rows.collect::<rusqlite::Result<_>>()?)
    }

    /// Schreibt die komplette Reihenfolge einer Playlist neu.
    pub fn set_playlist_tracks(&self, id: i64, track_ids: &[i64]) -> Result<()> {
        let mut db = self.db();
        let tx = db.transaction()?;
        let exists: bool = tx
            .query_row("SELECT 1 FROM playlists WHERE id = ?1", [id], |_| Ok(true))
            .optional()?
            .unwrap_or(false);
        if !exists {
            bail!("Playlist {id} gibt es nicht");
        }
        tx.execute("DELETE FROM playlist_tracks WHERE playlist_id = ?1", [id])?;
        {
            let mut st = tx.prepare(
                "INSERT INTO playlist_tracks(playlist_id, position, track_id) VALUES (?1, ?2, ?3)",
            )?;
            for (pos, tid) in track_ids.iter().enumerate() {
                st.execute(params![id, pos as i64, tid])?;
            }
        }
        tx.commit()?;
        Ok(())
    }

    pub fn add_to_playlist(&self, id: i64, track_ids: &[i64]) -> Result<()> {
        let mut ids = self.playlist_track_ids(id)?;
        ids.extend_from_slice(track_ids);
        self.set_playlist_tracks(id, &ids)
    }

    pub fn remove_from_playlist(&self, id: i64, position: usize) -> Result<()> {
        let mut ids = self.playlist_track_ids(id)?;
        if position < ids.len() {
            ids.remove(position);
        }
        self.set_playlist_tracks(id, &ids)
    }

    pub fn move_in_playlist(&self, id: i64, from: usize, to: usize) -> Result<()> {
        let mut ids = self.playlist_track_ids(id)?;
        if from < ids.len() {
            let t = ids.remove(from);
            ids.insert(to.min(ids.len()), t);
        }
        self.set_playlist_tracks(id, &ids)
    }

    /// Importiert eine M3U-Datei als neue Playlist. Dateien, die noch nicht in der
    /// Bibliothek sind, werden direkt eingelesen.
    pub fn import_m3u(&self, file: &Path) -> Result<i64> {
        let content = std::fs::read_to_string(file)
            .with_context(|| format!("{} lesen", file.display()))?;
        let base = file.parent().unwrap_or(Path::new("/"));
        let mut ids = Vec::new();
        for path in m3u::parse(&content, base) {
            let Some(p) = path.to_str() else { continue };
            if !path.is_file() {
                continue;
            }
            if self.track_by_path(p)?.is_none() {
                let info = scanner::read_track_info(&path, scanner::mtime(&path));
                self.upsert_tracks(&[info])?;
            }
            if let Some(t) = self.track_by_path(p)? {
                ids.push(t.id);
            }
        }
        let name = file
            .file_stem()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_else(|| "Importiert".into());
        let id = self.create_playlist(&name)?;
        self.set_playlist_tracks(id, &ids)?;
        Ok(id)
    }

    pub fn export_m3u(&self, id: i64, file: &Path) -> Result<()> {
        let tracks = self.playlist_tracks(id)?;
        std::fs::write(file, m3u::write(&tracks))
            .with_context(|| format!("{} schreiben", file.display()))?;
        Ok(())
    }

    // ---------- Radiosender ----------

    pub fn stations(&self) -> Result<Vec<Station>> {
        let db = self.db();
        let mut st = db.prepare(
            "SELECT id, uuid, name, url, favicon, homepage, country, tags, codec, bitrate
             FROM stations ORDER BY position, id",
        )?;
        let rows = st.query_map([], |r| {
            Ok(Station {
                id: r.get(0)?,
                uuid: r.get(1)?,
                name: r.get(2)?,
                url: r.get(3)?,
                favicon: r.get(4)?,
                homepage: r.get(5)?,
                country: r.get(6)?,
                tags: r.get(7)?,
                codec: r.get(8)?,
                bitrate: r.get(9)?,
            })
        })?;
        Ok(rows.collect::<rusqlite::Result<_>>()?)
    }

    pub fn station(&self, id: i64) -> Result<Option<Station>> {
        Ok(self.stations()?.into_iter().find(|s| s.id == id))
    }

    /// Speichert einen Sender in „Meine Sender“ (gleiche Adresse = wird aktualisiert).
    pub fn add_station(&self, s: &Station) -> Result<i64> {
        let url = s.url.trim();
        if !(url.starts_with("http://") || url.starts_with("https://")) {
            bail!("Die Adresse muss mit http:// oder https:// beginnen");
        }
        let name = if s.name.trim().is_empty() { url } else { s.name.trim() };
        let db = self.db();
        db.execute(
            "INSERT INTO stations (uuid, name, url, favicon, homepage, country, tags, codec, bitrate, position)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, (SELECT COALESCE(MAX(position), 0) + 1 FROM stations))
             ON CONFLICT(url) DO UPDATE SET name = excluded.name, favicon = excluded.favicon",
            params![s.uuid, name, url, s.favicon, s.homepage, s.country, s.tags, s.codec, s.bitrate],
        )?;
        Ok(db.query_row("SELECT id FROM stations WHERE url = ?1", [url], |r| r.get(0))?)
    }

    pub fn remove_station(&self, id: i64) -> Result<()> {
        self.db().execute("DELETE FROM stations WHERE id = ?1", [id])?;
        Ok(())
    }

    // ---------- Einstellungen ----------

    pub fn setting(&self, key: &str) -> Option<String> {
        self.db()
            .query_row("SELECT value FROM settings WHERE key = ?1", [key], |r| r.get(0))
            .optional()
            .ok()
            .flatten()
    }

    pub fn setting_as<T: serde::de::DeserializeOwned>(&self, key: &str) -> Option<T> {
        self.setting(key).and_then(|v| serde_json::from_str(&v).ok())
    }

    pub fn set_setting(&self, key: &str, value: &str) -> Result<()> {
        self.db().execute(
            "INSERT INTO settings(key, value) VALUES (?1, ?2)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            [key, value],
        )?;
        Ok(())
    }

    pub fn set_setting_json<T: Serialize>(&self, key: &str, value: &T) -> Result<()> {
        self.set_setting(key, &serde_json::to_string(value)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn info(path: &str, title: &str, artist: &str, album: &str, no: u32) -> TrackInfo {
        TrackInfo {
            path: path.into(),
            title: title.into(),
            artist: artist.into(),
            album: album.into(),
            track_no: Some(no),
            duration_ms: 1000,
            ..Default::default()
        }
    }

    fn sample() -> Library {
        let lib = Library::open_in_memory().unwrap();
        lib.upsert_tracks(&[
            info("/m/a/2.mp3", "Zwei", "Alpha", "Erstes", 2),
            info("/m/a/1.mp3", "Eins", "Alpha", "Erstes", 1),
            info("/m/b/1.mp3", "Morgen", "Beta", "Zweites", 1),
        ])
        .unwrap();
        lib
    }

    #[test]
    fn query_sort_and_search() {
        let lib = sample();
        let all = lib.tracks(&TrackQuery::default()).unwrap();
        let titles: Vec<_> = all.iter().map(|t| t.title.as_str()).collect();
        assert_eq!(titles, ["Eins", "Zwei", "Morgen"]);

        let found = lib
            .tracks(&TrackQuery {
                search: Some("beta morg".into()),
                ..Default::default()
            })
            .unwrap();
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].title, "Morgen");

        assert_eq!(lib.albums().unwrap().len(), 2);
        assert_eq!(lib.artists().unwrap()[0].track_count, 2);
    }

    #[test]
    fn upsert_keeps_id_and_play_count() {
        let lib = sample();
        let t = lib.track_by_path("/m/b/1.mp3").unwrap().unwrap();
        lib.mark_played(t.id).unwrap();
        lib.upsert_tracks(&[info("/m/b/1.mp3", "Neu", "Beta", "Zweites", 1)])
            .unwrap();
        let t2 = lib.track(t.id).unwrap().unwrap();
        assert_eq!(t2.title, "Neu");
        assert_eq!(t2.play_count, 1);
    }

    #[test]
    fn playlist_crud_and_order() {
        let lib = sample();
        let ids: Vec<i64> = lib
            .tracks(&TrackQuery::default())
            .unwrap()
            .iter()
            .map(|t| t.id)
            .collect();
        let pl = lib.create_playlist("Mix").unwrap();
        lib.add_to_playlist(pl, &ids).unwrap();
        lib.move_in_playlist(pl, 2, 0).unwrap();
        assert_eq!(lib.playlist_track_ids(pl).unwrap(), [ids[2], ids[0], ids[1]]);
        lib.remove_from_playlist(pl, 1).unwrap();
        assert_eq!(lib.playlist(pl).unwrap().unwrap().track_count, 2);
        lib.rename_playlist(pl, "Neu").unwrap();
        assert_eq!(lib.playlists().unwrap()[0].name, "Neu");
        lib.delete_playlist(pl).unwrap();
        assert!(lib.playlists().unwrap().is_empty());
    }

    #[test]
    fn remove_folder_drops_its_tracks() {
        let lib = sample();
        lib.remove_folder("/m/a").unwrap();
        assert_eq!(lib.track_count().unwrap(), 1);
    }

    #[test]
    fn online_tracks_only_visible_when_used() {
        let lib = sample();
        let item = crate::sources::SourceItem { id: "abc".into(), title: "Weh Mir".into(), artist: "Santiano".into(), duration_ms: 1000, thumbnail: String::new() };
        let ids = lib.upsert_online("meine-quelle", "Meine Quelle", &[item.clone()]).unwrap();
        // gleiches Video erneut → gleiche ID
        assert_eq!(lib.upsert_online("meine-quelle", "Meine Quelle", &[item]).unwrap(), ids);
        let all = |lib: &Library| lib.tracks(&TrackQuery::default()).unwrap().len();
        assert_eq!(all(&lib), 3);
        assert_eq!(lib.track_count().unwrap(), 3);
        lib.mark_played(ids[0]).unwrap();
        assert_eq!(all(&lib), 4);
        assert_eq!(lib.track_count().unwrap(), 3, "zählt nur eigene Dateien");
        assert!(lib.albums().unwrap().iter().all(|a| a.name != "Meine Quelle"));
        assert_eq!(lib.track(ids[0]).unwrap().unwrap().path, "plugin://meine-quelle/abc");
    }

    #[test]
    fn settings_roundtrip() {
        let lib = sample();
        lib.set_setting_json("volume", &42u8).unwrap();
        assert_eq!(lib.setting_as::<u8>("volume"), Some(42));
    }
}
