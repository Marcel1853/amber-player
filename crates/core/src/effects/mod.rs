//! Plugin-System für Klangeffekte.
//!
//! Ein [`EffectPlugin`] beschreibt einen Effekt (Name, Parameter, Presets) und erzeugt für
//! jeden Titel einen [`Processor`], der das Audio Frame für Frame bearbeitet. Der
//! [`PluginHost`] merkt sich, welche Plugins aktiv sind und welche Werte sie haben;
//! [`EffectChain`] hängt die aktiven Plugins als rodio-`Source` in den Audiostrom.

mod crossfeed;
pub mod custom;
mod equalizer;
mod limiter;
mod reverb;
mod stereo;
mod tone;

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, RwLock};
use std::time::Duration;

use anyhow::{anyhow, bail, Result};
use rodio::source::SeekError;
use rodio::Source;
use serde::{Deserialize, Serialize};

use crate::analyzer::Analyzer;
use crate::library::Library;

pub use crossfeed::Crossfeed;
pub use equalizer::Equalizer;
pub use limiter::Limiter;
pub use reverb::Reverb;
pub use stereo::StereoWidth;
pub use tone::Tone;

#[derive(Debug, Clone, Serialize)]
pub struct ParamInfo {
    pub id: &'static str,
    pub name: &'static str,
    pub min: f32,
    pub max: f32,
    pub default: f32,
    pub step: f32,
    pub unit: &'static str,
}

#[derive(Debug, Clone, Serialize)]
pub struct Preset {
    pub name: &'static str,
    pub values: Vec<f32>,
}

/// Beschreibung eines Effekts. Implementierungen sind zustandslos; der Klangzustand
/// (Filter usw.) steckt im [`Processor`].
pub trait EffectPlugin: Send + Sync {
    fn id(&self) -> &'static str;
    fn name(&self) -> &'static str;
    fn description(&self) -> &'static str;
    fn params(&self) -> Vec<ParamInfo>;
    fn presets(&self) -> Vec<Preset> {
        Vec::new()
    }
    /// Dateiname bei Plugins aus dem Plugin-Ordner (`None` = mitgeliefert).
    fn source(&self) -> Option<String> {
        None
    }
    fn create(&self, sample_rate: u32, channels: u16) -> Box<dyn Processor>;
}

pub trait Processor: Send {
    /// Neue Parameterwerte (gleiche Reihenfolge wie [`EffectPlugin::params`]).
    fn set_params(&mut self, values: &[f32]);
    /// Bearbeitet ein Frame: ein Sample pro Kanal, verschränkt.
    fn process(&mut self, frame: &mut [f32]);
    /// Interne Zustände löschen (nach dem Springen im Titel).
    fn reset(&mut self);
}

/// Alle eingebauten Plugins in Verarbeitungsreihenfolge.
pub fn builtin_plugins() -> Vec<Arc<dyn EffectPlugin>> {
    let native: Vec<Arc<dyn EffectPlugin>> = vec![
        Arc::new(Equalizer),
        Arc::new(Tone),
        Arc::new(Crossfeed),
        Arc::new(StereoWidth),
        Arc::new(Reverb),
    ];
    native
        .into_iter()
    .chain(custom::builtin_json_plugins().into_iter().map(|p| Arc::new(p) as Arc<dyn EffectPlugin>))
    // Der Limiter kommt zuletzt, damit er Spitzen aller anderen Effekte abfängt.
    .chain(std::iter::once(Arc::new(Limiter) as Arc<dyn EffectPlugin>))
    .collect()
}

#[derive(Debug, Clone, Serialize)]
pub struct EffectInfo {
    pub id: &'static str,
    pub name: &'static str,
    pub description: &'static str,
    pub enabled: bool,
    pub values: Vec<f32>,
    pub params: Vec<ParamInfo>,
    pub presets: Vec<Preset>,
    /// Selbst gespeicherte Voreinstellungen
    pub user_presets: Vec<UserPreset>,
    /// Dateiname, wenn das Plugin aus dem Plugin-Ordner stammt
    pub source: Option<String>,
}

/// Eine selbst gespeicherte Voreinstellung.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPreset {
    pub name: String,
    pub values: Vec<f32>,
}

const MAX_USER_PRESETS: usize = 50;

