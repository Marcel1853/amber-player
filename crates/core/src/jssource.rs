//! Online-Quellen als JavaScript-Plugin („erweiterte Plugins“).
//!
//! Jede Quelle läuft in einer eigenen, kleinen JavaScript-Umgebung (QuickJS) in einem eigenen
//! Thread – am PC und am Handy gleich, auch wenn der Bildschirm aus ist. Das Plugin sieht nur das
//! Objekt `amber`:
//!
//! ```js
//! amber.source({
//!   id: 'meine-quelle', name: 'Meine Quelle', description: '…', icon: '♪',
//!   search(query, limit) { return [{ id, title, artist, duration_ms, thumbnail }]; },
//!   playlist(url) { return { name, items: [ … ] }; },     // optional
//!   resolve(id) { return { url, headers: { … } }; },
//! });
//! const r = amber.http.get('https://example.org/api?q=x');          // { status, ok, body }
//! const j = amber.http.post('https://example.org/api', { a: 1 });   // JSON-Body
//! amber.log('Hallo');
//! ```
//!
//! Ins Netz geht es nur über `amber.http`: ausschließlich https, nur zu den Adressen aus
//! `permissions.network` in der plugin.json und ohne Weiterleitungen (sonst könnte ein erlaubter
//! Server auf eine verbotene Adresse umleiten). Kein Zugriff auf Dateien, Programme oder den Player.

use std::sync::atomic::{AtomicI64, Ordering};
use std::sync::mpsc::{channel, Sender};
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use anyhow::{anyhow, bail, Result};
use rquickjs::{CatchResultExt, Context, Function, Runtime};
use serde_json::{json, Value};

use crate::sources::SourceInfo;

const MEMORY_LIMIT: usize = 64 * 1024 * 1024;
/// Rechenzeit je Aufruf (Netzanfragen zählen nicht mit).
const CALL_LIMIT_MS: i64 = 10_000;
const HTTP_TIMEOUT: Duration = Duration::from_secs(20);
const HTTP_MAX_BODY: u64 = 8 * 1024 * 1024;
const HTTP_MAX_PER_CALL: u32 = 40;

const PRELUDE: &str = r#"
'use strict';
globalThis.amber = Object.freeze({
  source(def) {
    if (!def || typeof def.id !== 'string' || typeof def.resolve !== 'function') {
      throw new Error('amber.source() braucht mindestens „id“ und „resolve“');
    }
    globalThis.__source = def;
  },
  http: Object.freeze({
    request(o) {
      const r = JSON.parse(__http(JSON.stringify(o || {})));
      return r;
    },
    get(url, headers) {
      return this.request({ method: 'GET', url, headers });
    },
    post(url, body, headers) {
      const json = typeof body !== 'string';
      return this.request({ method: 'POST', url, headers, body: json ? JSON.stringify(body) : body, isJson: json });
    },
  }),
  log(...args) {
    __log(args.map((a) => (typeof a === 'string' ? a : JSON.stringify(a))).join(' '));
  },
});
globalThis.__call = function (name, argsJson) {
  const s = globalThis.__source;
  if (!s || typeof s[name] !== 'function') throw new Error('Das Plugin kann „' + name + '“ nicht');
  const result = s[name](...JSON.parse(argsJson));
  return JSON.stringify(result === undefined ? null : result);
};
globalThis.__info = function () {
  const s = globalThis.__source;
  if (!s) throw new Error('Das Plugin hat sich nicht mit amber.source() angemeldet');
  return JSON.stringify({
    id: s.id, name: String(s.name || s.id), description: String(s.description || ''), icon: String(s.icon || '♪'),
    features: ['search', 'playlist'].filter((f) => typeof s[f] === 'function'),
  });
};
"#;

struct Job {
    func: &'static str,
    args: String,
    reply: Sender<Result<String>>,
}

/// Eine laufende JavaScript-Quelle.
pub struct JsSource {
    pub info: SourceInfo,
    tx: Mutex<Sender<Job>>,
}

fn now_ms() -> i64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_millis() as i64).unwrap_or(0)
}

/// Darf diese Adresse erreicht werden? `example.org` erlaubt genau diese Adresse,
/// `*.example.org` zusätzlich alle Unteradressen.
pub fn host_allowed(patterns: &[String], url: &str) -> bool {
    let Some(rest) = url.strip_prefix("https://") else { return false };
    let host_port = rest.split(['/', '?', '#']).next().unwrap_or("");
    if host_port.contains('@') {
        return false; // keine Zugangsdaten in der Adresse
    }
    let host = host_port.split(':').next().unwrap_or("").to_ascii_lowercase();
    if host.is_empty() {
        return false;
    }
    patterns.iter().any(|p| {
        let p = p.to_ascii_lowercase();
        match p.strip_prefix("*.") {
            Some(base) => host == base || host.ends_with(&format!(".{base}")),
            None => host == p,
        }
    })
}

