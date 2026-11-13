//! Klang-Plugins aus JSON-Dateien: Regler, Presets und eine Kette aus fertigen Bausteinen.
//! Werte dürfen Formeln mit den Reglern sein, z. B. `"freq": "3500 - staerke * 20"`.
//! Formeln werden nur bei Regleränderungen ausgewertet, die Klangberechnung bleibt nativ.

use std::collections::HashMap;
use std::f32::consts::PI;

use anyhow::{anyhow, bail, Context, Result};
use serde::Deserialize;
use serde_json::Value;

use super::{Biquad, BiquadState, EffectPlugin, ParamInfo, Preset, Processor};

const MAX_CHANNELS: usize = 8;
const MAX_DELAY_S: f32 = 2.0;

// ---------- Beschreibung (JSON) ----------

#[derive(Debug, Clone, Deserialize)]
struct Def {
    id: String,
    name: String,
    #[serde(default)]
    author: String,
    #[serde(default)]
    description: String,
    #[serde(default)]
    params: Vec<ParamDef>,
    #[serde(default)]
    presets: Vec<PresetDef>,
    chain: Vec<HashMap<String, Value>>,
    /// Anteil des bearbeiteten Signals (Formel möglich), Standard 1
    #[serde(default)]
    mix: Option<Value>,
}

#[derive(Debug, Clone, Deserialize)]
struct ParamDef {
    id: String,
    name: String,
    min: f32,
    max: f32,
    default: f32,
    #[serde(default = "one")]
    step: f32,
    #[serde(default)]
    unit: String,
}

fn one() -> f32 {
    1.0
}

#[derive(Debug, Clone, Deserialize)]
struct PresetDef {
    name: String,
    values: Vec<f32>,
}

/// Eingebaute Beispiel-Plugins im selben Format, wie es Nutzer schreiben.
const BUILTIN_JSON: &[(&str, &str)] = &[
    ("bass-booster.json", include_str!("builtin/bass-booster.json")),
    ("stimmen-klarer.json", include_str!("builtin/stimmen-klarer.json")),
    ("altes-radio.json", include_str!("builtin/altes-radio.json")),
    ("echo.json", include_str!("builtin/echo.json")),
    ("rundflug.json", include_str!("builtin/rundflug.json")),
];

pub fn builtin_json_plugins() -> Vec<CustomPlugin> {
    BUILTIN_JSON
        .iter()
        .map(|(file, json)| CustomPlugin::parse(json, "").unwrap_or_else(|e| panic!("{file}: {e:#}")))
        .collect()
}

/// Lädt alle `*.json` aus einem Ordner und alle Plugin-Pakete (Unterordner mit `plugin.json`).
/// Fehlerhafte Dateien landen mit Meldung in der zweiten Liste.
pub fn load_dir(dir: &std::path::Path) -> (Vec<CustomPlugin>, Vec<String>) {
    let (mut ok, mut errors) = load_files(dir);
    // Pakete: der Paket-Ordner neben effects/ und alte Ein-Teil-Pakete in effects/<id>/
    let root = dir.parent().unwrap_or(dir);
    let bases = [root.join(crate::packages::PACKAGES_DIR), dir.to_path_buf()];
    let (parts, package_errors) = crate::packages::contents_in(&bases, crate::packages::Kind::Effect);
    errors.extend(package_errors);
    for (source, text) in parts {
        match text.and_then(|text| CustomPlugin::parse(&text, &source)) {
            Ok(plugin) => ok.push(plugin),
            Err(e) => errors.push(format!("{source}: {e:#}")),
        }
    }
    (ok, errors)
}

/// Einzelne `*.json`-Dateien direkt im Ordner (ältere Art, ohne Paket).
fn load_files(dir: &std::path::Path) -> (Vec<CustomPlugin>, Vec<String>) {
    let mut ok = Vec::new();
    let mut errors = Vec::new();
    let Ok(entries) = std::fs::read_dir(dir) else {
        return (ok, errors);
    };
    let mut files: Vec<_> = entries
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|x| x == "json"))
        .collect();
    files.sort();
    for path in files {
        let name = path.file_name().unwrap_or_default().to_string_lossy().into_owned();
        let result = std::fs::read_to_string(&path)
            .map_err(anyhow::Error::from)
            .and_then(|text| CustomPlugin::parse(&text, &name));
        match result {
            Ok(p) => ok.push(p),
            Err(e) => errors.push(format!("{name}: {e:#}")),
        }
    }
    (ok, errors)
}

