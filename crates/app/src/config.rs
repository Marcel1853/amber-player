use std::net::{IpAddr, Ipv4Addr, SocketAddr};

/// Standard-Port der HTTP-API (wie `amber_core::api::DEFAULT_PORT`).
pub const DEFAULT_PORT: u16 = 7878;
use amber_core::Library;
use rand::distributions::Alphanumeric;
use rand::Rng;
use serde::{Deserialize, Serialize};

/// Einstellungen der HTTP-API, gespeichert in der Bibliothek.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ApiConfig {
    /// `true` = im Heimnetz erreichbar (dann nur mit Token)
    pub lan: bool,
    pub port: u16,
    pub token: String,
}

impl Default for ApiConfig {
    fn default() -> Self {
        Self {
            lan: false,
            port: DEFAULT_PORT,
            token: String::new(),
        }
    }
}

impl ApiConfig {
    const KEY: &'static str = "api";

    pub fn load(lib: &Library) -> Self {
        lib.setting_as(Self::KEY).unwrap_or_default()
    }

    pub fn save(&self, lib: &Library) {
        if let Err(e) = lib.set_setting_json(Self::KEY, self) {
            log::warn!("API-Einstellungen speichern: {e:#}");
        }
    }

    pub fn ensure_token(&mut self) {
        if self.token.is_empty() {
            self.regenerate_token();
        }
    }

    pub fn regenerate_token(&mut self) {
        self.token = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(24)
            .map(char::from)
            .collect();
    }

    pub fn bind_addr(&self) -> SocketAddr {
        let ip = if self.lan {
            IpAddr::V4(Ipv4Addr::UNSPECIFIED)
        } else {
            IpAddr::V4(Ipv4Addr::LOCALHOST)
        };
        SocketAddr::new(ip, self.port)
    }

    /// Nur im Heimnetz-Modus wird das Token verlangt.
    pub fn required_token(&self) -> Option<String> {
        self.lan.then(|| self.token.clone())
    }

    pub fn url(&self) -> String {
        format!("http://127.0.0.1:{}", self.port)
    }

    pub fn mcp_url(&self) -> String {
        format!("{}/mcp", self.url())
    }
}

/// Adresse dieses PCs im Heimnetz (z. B. 192.168.178.35). Es wird nichts gesendet;
/// der Verbindungsaufbau eines UDP-Sockets verrät nur, welche Netzwerkkarte genutzt würde.
pub fn lan_ip() -> Option<String> {
    let socket = std::net::UdpSocket::bind("0.0.0.0:0").ok()?;
    socket.connect("192.0.2.1:9").ok()?;
    let ip = socket.local_addr().ok()?.ip();
    (!ip.is_loopback() && !ip.is_unspecified()).then(|| ip.to_string())
}