#[derive(Serialize, Deserialize)]
struct SavedEffect {
    enabled: bool,
    values: Vec<f32>,
}

struct Slot {
    plugin: Arc<dyn EffectPlugin>,
    params: Vec<ParamInfo>,
    enabled: AtomicBool,
    values: RwLock<Vec<f32>>,
    user_presets: RwLock<Vec<UserPreset>>,
}

pub struct PluginHost {
    slots: Vec<Slot>,
    version: AtomicU64,
    library: Option<Arc<Library>>,
    load_errors: Vec<String>,
}

impl PluginHost {
    /// Host mit den eingebauten Plugins; gespeicherte Einstellungen werden geladen.
    pub fn with_builtin(library: Option<Arc<Library>>) -> Arc<Self> {
        Self::new(builtin_plugins(), library)
    }

    /// Mitgelieferte Plugins plus alle `*.json`-Plugins aus `dir`. Fehlerhafte Dateien und
    /// doppelte IDs werden übersprungen und in [`PluginHost::load_errors`] gemeldet.
    pub fn load(dir: Option<&std::path::Path>, library: Option<Arc<Library>>) -> Arc<Self> {
        let mut plugins = builtin_plugins();
        let mut errors = Vec::new();
        if let Some(dir) = dir {
            let (found, errs) = custom::load_dir(dir);
            errors = errs;
            for p in found {
                if plugins.iter().any(|x| x.id() == p.id()) {
                    errors.push(format!("{}: Die ID „{}“ gibt es schon", p.source.clone().unwrap_or_default(), p.id()));
                    continue;
                }
                // Eigene Plugins vor dem Limiter einreihen.
                let at = plugins.len() - 1;
                plugins.insert(at, Arc::new(p));
            }
        }
        for e in &errors {
            log::warn!("Klang-Plugin: {e}");
        }
        let host = Self::new(plugins, library);
        // `new` gibt ein frisches Arc zurück, das noch niemand teilt.
        let mut host = Arc::try_unwrap(host).ok().expect("neuer Host");
        host.load_errors = errors;
        Arc::new(host)
    }

    pub fn load_errors(&self) -> &[String] {
        &self.load_errors
    }

    pub fn new(plugins: Vec<Arc<dyn EffectPlugin>>, library: Option<Arc<Library>>) -> Arc<Self> {
        let slots = plugins
            .into_iter()
            .map(|plugin| {
                let params = plugin.params();
                let mut values: Vec<f32> = params.iter().map(|p| p.default).collect();
                let mut enabled = false;
                let saved = library
                    .as_ref()
                    .and_then(|l| l.setting_as::<SavedEffect>(&setting_key(plugin.id())));
                if let Some(s) = saved {
                    enabled = s.enabled;
                    for ((v, p), s) in values.iter_mut().zip(&params).zip(s.values) {
                        *v = s.clamp(p.min, p.max);
                    }
                }
                let user_presets = library
                    .as_ref()
                    .and_then(|l| l.setting_as::<Vec<UserPreset>>(&presets_key(plugin.id())))
                    .unwrap_or_default();
                Slot {
                    plugin,
                    params,
                    enabled: AtomicBool::new(enabled),
                    values: RwLock::new(values),
                    user_presets: RwLock::new(user_presets),
                }
            })
            .collect();
        Arc::new(Self {
            slots,
            version: AtomicU64::new(1),
            library,
            load_errors: Vec::new(),
        })
    }

    fn slot(&self, id: &str) -> Result<&Slot> {
        self.slots
            .iter()
            .find(|s| s.plugin.id() == id)
            .ok_or_else(|| anyhow!("Unbekanntes Effekt-Plugin: {id}"))
    }

    fn info(slot: &Slot) -> EffectInfo {
        EffectInfo {
            id: slot.plugin.id(),
            name: slot.plugin.name(),
            description: slot.plugin.description(),
            enabled: slot.enabled.load(Ordering::Relaxed),
            values: slot.values.read().unwrap().clone(),
            params: slot.params.clone(),
            presets: slot.plugin.presets(),
            user_presets: slot.user_presets.read().unwrap().clone(),
            source: slot.plugin.source(),
        }
    }

    pub fn list(&self) -> Vec<EffectInfo> {
        self.slots.iter().map(Self::info).collect()
    }