/// Strings der Plugin-Beschreibung leben so lange wie das Programm (Plugins werden einmal geladen).
fn leak(s: String) -> &'static str {
    Box::leak(s.into_boxed_str())
}

// ---------- Formeln ----------

#[derive(Debug, Clone)]
enum Expr {
    Num(f32),
    Var(usize),
    Neg(Box<Expr>),
    Bin(char, Box<Expr>, Box<Expr>),
    Call(String, Vec<Expr>),
}

impl Expr {
    fn eval(&self, vars: &[f32]) -> f32 {
        match self {
            Expr::Num(n) => *n,
            Expr::Var(i) => vars.get(*i).copied().unwrap_or(0.0),
            Expr::Neg(e) => -e.eval(vars),
            Expr::Bin(op, a, b) => {
                let (a, b) = (a.eval(vars), b.eval(vars));
                match op {
                    '+' => a + b,
                    '-' => a - b,
                    '*' => a * b,
                    '/' => if b == 0.0 { 0.0 } else { a / b },
                    _ => a.powf(b),
                }
            }
            Expr::Call(f, args) => {
                let v: Vec<f32> = args.iter().map(|a| a.eval(vars)).collect();
                match (f.as_str(), v.as_slice()) {
                    ("min", [a, b]) => a.min(*b),
                    ("max", [a, b]) => a.max(*b),
                    ("clamp", [x, a, b]) => x.clamp(a.min(*b), a.max(*b)),
                    ("abs", [x]) => x.abs(),
                    ("sqrt", [x]) => x.max(0.0).sqrt(),
                    ("pow", [a, b]) => a.powf(*b),
                    ("exp", [x]) => x.exp(),
                    ("log10", [x]) => x.max(1e-9).log10(),
                    _ => 0.0,
                }
            }
        }
    }
}

struct ExprParser<'a> {
    chars: Vec<char>,
    pos: usize,
    params: &'a [String],
}

impl<'a> ExprParser<'a> {
    fn parse(src: &str, params: &'a [String]) -> Result<Expr> {
        let mut p = ExprParser { chars: src.chars().collect(), pos: 0, params };
        let e = p.sum()?;
        p.ws();
        if p.pos < p.chars.len() {
            bail!("unerwartetes Zeichen „{}“ in „{src}“", p.chars[p.pos]);
        }
        Ok(e)
    }

    fn ws(&mut self) {
        while self.chars.get(self.pos).is_some_and(|c| c.is_whitespace()) {
            self.pos += 1;
        }
    }

    fn peek(&mut self) -> Option<char> {
        self.ws();
        self.chars.get(self.pos).copied()
    }

    fn sum(&mut self) -> Result<Expr> {
        let mut e = self.product()?;
        while let Some(op @ ('+' | '-')) = self.peek() {
            self.pos += 1;
            e = Expr::Bin(op, Box::new(e), Box::new(self.product()?));
        }
        Ok(e)
    }

    fn product(&mut self) -> Result<Expr> {
        let mut e = self.power()?;
        while let Some(op @ ('*' | '/')) = self.peek() {
            self.pos += 1;
            e = Expr::Bin(op, Box::new(e), Box::new(self.power()?));
        }
        Ok(e)
    }

    fn power(&mut self) -> Result<Expr> {
        let base = self.unary()?;
        if self.peek() == Some('^') {
            self.pos += 1;
            return Ok(Expr::Bin('^', Box::new(base), Box::new(self.power()?)));
        }
        Ok(base)
    }

    fn unary(&mut self) -> Result<Expr> {
        if self.peek() == Some('-') {
            self.pos += 1;
            return Ok(Expr::Neg(Box::new(self.unary()?)));
        }
        self.atom()
    }

