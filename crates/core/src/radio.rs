//! Internetradio: Sendersuche über radio-browser.info (freie Senderdatenbank).

use std::time::Duration;

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;

const USER_AGENT: &str = concat!("AmberPlayer/", env!("CARGO_PKG_VERSION"));
/// Server der Senderdatenbank – fällt einer aus, wird der nächste genommen.
const SERVERS: &[&str] = &[
    "https://all.api.radio-browser.info",
    "https://de1.api.radio-browser.info",
    "https://de2.api.radio-browser.info",
];
/// Bei Überlastung (503) kurz warten und erneut versuchen.
const ROUNDS: usize = 3;

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
#[serde(default)]
pub struct Station {
    /// ID in der eigenen Senderliste (0 = nicht gespeichert)
    pub id: i64,
    /// ID bei radio-browser.info (leer bei eigenen Adressen)
    pub uuid: String,
    pub name: String,
    pub url: String,
    pub favicon: String,
    pub homepage: String,
    pub country: String,
    pub tags: String,
    pub codec: String,
    pub bitrate: u32,
}

impl Station {
    fn from_api(v: &Value) -> Option<Self> {
        let s = |k: &str| v[k].as_str().unwrap_or("").trim().to_string();
        let url = Some(s("url_resolved")).filter(|u| !u.is_empty()).unwrap_or_else(|| s("url"));
        if url.is_empty() || s("name").is_empty() {
            return None;
        }
        Some(Self {
            id: 0,
            uuid: s("stationuuid"),
            name: s("name"),
            url,
            favicon: s("favicon"),
            homepage: s("homepage"),
            country: s("countrycode"),
            tags: s("tags").split(',').filter(|t| !t.is_empty()).take(4).collect::<Vec<_>>().join(", "),
            codec: s("codec"),
            bitrate: v["bitrate"].as_u64().unwrap_or(0) as u32,
        })
    }
}

fn get(path: &str, query: &[(&str, &str)]) -> Result<Value> {
    for round in 0..ROUNDS {
        if round > 0 {
            std::thread::sleep(Duration::from_millis(800 * round as u64));
        }
        for server in SERVERS {
            let mut req = ureq::get(&format!("{server}{path}"))
                .set("User-Agent", USER_AGENT)
                .timeout(Duration::from_secs(8));
            for (k, v) in query {
                req = req.query(k, v);
            }
            match req.call() {
                Ok(r) => return Ok(r.into_json()?),
                Err(e) => log::debug!("Senderdatenbank {server}: {e}"),
            }
        }
    }
    Err(anyhow!(
        "Die Senderdatenbank (radio-browser.info) ist gerade nicht erreichbar – bitte später nochmal versuchen. \
         „Meine Sender“ und eigene Adressen funktionieren trotzdem."
    ))
}

/// Sender suchen. Leerer Suchbegriff = beliebteste Sender (im Land).
/// `country`: zweistelliger Ländercode wie „DE“ oder leer für alle.
pub fn search(query: &str, country: &str, limit: u32) -> Result<Vec<Station>> {
    let limit = limit.clamp(1, 200).to_string();
    let mut params = vec![
        ("hidebroken", "true"),
        ("order", "clickcount"),
        ("reverse", "true"),
        ("limit", limit.as_str()),
    ];
    let query = query.trim();
    if !query.is_empty() {
        params.push(("name", query));
    }
    if !country.is_empty() {
        params.push(("countrycode", country));
    }
    let v = get("/json/stations/search", &params)?;
    let mut out: Vec<Station> = v.as_array().into_iter().flatten().filter_map(Station::from_api).collect();
    // Gleiche Sender tauchen oft mehrfach auf – nach Adresse aussortieren.
    let mut seen = std::collections::HashSet::new();
    out.retain(|s| seen.insert(s.url.clone()));
    Ok(out)
}

/// Meldet der Senderdatenbank, dass ein Sender gehört wird (hilft bei der Beliebtheit).
pub fn count_click(uuid: &str) {
    if uuid.is_empty() {
        return;
    }
    let path = format!("/json/url/{uuid}");
    std::thread::spawn(move || {
        let _ = get(&path, &[]);
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn parses_api_station() {
        let v = json!({
            "stationuuid": "abc", "name": " Radio Test ", "url": "http://x/stream.m3u",
            "url_resolved": "http://x/stream", "favicon": "", "countrycode": "DE",
            "tags": "rock,pop,,80s,90s,oldies", "codec": "MP3", "bitrate": 128
        });
        let s = Station::from_api(&v).unwrap();
        assert_eq!(s.name, "Radio Test");
        assert_eq!(s.url, "http://x/stream");
        assert_eq!(s.tags, "rock, pop, 80s, 90s");
        assert!(Station::from_api(&json!({ "name": "ohne Adresse" })).is_none());
    }
}