    pub fn get(&self, id: &str) -> Result<EffectInfo> {
        Ok(Self::info(self.slot(id)?))
    }

    /// Schaltet ein Plugin an/aus und/oder setzt Werte. Werte werden auf den
    /// erlaubten Bereich begrenzt; fehlende Werte bleiben unverändert.
    pub fn set(&self, id: &str, enabled: Option<bool>, values: Option<&[f32]>) -> Result<EffectInfo> {
        let slot = self.slot(id)?;
        if let Some(on) = enabled {
            slot.enabled.store(on, Ordering::Relaxed);
        }
        if let Some(new) = values {
            let mut vals = slot.values.write().unwrap();
            for ((v, p), n) in vals.iter_mut().zip(&slot.params).zip(new) {
                if n.is_finite() {
                    *v = n.clamp(p.min, p.max);
                }
            }
        }
        self.version.fetch_add(1, Ordering::Release);
        let info = Self::info(slot);
        if let Some(lib) = &self.library {
            let saved = SavedEffect {
                enabled: info.enabled,
                values: info.values.clone(),
            };
            let _ = lib.set_setting_json(&setting_key(id), &saved);
        }
        Ok(info)
    }

    /// Setzt einen einzelnen Parameter über seine ID (z. B. `"32"` beim Equalizer).
    pub fn set_param(&self, id: &str, param: &str, value: f32) -> Result<EffectInfo> {
        let slot = self.slot(id)?;
        let idx = slot
            .params
            .iter()
            .position(|p| p.id == param)
            .ok_or_else(|| anyhow!("Plugin {id} hat keinen Parameter {param}"))?;
        let mut vals = slot.values.read().unwrap().clone();
        vals[idx] = value;
        self.set(id, None, Some(&vals))
    }

    pub fn apply_preset(&self, id: &str, preset: &str) -> Result<EffectInfo> {
        let slot = self.slot(id)?;
        let own = slot.user_presets.read().unwrap().iter().find(|p| p.name == preset).map(|p| p.values.clone());
        if let Some(values) = own {
            return self.set(id, Some(true), Some(&values));
        }
        let Some(p) = slot
            .plugin
            .presets()
            .into_iter()
            .find(|p| p.name.eq_ignore_ascii_case(preset))
        else {
            bail!("Plugin {id} hat kein Preset „{preset}“");
        };
        self.set(id, Some(true), Some(&p.values))
    }

    /// Speichert die aktuellen Werte als eigene Voreinstellung (gleicher Name wird ersetzt).
    pub fn save_preset(&self, id: &str, name: &str) -> Result<EffectInfo> {
        let slot = self.slot(id)?;
        let name = name.trim();
        if name.is_empty() || name.chars().count() > 40 {
            bail!("Der Name muss 1 bis 40 Zeichen lang sein");
        }
        let values = slot.values.read().unwrap().clone();
        {
            let mut list = slot.user_presets.write().unwrap();
            if let Some(p) = list.iter_mut().find(|p| p.name == name) {
                p.values = values;
            } else {
                if list.len() >= MAX_USER_PRESETS {
                    bail!("Höchstens {MAX_USER_PRESETS} eigene Voreinstellungen je Effekt");
                }
                list.push(UserPreset { name: name.to_string(), values });
            }
        }
        self.store_presets(id, slot);
        Ok(Self::info(slot))
    }

    pub fn delete_preset(&self, id: &str, name: &str) -> Result<EffectInfo> {
        let slot = self.slot(id)?;
        slot.user_presets.write().unwrap().retain(|p| p.name != name);
        self.store_presets(id, slot);
        Ok(Self::info(slot))
    }

    /// Setzt alle Werte auf die Standardwerte zurück (an/aus bleibt).
    pub fn reset(&self, id: &str) -> Result<EffectInfo> {
        let defaults: Vec<f32> = self.slot(id)?.params.iter().map(|p| p.default).collect();
        self.set(id, None, Some(&defaults))
    }

    fn store_presets(&self, id: &str, slot: &Slot) {
        if let Some(lib) = &self.library {
            let _ = lib.set_setting_json(&presets_key(id), &*slot.user_presets.read().unwrap());
        }
    }