    fn atom(&mut self) -> Result<Expr> {
        match self.peek() {
            Some('(') => {
                self.pos += 1;
                let e = self.sum()?;
                if self.peek() != Some(')') {
                    bail!("„)“ fehlt");
                }
                self.pos += 1;
                Ok(e)
            }
            Some(c) if c.is_ascii_digit() || c == '.' => {
                let start = self.pos;
                while self.chars.get(self.pos).is_some_and(|c| c.is_ascii_digit() || *c == '.') {
                    self.pos += 1;
                }
                let s: String = self.chars[start..self.pos].iter().collect();
                Ok(Expr::Num(s.parse().map_err(|_| anyhow!("ungültige Zahl „{s}“"))?))
            }
            Some(c) if c.is_alphabetic() || c == '_' => {
                let start = self.pos;
                while self.chars.get(self.pos).is_some_and(|c| c.is_alphanumeric() || *c == '_') {
                    self.pos += 1;
                }
                let name: String = self.chars[start..self.pos].iter().collect();
                if self.peek() == Some('(') {
                    self.pos += 1;
                    let mut args = Vec::new();
                    if self.peek() != Some(')') {
                        loop {
                            args.push(self.sum()?);
                            match self.peek() {
                                Some(',') => self.pos += 1,
                                Some(')') => break,
                                _ => bail!("„,“ oder „)“ fehlt nach Argument von {name}"),
                            }
                        }
                    }
                    self.pos += 1;
                    if !["min", "max", "clamp", "abs", "sqrt", "pow", "exp", "log10"].contains(&name.as_str()) {
                        bail!("unbekannte Funktion „{name}“");
                    }
                    return Ok(Expr::Call(name, args));
                }
                match self.params.iter().position(|p| *p == name) {
                    Some(i) => Ok(Expr::Var(i)),
                    None if name == "pi" => Ok(Expr::Num(PI)),
                    None => bail!("unbekannter Regler „{name}“"),
                }
            }
            Some(c) => bail!("unerwartetes Zeichen „{c}“"),
            None => bail!("Formel endet zu früh"),
        }
    }
}

/// Wert aus JSON: Zahl oder Formel-Text.
fn expr(v: Option<&Value>, default: f32, params: &[String]) -> Result<Expr> {
    match v {
        None => Ok(Expr::Num(default)),
        Some(Value::Number(n)) => Ok(Expr::Num(n.as_f64().unwrap_or(0.0) as f32)),
        Some(Value::String(s)) => ExprParser::parse(s, params),
        Some(other) => bail!("Zahl oder Formel erwartet, nicht {other}"),
    }
}

// ---------- Bausteine ----------

#[derive(Debug, Clone)]
enum BlockKind {
    Gain,
    Peaking,
    LowShelf,
    HighShelf,
    LowPass,
    HighPass,
    Echo,
    Width,
    Saturate,
    Compressor,
    AutoPan,
}

/// Erlaubte Felder je Baustein mit Standardwert.
fn block_fields(kind: &BlockKind) -> &'static [(&'static str, f32)] {
    match kind {
        BlockKind::Gain => &[("db", 0.0)],
        BlockKind::Peaking => &[("freq", 1000.0), ("gain", 0.0), ("q", 1.0)],
        BlockKind::LowShelf | BlockKind::HighShelf => &[("freq", 200.0), ("gain", 0.0), ("q", 0.707)],
        BlockKind::LowPass | BlockKind::HighPass => &[("freq", 1000.0), ("q", 0.707)],
        BlockKind::Echo => &[("time", 300.0), ("feedback", 0.3), ("mix", 0.3)],
        BlockKind::Width => &[("amount", 1.0)],
        BlockKind::Saturate => &[("drive", 1.0)],
        BlockKind::Compressor => &[("threshold", -18.0), ("ratio", 3.0), ("attack", 10.0), ("release", 150.0), ("makeup", 0.0)],
        BlockKind::AutoPan => &[("rate", 0.2), ("depth", 0.8)],
    }
}

#[derive(Debug, Clone)]
struct BlockDef {
    kind: BlockKind,
    fields: Vec<Expr>,
}