fn js_err(ctx: &rquickjs::Ctx<'_>, msg: impl AsRef<str>) -> rquickjs::Error {
    rquickjs::Exception::throw_message(ctx, msg.as_ref())
}

/// Eine Netzanfrage für das Plugin (blockierend im Thread der Quelle).
fn http_request(hosts: &[String], req: &Value) -> Result<Value> {
    let url = req["url"].as_str().unwrap_or("");
    if !host_allowed(hosts, url) {
        bail!("Keine Erlaubnis für {url} – die Adresse fehlt in „permissions.network“ (nur https)");
    }
    let method = req["method"].as_str().unwrap_or("GET").to_ascii_uppercase();
    let agent = ureq::AgentBuilder::new().redirects(0).timeout(HTTP_TIMEOUT).build();
    let mut r = agent.request(&method, url);
    for (k, v) in req["headers"].as_object().into_iter().flatten() {
        let v = v.as_str().map(str::to_string).unwrap_or_else(|| v.to_string());
        r = r.set(k, &v);
    }
    if req["isJson"] == true {
        r = r.set("Content-Type", "application/json");
    }
    let resp = match req["body"].as_str() {
        Some(body) => r.send_string(body),
        None => r.call(),
    };
    let resp = match resp {
        Ok(resp) => resp,
        Err(ureq::Error::Status(_, resp)) => resp,
        Err(e) => bail!("{e}"),
    };
    let status = resp.status();
    let mut headers = serde_json::Map::new();
    for name in resp.headers_names() {
        if let Some(v) = resp.header(&name) {
            headers.insert(name.to_ascii_lowercase(), json!(v));
        }
    }
    let mut body = String::new();
    std::io::Read::read_to_string(&mut std::io::Read::take(resp.into_reader(), HTTP_MAX_BODY), &mut body)?;
    Ok(json!({ "status": status, "ok": (200..300).contains(&status), "headers": headers, "body": body }))
}