    fn version(&self) -> u64 {
        self.version.load(Ordering::Acquire)
    }

    /// Index und Werte der aktiven Plugins.
    fn active(&self) -> Vec<(usize, Vec<f32>)> {
        self.slots
            .iter()
            .enumerate()
            .filter(|(_, s)| s.enabled.load(Ordering::Relaxed))
            .map(|(i, s)| (i, s.values.read().unwrap().clone()))
            .collect()
    }
}

fn setting_key(id: &str) -> String {
    format!("effect:{id}")
}

fn presets_key(id: &str) -> String {
    format!("effect-presets:{id}")
}

/// rodio-`Source`, die alle aktiven Plugins des Hosts anwendet.
pub struct EffectChain<S> {
    inner: S,
    host: Arc<PluginHost>,
    seen_version: u64,
    procs: Vec<(usize, Box<dyn Processor>)>,
    frame: Vec<f32>,
    pos: usize,
    /// Abgriff für die Visualisierung (Mono, gesammelt in kleinen Blöcken).
    analyzer: Option<Arc<Analyzer>>,
    /// Schutz gegen Übersteuern, sobald ein Effekt aktiv ist.
    safety: Option<SafetyLimiter>,
    tap: Vec<f32>,
}

impl<S: Source<Item = f32>> EffectChain<S> {
    pub fn new(inner: S, host: Arc<PluginHost>) -> Self {
        Self {
            inner,
            host,
            seen_version: 0,
            procs: Vec::new(),
            frame: Vec::with_capacity(8),
            pos: 0,
            analyzer: None,
            tap: Vec::new(),
            safety: None,
        }
    }

    /// Gibt das bearbeitete Signal zusätzlich an die Visualisierung weiter.
    pub fn with_analyzer(mut self, analyzer: Arc<Analyzer>) -> Self {
        self.analyzer = Some(analyzer);
        self.tap = Vec::with_capacity(512);
        self
    }

    /// Übernimmt Änderungen am Host; bestehende Prozessoren behalten ihren Zustand.
    fn sync(&mut self) {
        let v = self.host.version();
        if v == self.seen_version {
            return;
        }
        self.seen_version = v;
        let mut old = std::mem::take(&mut self.procs);
        for (idx, values) in self.host.active() {
            let mut proc = match old.iter().position(|(i, _)| *i == idx) {
                Some(p) => old.swap_remove(p).1,
                None => self.host.slots[idx]
                    .plugin
                    .create(self.inner.sample_rate(), self.inner.channels()),
            };
            proc.set_params(&values);
            self.procs.push((idx, proc));
        }
    }
}

impl<S: Source<Item = f32>> Iterator for EffectChain<S> {
    type Item = f32;

    fn next(&mut self) -> Option<f32> {
        if self.pos < self.frame.len() {
            self.pos += 1;
            return Some(self.frame[self.pos - 1]);
        }
        self.sync();
        let channels = self.inner.channels().max(1) as usize;
        self.frame.clear();
        while self.frame.len() < channels {
            match self.inner.next() {
                Some(s) => self.frame.push(s),
                None => break,
            }
        }
        if self.frame.is_empty() {
            return None;
        }
        if self.frame.len() == channels && !self.procs.is_empty() {
            for (_, p) in &mut self.procs {
                p.process(&mut self.frame);
            }
            // Weich begrenzen statt hart abschneiden – Abschneiden knackt hörbar.
            let rate = self.inner.sample_rate();
            self.safety.get_or_insert_with(|| SafetyLimiter::new(rate)).process(&mut self.frame);
        }
        if let Some(a) = &self.analyzer {
            self.tap.push(self.frame.iter().sum::<f32>() / self.frame.len() as f32);
            if self.tap.len() >= 512 {
                a.push(&self.tap, self.inner.sample_rate());
                self.tap.clear();
            }
        }
        self.pos = 1;
        Some(self.frame[0])
    }
}

impl<S: Source<Item = f32>> Source for EffectChain<S> {
    fn current_frame_len(&self) -> Option<usize> {
        let buffered = self.frame.len() - self.pos;
        self.inner.current_frame_len().map(|l| l + buffered)
    }

    fn channels(&self) -> u16 {
        self.inner.channels()
    }

    fn sample_rate(&self) -> u32 {
        self.inner.sample_rate()
    }