fn parse_block(raw: &HashMap<String, Value>, params: &[String]) -> Result<BlockDef> {
    let ty = raw.get("type").and_then(Value::as_str).ok_or_else(|| anyhow!("Baustein ohne „type“"))?;
    let kind = match ty {
        "gain" => BlockKind::Gain,
        "peaking" => BlockKind::Peaking,
        "lowshelf" => BlockKind::LowShelf,
        "highshelf" => BlockKind::HighShelf,
        "lowpass" => BlockKind::LowPass,
        "highpass" => BlockKind::HighPass,
        "echo" => BlockKind::Echo,
        "width" => BlockKind::Width,
        "saturate" => BlockKind::Saturate,
        "compressor" => BlockKind::Compressor,
        "autopan" => BlockKind::AutoPan,
        other => bail!(
            "unbekannter Baustein „{other}“ (möglich: gain, peaking, lowshelf, highshelf, lowpass, highpass, echo, width, saturate, compressor, autopan)"
        ),
    };
    let allowed = block_fields(&kind);
    for key in raw.keys() {
        if key != "type" && !allowed.iter().any(|(k, _)| k == key) {
            bail!("Baustein „{ty}“ kennt das Feld „{key}“ nicht");
        }
    }
    let fields = allowed
        .iter()
        .map(|(k, d)| expr(raw.get(*k), *d, params).with_context(|| format!("{ty}.{k}")))
        .collect::<Result<_>>()?;
    Ok(BlockDef { kind, fields })
}

// ---------- Plugin ----------

pub struct CustomPlugin {
    id: &'static str,
    name: &'static str,
    description: &'static str,
    params: Vec<ParamInfo>,
    presets: Vec<(String, Vec<f32>)>,
    blocks: Vec<BlockDef>,
    mix: Expr,
    /// Dateiname bei eigenen Plugins, `None` bei mitgelieferten
    pub source: Option<String>,
}

impl CustomPlugin {
    pub fn parse(json: &str, source: &str) -> Result<Self> {
        let def: Def = serde_json::from_str(json).context("ungültiges JSON")?;
        if def.id.is_empty() || def.id.len() > 32 || !def.id.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_') {
            bail!("„id“ muss 1–32 Zeichen aus Buchstaben, Ziffern, - oder _ sein");
        }
        if def.chain.is_empty() || def.chain.len() > 32 {
            bail!("„chain“ braucht 1 bis 32 Bausteine");
        }
        if def.params.len() > 16 {
            bail!("höchstens 16 Regler");
        }
        let names: Vec<String> = def.params.iter().map(|p| p.id.clone()).collect();
        for p in &def.params {
            if !(p.min < p.max) || p.default < p.min || p.default > p.max {
                bail!("Regler „{}“: min < max und default dazwischen", p.id);
            }
        }
        let blocks = def
            .chain
            .iter()
            .enumerate()
            .map(|(i, b)| parse_block(b, &names).with_context(|| format!("Baustein {}", i + 1)))
            .collect::<Result<_>>()?;
        let mix = expr(def.mix.as_ref(), 1.0, &names).context("mix")?;
        let description = if def.author.is_empty() {
            def.description
        } else {
            format!("{} · von {}", def.description, def.author)
        };
        Ok(Self {
            id: leak(def.id),
            name: leak(def.name.chars().take(40).collect()),
            description: leak(description.chars().take(240).collect()),
            params: def
                .params
                .into_iter()
                .map(|p| ParamInfo {
                    id: leak(p.id),
                    name: leak(p.name),
                    min: p.min,
                    max: p.max,
                    default: p.default,
                    step: p.step,
                    unit: leak(p.unit),
                })
                .collect(),
            presets: def.presets.into_iter().map(|p| (p.name, p.values)).collect(),
            blocks,
            mix,
            source: (!source.is_empty()).then(|| source.to_string()),
        })
    }
}

impl EffectPlugin for CustomPlugin {
    fn id(&self) -> &'static str {
        self.id
    }
    fn name(&self) -> &'static str {
        self.name
    }
    fn description(&self) -> &'static str {
        self.description
    }
    fn params(&self) -> Vec<ParamInfo> {
        self.params.clone()
    }
    fn presets(&self) -> Vec<Preset> {
        self.presets
            .iter()
            .map(|(n, v)| Preset { name: leak(n.clone()), values: v.clone() })
            .collect()
    }
    fn source(&self) -> Option<String> {
        self.source.clone()
    }
    fn create(&self, sample_rate: u32, channels: u16) -> Box<dyn Processor> {
        let ch = (channels as usize).clamp(1, MAX_CHANNELS);
        Box::new(CustomProcessor {
            rate: sample_rate as f32,
            channels: ch,
            mix_expr: self.mix.clone(),
            mix: 1.0,
            blocks: self.blocks.iter().map(|d| Block::new(d.clone(), sample_rate as f32, ch)).collect(),
            dry: vec![0.0; ch],
        })
    }
}