impl JsSource {
    /// Plugin starten: Code laden und auf die Anmeldung mit `amber.source()` warten.
    /// `label` steht in Meldungen (z. B. „paket/quelle.js“), `hosts` sind die erlaubten Adressen.
    pub fn start(label: &str, code: String, hosts: Vec<String>) -> Result<Self> {
        let (tx, rx) = channel::<Job>();
        let (init_tx, init_rx) = channel::<Result<SourceInfo>>();
        let name = label.to_string();
        std::thread::Builder::new().name(format!("quelle-{}", label.split('/').next().unwrap_or("js"))).spawn(move || {
            let report = |e: String| {
                let _ = init_tx.send(Err(anyhow!("{name}: {e}")));
            };
            let Ok(runtime) = Runtime::new() else { return report("JavaScript-Engine lässt sich nicht starten".into()) };
            runtime.set_memory_limit(MEMORY_LIMIT);
            runtime.set_max_stack_size(512 * 1024);
            // Endlosschleifen abbrechen: jeder Aufruf hat begrenzte Rechenzeit.
            let deadline = Arc::new(AtomicI64::new(i64::MAX));
            let d = deadline.clone();
            runtime.set_interrupt_handler(Some(Box::new(move || now_ms() > d.load(Ordering::Relaxed))));
            let Ok(context) = Context::full(&runtime) else { return report("JavaScript-Engine lässt sich nicht starten".into()) };
            let requests = Arc::new(std::sync::atomic::AtomicU32::new(0));

            let start_call = |deadline: &AtomicI64| deadline.store(now_ms() + CALL_LIMIT_MS, Ordering::Relaxed);
            start_call(&deadline);
            let loaded: Result<SourceInfo, String> = context.with(|ctx| {
                let g = ctx.globals();
                let plugin = name.clone();
                g.set(
                    "__log",
                    Function::new(ctx.clone(), move |msg: String| log::info!("Quelle {plugin}: {}", msg.chars().take(500).collect::<String>()))
                        .map_err(|e| e.to_string())?,
                )
                .map_err(|e| e.to_string())?;
                let hosts = hosts.clone();
                let count = requests.clone();
                let d2 = deadline.clone();
                g.set(
                    "__http",
                    Function::new(ctx.clone(), move |ctx: rquickjs::Ctx<'_>, req: String| -> rquickjs::Result<String> {
                        if count.fetch_add(1, Ordering::Relaxed) >= HTTP_MAX_PER_CALL {
                            return Err(js_err(&ctx, "Zu viele Netzanfragen in einem Aufruf"));
                        }
                        let req: Value = serde_json::from_str(&req).unwrap_or_default();
                        // Wartezeit im Netz zählt nicht zur Rechenzeit.
                        let left = d2.load(Ordering::Relaxed) - now_ms();
                        let result = http_request(&hosts, &req);
                        d2.store(now_ms() + left.max(1000), Ordering::Relaxed);
                        result.map(|v| v.to_string()).map_err(|e| js_err(&ctx, format!("HTTP: {e:#}")))
                    })
                    .map_err(|e| e.to_string())?,
                )
                .map_err(|e| e.to_string())?;
                ctx.eval::<(), _>(PRELUDE).catch(&ctx).map_err(|e| e.to_string())?;
                ctx.eval::<(), _>(code.as_str()).catch(&ctx).map_err(|e| format!("Fehler im Plugin: {e}"))?;
                let info: Function = g.get("__info").map_err(|e| e.to_string())?;
                let text: String = info.call(()).catch(&ctx).map_err(|e| e.to_string())?;
                let info: SourceInfo = serde_json::from_str(&text).map_err(|e| e.to_string())?;
                if info.id.is_empty() || info.id.len() > 32 || !info.id.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_') {
                    return Err("„id“ in amber.source(): 1–32 Zeichen aus Buchstaben, Ziffern, - und _".into());
                }
                Ok(info)
            });
            deadline.store(i64::MAX, Ordering::Relaxed);
            match loaded {
                Ok(info) => {
                    let _ = init_tx.send(Ok(info));
                }
                Err(e) => return report(e),
            }

            for job in rx {
                requests.store(0, Ordering::Relaxed);
                start_call(&deadline);
                let result: Result<String> = context.with(|ctx| {
                    let call: Function = ctx.globals().get("__call").map_err(|e| anyhow!("{e}"))?;
                    call.call::<_, String>((job.func, job.args.as_str())).catch(&ctx).map_err(|e| anyhow!("{e}"))
                });
                deadline.store(i64::MAX, Ordering::Relaxed);
                let _ = job.reply.send(result);
            }
        })?;
        let info = init_rx.recv_timeout(Duration::from_secs(30)).map_err(|_| anyhow!("{label}: Plugin startet nicht"))??;
        Ok(Self { info, tx: Mutex::new(tx) })
    }

    /// Funktion des Plugins aufrufen; Argumente und Antwort als JSON.
    pub fn call(&self, func: &'static str, args: Value) -> Result<Value> {
        let (reply, rx) = channel();
        self.tx
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .send(Job { func, args: args.to_string(), reply })
            .map_err(|_| anyhow!("Plugin {} läuft nicht mehr", self.info.name))?;
        let text = rx
            .recv_timeout(Duration::from_secs(120))
            .map_err(|_| anyhow!("Plugin {} antwortet nicht", self.info.name))??;
        Ok(serde_json::from_str(&text)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const PLUGIN: &str = r#"
amber.source({
  id: 'probe', name: 'Probe', description: 'Test',
  search(q, limit) { return [{ id: 'a1', title: 'Titel ' + q, artist: 'Wer', duration_ms: 1000 }].slice(0, limit); },
  resolve(id) { return { url: 'https://example.org/' + id + '.mp3', headers: { 'X-Test': '1' } }; },
  boese() { return amber.http.get('https://verboten.example.com/'); },
  ewig() { for (;;) {} },
});
"#;

    #[test]
    fn hosts() {
        let p = vec!["example.org".to_string(), "*.cdn-example.net".to_string()];
        assert!(host_allowed(&p, "https://example.org/api"));
        assert!(!host_allowed(&p, "https://www.example.org/api"));
        assert!(host_allowed(&p, "https://a1.cdn-example.net/stream?x=1"));
        assert!(host_allowed(&p, "https://cdn-example.net/"));
        assert!(!host_allowed(&p, "http://example.org/"), "nur https");
        assert!(!host_allowed(&p, "https://example.org.boese.de/"));
        assert!(!host_allowed(&p, "https://user@example.org/"));
        assert!(!host_allowed(&p, "https://notcdn-example.net/"));
    }

    #[test]
    fn plugin_calls_and_limits() {
        let s = JsSource::start("probe/source.js", PLUGIN.to_string(), vec!["example.org".into()]).unwrap();
        assert_eq!(s.info.id, "probe");
        assert_eq!(s.info.features, vec!["search".to_string()]);
        let r = s.call("search", json!(["Lied", 5])).unwrap();
        assert_eq!(r[0]["title"], "Titel Lied");
        let r = s.call("resolve", json!(["a1"])).unwrap();
        assert_eq!(r["url"], "https://example.org/a1.mp3");
        let e = s.call("boese", json!([])).unwrap_err().to_string();
        assert!(e.contains("Keine Erlaubnis"), "{e}");
        let e = s.call("ewig", json!([])).unwrap_err().to_string();
        assert!(e.to_lowercase().contains("interrupt"), "{e}");
        // nach dem Abbruch läuft das Plugin weiter
        assert!(s.call("search", json!(["x", 1])).is_ok());
    }

    #[test]
    fn broken_plugins() {
        assert!(JsSource::start("x", "this is not js(".into(), vec![]).is_err());
        assert!(JsSource::start("x", "1 + 1".into(), vec![]).is_err(), "ohne amber.source");
        assert!(JsSource::start("x", "amber.source({ id: 'a b', resolve() {} })".into(), vec![]).is_err(), "ungültige ID");
    }
}