    fn total_duration(&self) -> Option<Duration> {
        self.inner.total_duration()
    }

    fn try_seek(&mut self, pos: Duration) -> Result<(), SeekError> {
        self.inner.try_seek(pos)?;
        self.frame.clear();
        self.pos = 0;
        for (_, p) in &mut self.procs {
            p.reset();
        }
        Ok(())
    }
}

/// Fängt Spitzen über −0,3 dBFS ab: sofort herunterregeln, in ~80 ms wieder freigeben.
/// So entsteht kein hartes Abschneiden (Knacken), nur bei echten Spitzen eine kurze Absenkung.
struct SafetyLimiter {
    gain: f32,
    release: f32,
}

impl SafetyLimiter {
    const CEILING: f32 = 0.966; // −0,3 dBFS

    fn new(rate: u32) -> Self {
        Self {
            gain: 1.0,
            release: 1.0 - (-1.0 / (0.08 * rate.max(1) as f32)).exp(),
        }
    }

    fn process(&mut self, frame: &mut [f32]) {
        let peak = frame.iter().fold(0f32, |m, s| m.max(s.abs()));
        let target = if peak > Self::CEILING { Self::CEILING / peak } else { 1.0 };
        if target < self.gain {
            self.gain = target;
        } else {
            self.gain += (target - self.gain) * self.release;
        }
        for s in frame {
            // letzte Sicherung, falls ein Plugin NaN oder Unsinn liefert
            *s = (*s * self.gain).clamp(-1.0, 1.0);
            if !s.is_finite() {
                *s = 0.0;
            }
        }
    }
}

/// Peaking-Biquad nach RBJ-Cookbook (wird von mehreren Plugins genutzt).
#[derive(Clone, Copy, Default)]
pub(crate) struct Biquad {
    b0: f32,
    b1: f32,
    b2: f32,
    a1: f32,
    a2: f32,
}

impl Biquad {
    pub fn identity() -> Self {
        Self {
            b0: 1.0,
            ..Default::default()
        }
    }

    pub fn peaking(freq: f32, gain_db: f32, q: f32, sample_rate: f32) -> Self {
        if gain_db.abs() < 0.01 || freq >= sample_rate * 0.45 {
            return Self::identity();
        }
        let a = 10f32.powf(gain_db / 40.0);
        let w0 = 2.0 * std::f32::consts::PI * freq / sample_rate;
        let (sin, cos) = w0.sin_cos();
        let alpha = sin / (2.0 * q);
        let a0 = 1.0 + alpha / a;
        Self {
            b0: (1.0 + alpha * a) / a0,
            b1: (-2.0 * cos) / a0,
            b2: (1.0 - alpha * a) / a0,
            a1: (-2.0 * cos) / a0,
            a2: (1.0 - alpha / a) / a0,
        }
    }

    /// Bass- (`high = false`) oder Höhenregler (`high = true`).
    pub fn shelf(high: bool, freq: f32, gain_db: f32, q: f32, sample_rate: f32) -> Self {
        let a = 10f32.powf(gain_db / 40.0);
        let w0 = 2.0 * std::f32::consts::PI * freq / sample_rate;
        let (sin, cos) = w0.sin_cos();
        let alpha = sin / (2.0 * q);
        let k = 2.0 * a.sqrt() * alpha;
        let (b0, b1, b2, a0, a1, a2) = if high {
            (
                a * ((a + 1.0) + (a - 1.0) * cos + k),
                -2.0 * a * ((a - 1.0) + (a + 1.0) * cos),
                a * ((a + 1.0) + (a - 1.0) * cos - k),
                (a + 1.0) - (a - 1.0) * cos + k,
                2.0 * ((a - 1.0) - (a + 1.0) * cos),
                (a + 1.0) - (a - 1.0) * cos - k,
            )
        } else {
            (
                a * ((a + 1.0) - (a - 1.0) * cos + k),
                2.0 * a * ((a - 1.0) - (a + 1.0) * cos),
                a * ((a + 1.0) - (a - 1.0) * cos - k),
                (a + 1.0) + (a - 1.0) * cos + k,
                -2.0 * ((a - 1.0) + (a + 1.0) * cos),
                (a + 1.0) + (a - 1.0) * cos - k,
            )
        };
        Self { b0: b0 / a0, b1: b1 / a0, b2: b2 / a0, a1: a1 / a0, a2: a2 / a0 }
    }