struct Block {
    def: BlockDef,
    v: Vec<f32>,
    biquad: Biquad,
    states: Vec<BiquadState>,
    delay: Vec<Vec<f32>>,
    delay_pos: usize,
    env: f32,
    phase: f32,
    rate: f32,
}

impl Block {
    fn new(def: BlockDef, rate: f32, ch: usize) -> Self {
        let delay = if matches!(def.kind, BlockKind::Echo) {
            vec![vec![0.0; (rate * MAX_DELAY_S) as usize + 1]; ch]
        } else {
            Vec::new()
        };
        let n = def.fields.len();
        Self {
            def,
            v: vec![0.0; n],
            biquad: Biquad::identity(),
            states: vec![BiquadState::default(); ch],
            delay,
            delay_pos: 0,
            env: 0.0,
            phase: 0.0,
            rate,
        }
    }

    fn update(&mut self, vars: &[f32]) {
        self.v = self.def.fields.iter().map(|e| e.eval(vars)).collect();
        let r = self.rate;
        let nyq = r * 0.45;
        self.biquad = match self.def.kind {
            BlockKind::Peaking => Biquad::peaking(self.v[0].clamp(20.0, nyq), self.v[1].clamp(-24.0, 24.0), self.v[2].max(0.1), r),
            BlockKind::LowShelf => Biquad::shelf(false, self.v[0].clamp(20.0, nyq), self.v[1].clamp(-24.0, 24.0), self.v[2].max(0.1), r),
            BlockKind::HighShelf => Biquad::shelf(true, self.v[0].clamp(20.0, nyq), self.v[1].clamp(-24.0, 24.0), self.v[2].max(0.1), r),
            BlockKind::LowPass => Biquad::pass(false, self.v[0].clamp(20.0, nyq), self.v[1].max(0.1), r),
            BlockKind::HighPass => Biquad::pass(true, self.v[0].clamp(20.0, nyq), self.v[1].max(0.1), r),
            _ => Biquad::identity(),
        };
    }

    fn process(&mut self, frame: &mut [f32]) {
        match self.def.kind {
            BlockKind::Gain => {
                let g = 10f32.powf(self.v[0].clamp(-48.0, 24.0) / 20.0);
                frame.iter_mut().for_each(|s| *s *= g);
            }
            BlockKind::Peaking | BlockKind::LowShelf | BlockKind::HighShelf | BlockKind::LowPass | BlockKind::HighPass => {
                for (s, st) in frame.iter_mut().zip(&mut self.states) {
                    *s = self.biquad.run(st, *s);
                }
            }
            BlockKind::Echo => {
                let len = self.delay[0].len();
                let d = ((self.v[0].clamp(1.0, MAX_DELAY_S * 1000.0) / 1000.0) * self.rate) as usize;
                let fb = self.v[1].clamp(0.0, 0.95);
                let mix = self.v[2].clamp(0.0, 1.0);
                let read = (self.delay_pos + len - d.min(len - 1)) % len;
                for (s, buf) in frame.iter_mut().zip(&mut self.delay) {
                    let echo = buf[read];
                    buf[self.delay_pos] = *s + echo * fb;
                    *s += echo * mix;
                }
                self.delay_pos = (self.delay_pos + 1) % len;
            }
            BlockKind::Width => {
                if let [l, r] = frame {
                    let mid = (*l + *r) * 0.5;
                    let side = (*l - *r) * 0.5 * self.v[0].clamp(0.0, 3.0);
                    *l = mid + side;
                    *r = mid - side;
                }
            }
            BlockKind::Saturate => {
                let drive = self.v[0].clamp(0.01, 20.0);
                // Vollaussteuerung bleibt Vollaussteuerung, leise Stellen werden dichter.
                let norm = drive.tanh();
                frame.iter_mut().for_each(|s| *s = (*s * drive).tanh() / norm);
            }
            BlockKind::Compressor => {
                let peak = frame.iter().fold(0f32, |m, s| m.max(s.abs())).max(1e-9);
                let level_db = 20.0 * peak.log10();
                let coef = |ms: f32| (-1.0 / (ms.max(0.1) / 1000.0 * self.rate)).exp();
                let c = if level_db > self.env { coef(self.v[2]) } else { coef(self.v[3]) };
                self.env = c * self.env + (1.0 - c) * level_db;
                let over = (self.env - self.v[0]).max(0.0);
                let reduce = over - over / self.v[1].max(1.0);
                let g = 10f32.powf((self.v[4] - reduce) / 20.0);
                frame.iter_mut().for_each(|s| *s *= g);
            }
            BlockKind::AutoPan => {
                if let [l, r] = frame {
                    self.phase = (self.phase + self.v[0].clamp(0.01, 20.0) / self.rate) % 1.0;
                    let pan = (self.phase * 2.0 * PI).sin() * self.v[1].clamp(0.0, 1.0);
                    let (gl, gr) = ((1.0 - pan).min(1.0), (1.0 + pan).min(1.0));
                    *l *= gl;
                    *r *= gr;
                }
            }
        }
    }