    /// Hochpass (`high = true`) oder Tiefpass.
    pub fn pass(high: bool, freq: f32, q: f32, sample_rate: f32) -> Self {
        let w0 = 2.0 * std::f32::consts::PI * freq / sample_rate;
        let (sin, cos) = w0.sin_cos();
        let alpha = sin / (2.0 * q);
        let a0 = 1.0 + alpha;
        let (b0, b1) = if high { ((1.0 + cos) / 2.0, -(1.0 + cos)) } else { ((1.0 - cos) / 2.0, 1.0 - cos) };
        Self { b0: b0 / a0, b1: b1 / a0, b2: b0 / a0, a1: -2.0 * cos / a0, a2: (1.0 - alpha) / a0 }
    }

    #[inline]
    pub fn run(&self, s: &mut BiquadState, x: f32) -> f32 {
        let y = self.b0 * x + self.b1 * s.x1 + self.b2 * s.x2 - self.a1 * s.y1 - self.a2 * s.y2;
        s.x2 = s.x1;
        s.x1 = x;
        s.y2 = s.y1;
        s.y1 = y;
        y
    }
}

#[derive(Clone, Copy, Default)]
pub(crate) struct BiquadState {
    x1: f32,
    x2: f32,
    y1: f32,
    y2: f32,
}

#[cfg(test)]
mod tests {
    use super::*;
    use rodio::buffer::SamplesBuffer;

    fn sine(freq: f32, rate: u32, channels: u16, amp: f32) -> SamplesBuffer<f32> {
        let samples = (0..rate)
            .flat_map(|i| {
                let v = amp * (2.0 * std::f32::consts::PI * freq * i as f32 / rate as f32).sin();
                std::iter::repeat(v).take(channels as usize)
            })
            .collect::<Vec<_>>();
        SamplesBuffer::new(channels, rate, samples)
    }

    fn peak(v: &[f32]) -> f32 {
        v[v.len() / 2..].iter().fold(0f32, |m, s| m.max(s.abs()))
    }

    #[test]
    fn disabled_chain_is_transparent() {
        let host = PluginHost::with_builtin(None);
        let input: Vec<f32> = sine(440.0, 8000, 2, 0.5).collect();
        let out: Vec<f32> = EffectChain::new(sine(440.0, 8000, 2, 0.5), host).collect();
        assert_eq!(out, input);
    }

    #[test]
    fn equalizer_boosts_band() {
        let host = PluginHost::with_builtin(None);
        let mut gains = vec![0.0; 10];
        gains[1] = 12.0;
        host.set("equalizer", Some(true), Some(&gains)).unwrap();
        // Der EQ senkt automatisch um die stärkste Anhebung ab: das angehobene Band bleibt etwa
        // gleich laut, alles andere wird um 12 dB leiser – das Verhältnis ist die Anhebung.
        let bass = peak(&EffectChain::new(sine(62.0, 44100, 1, 0.1), host.clone()).collect::<Vec<_>>());
        let mid = peak(&EffectChain::new(sine(4000.0, 44100, 1, 0.1), host).collect::<Vec<_>>());
        assert!((bass - 0.1).abs() < 0.02, "Bass {bass}");
        assert!(bass / mid > 3.0, "Bass {bass}, Mitten {mid}");
    }

    #[test]
    fn stereo_width_zero_makes_mono() {
        let host = PluginHost::with_builtin(None);
        host.set("stereo_width", Some(true), Some(&[0.0])).unwrap();
        let src = SamplesBuffer::new(2, 44100, vec![1.0, 0.0, 0.0, 1.0]);
        let out: Vec<f32> = EffectChain::new(src, host).collect();
        assert_eq!(out, vec![0.5, 0.5, 0.5, 0.5]);
    }

    #[test]
    fn loud_bass_boost_does_not_clip() {
        // Sehr lauter Bass (0,95) mit EQ +12 dB: früher hart abgeschnitten, jetzt sauber begrenzt.
        let host = PluginHost::with_builtin(None);
        let mut gains = vec![0.0; 10];
        gains[0] = 12.0;
        gains[1] = 12.0;
        host.set("equalizer", Some(true), Some(&gains)).unwrap();
        let out: Vec<f32> = EffectChain::new(sine(50.0, 44100, 2, 0.95), host).collect();
        let tail = &out[out.len() / 2..];
        let clipped = tail.iter().filter(|s| s.abs() >= 0.999).count();
        assert_eq!(clipped, 0, "kein hartes Abschneiden");
        assert!(peak(&out) <= SafetyLimiter::CEILING + 1e-3);
        assert!(peak(&out) > 0.3, "aber trotzdem kräftig");
    }

    #[test]
    fn reverb_adds_tail_and_mix_controls_it() {
        // Ein kurzer Klick: ohne Hall ist danach Stille, mit Hall klingt er nach.
        let click = || {
            let mut v = vec![0.0f32; 2 * 44100];
            v[0] = 0.8;
            v[1] = 0.8;
            SamplesBuffer::new(2, 44100, v)
        };
        let tail = |mix: f32| {
            let host = PluginHost::with_builtin(None);
            host.set("reverb", Some(true), Some(&[45.0, 60.0, 180.0, 1.6, 20.0, mix])).unwrap();
            let out: Vec<f32> = EffectChain::new(click(), host).collect();
            out[2 * 4410..2 * 22050].iter().map(|s| s * s).sum::<f32>()
        };
        assert_eq!(tail(0.0), 0.0);
        let low = tail(20.0);
        let high = tail(100.0);
        assert!(low > 0.0, "Hall klingt nach");
        assert!(high > low * 10.0, "Mix macht den Hall stärker: {low} / {high}");
    }

    #[test]
    fn tone_and_balance() {
        let host = PluginHost::with_builtin(None);
        host.set("tone", Some(true), Some(&[10.0, 0.0])).unwrap();
        let bass = peak(&EffectChain::new(sine(50.0, 44100, 1, 0.1), host.clone()).collect::<Vec<_>>());
        let high = peak(&EffectChain::new(sine(5000.0, 44100, 1, 0.1), host).collect::<Vec<_>>());
        assert!(bass / high > 2.5, "Bass {bass}, Höhen {high}");

        let host = PluginHost::with_builtin(None);
        host.set("stereo_width", Some(true), Some(&[100.0, 100.0])).unwrap();
        let out: Vec<f32> = EffectChain::new(SamplesBuffer::new(2, 44100, vec![0.5, 0.5]), host).collect();
        assert_eq!(out, vec![0.0, 0.5], "Balance ganz rechts");
    }

    #[test]
    fn user_presets_save_apply_delete_reset() {
        let host = PluginHost::with_builtin(None);
        host.set("reverb", None, Some(&[80.0, 90.0, 300.0, 3.0, 50.0, 40.0])).unwrap();
        let info = host.save_preset("reverb", " Mein Raum ").unwrap();
        assert_eq!(info.user_presets[0].name, "Mein Raum");
        let info = host.reset("reverb").unwrap();
        assert_eq!(info.values[1], 60.0);
        let info = host.apply_preset("reverb", "Mein Raum").unwrap();
        assert_eq!(info.values[1], 90.0);
        assert!(info.enabled);
        let info = host.delete_preset("reverb", "Mein Raum").unwrap();
        assert!(info.user_presets.is_empty());
        assert!(host.save_preset("reverb", "  ").is_err());
    }

    #[test]
    fn limiter_respects_ceiling() {
        let host = PluginHost::with_builtin(None);
        host.set("limiter", Some(true), Some(&[12.0, -1.0])).unwrap();
        let out: Vec<f32> = EffectChain::new(sine(440.0, 44100, 2, 0.9), host).collect();
        let ceiling = 10f32.powf(-1.0 / 20.0);
        assert!(peak(&out) <= ceiling + 1e-3, "peak {}", peak(&out));
    }

    #[test]
    fn values_are_clamped_and_presets_apply() {
        let host = PluginHost::with_builtin(None);
        let info = host.set_param("equalizer", "31", 99.0).unwrap();
        assert_eq!(info.values[0], 12.0);
        let info = host.apply_preset("equalizer", "rock").unwrap();
        assert!(info.enabled);
        assert!(host.set("nope", Some(true), None).is_err());
    }
}