    fn reset(&mut self) {
        self.states.iter_mut().for_each(|s| *s = BiquadState::default());
        self.delay.iter_mut().for_each(|b| b.iter_mut().for_each(|x| *x = 0.0));
        self.env = 0.0;
    }
}

struct CustomProcessor {
    #[allow(dead_code)]
    rate: f32,
    channels: usize,
    mix_expr: Expr,
    mix: f32,
    blocks: Vec<Block>,
    dry: Vec<f32>,
}

impl Processor for CustomProcessor {
    fn set_params(&mut self, values: &[f32]) {
        self.mix = self.mix_expr.eval(values).clamp(0.0, 1.0);
        self.blocks.iter_mut().for_each(|b| b.update(values));
    }

    fn process(&mut self, frame: &mut [f32]) {
        let n = frame.len().min(self.channels);
        let frame = &mut frame[..n];
        self.dry[..n].copy_from_slice(frame);
        for b in &mut self.blocks {
            b.process(frame);
        }
        if self.mix < 1.0 {
            for (s, d) in frame.iter_mut().zip(&self.dry) {
                *s = d + (*s - d) * self.mix;
            }
        }
    }

    fn reset(&mut self) {
        self.blocks.iter_mut().for_each(Block::reset);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn names(v: &[&str]) -> Vec<String> {
        v.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn formulas() {
        let p = names(&["a", "b"]);
        let e = ExprParser::parse("3500 - a * 15 + max(b, 2) ^ 2 / (1 + 1)", &p).unwrap();
        assert_eq!(e.eval(&[10.0, 4.0]), 3500.0 - 150.0 + 8.0);
        assert_eq!(ExprParser::parse("-a", &p).unwrap().eval(&[2.0, 0.0]), -2.0);
        assert!(ExprParser::parse("c + 1", &p).is_err());
        assert!(ExprParser::parse("foo(1)", &p).is_err());
        assert!(ExprParser::parse("1 +", &p).is_err());
    }

    #[test]
    fn builtin_plugins_parse() {
        let all = builtin_json_plugins();
        assert_eq!(all.len(), BUILTIN_JSON.len());
        for p in &all {
            let mut proc = p.create(44_100, 2);
            let defaults: Vec<f32> = p.params().iter().map(|x| x.default).collect();
            proc.set_params(&defaults);
            let mut frame = [0.3, -0.2];
            for _ in 0..2000 {
                proc.process(&mut frame);
                assert!(frame.iter().all(|s| s.is_finite()), "{}", p.id());
            }
        }
    }

    #[test]
    fn bad_definitions_are_rejected() {
        let bad = [
            r#"{"id":"x","name":"X","chain":[{"type":"zauber"}]}"#,
            r#"{"id":"x","name":"X","chain":[{"type":"gain","db":"lautheit"}]}"#,
            r#"{"id":"x","name":"X","chain":[{"type":"gain","farbe":1}]}"#,
            r#"{"id":"x y","name":"X","chain":[{"type":"gain"}]}"#,
            r#"{"id":"x","name":"X","chain":[]}"#,
        ];
        for b in bad {
            assert!(CustomPlugin::parse(b, "t").is_err(), "{b}");
        }
    }

    #[test]
    fn gain_block_uses_param() {
        let p = CustomPlugin::parse(
            r#"{"id":"g","name":"G","params":[{"id":"db","name":"dB","min":-12,"max":12,"default":0}],
               "chain":[{"type":"gain","db":"db"}]}"#,
            "t",
        )
        .unwrap();
        let mut proc = p.create(44_100, 1);
        proc.set_params(&[6.0]);
        let mut f = [0.25];
        proc.process(&mut f);
        assert!((f[0] - 0.25 * 10f32.powf(0.3)).abs() < 1e-4);
    }
}
