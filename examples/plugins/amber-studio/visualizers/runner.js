// Amber Studio: Amber Runner – ein Läufer rennt durch eine Neon-Welt (Pseudo-3D im Spiel-Look).
// Er weicht Hindernissen aus, springt über niedrige Blöcke und sammelt Orbs im Takt der Musik.
amber.visualizer({
  id: 'amber-runner',
  name: 'Amber Runner',
  author: 'Marcel1853',
  description: 'Spiel-Optik: ein Läufer rennt durch eine Neon-Welt, weicht aus, springt und sammelt Orbs',

  setup({ state: s }) {
    Object.assign(s, {
      dist: 0, speed: 6, lane: 0, target: 0, cool: 0,
      obs: [], orbs: [], parts: [],
      score: 0, combo: 0, jt: -1, jamp: 1, phase: 0,
      flash: 0, shake: 0, hurt: 0, spawn: 1, prev: 0, bass: 0, t: 0,
      mt: new Array(40).fill(0),
    });
  },

  draw({ ctx, width: w, height: h, frame, info, state: s, amber }) {
    const dt = Math.min(frame.dt || 0.016, 0.05);
    const acc = info.accent, acc2 = info.accent2;
    const playing = frame.playing;
    const cx = w / 2, hor = h * 0.46, yp = h * 0.84;
    const laneW = Math.min(w * 0.14, h * 0.2);
    const ZP = 4; // Abstand des Läufers zur Kamera
    const Y = (z) => hor + (yp - hor) * ZP / z;
    const X = (lx, z) => cx + lx * laneW * ZP / z;
    const SC = (z) => ZP / z;

    // ---------- Logik ----------
    s.t += dt;
    s.bass = amber.lerp(s.bass, frame.bass, 0.25);
    s.speed = amber.lerp(s.speed, playing ? 7 + frame.level * 16 + frame.bass * 4 : 2.5, 0.05);
    s.dist += s.speed * dt;
    s.phase += dt * (2.5 + s.speed * 0.55);
    s.flash = Math.max(0, s.flash - dt * 3);
    s.shake = Math.max(0, s.shake - dt * 3);
    s.hurt = Math.max(0, s.hurt - dt * 2.5);
    s.cool -= dt;

    const beat = playing && frame.bass > 0.42 && frame.bass - s.prev > 0.06;
    s.prev = frame.bass;
    if (beat) s.flash = 1;

    // Spawnen
    s.spawn -= dt;
    if (playing && (s.spawn <= 0 || (beat && Math.random() < 0.35)) && s.obs.length + s.orbs.length < 28) {
      s.spawn = Math.max(0.3, 0.45 + Math.random() * 0.5 - frame.level * 0.25);
      if (Math.random() < 0.55) {
        const lane = Math.floor(Math.random() * 3) - 1;
        const wall = Math.random() < 0.5;
        s.obs.push({ z: 62, lane, wall });
        if (wall && Math.random() < 0.35) {
          const l2 = lane === 0 ? (Math.random() < 0.5 ? -1 : 1) : 0;
          s.obs.push({ z: 62, lane: l2, wall: true });
        }
      } else {
        const lane = Math.floor(Math.random() * 3) - 1;
        for (let i = 0; i < 5; i++) s.orbs.push({ z: 62 + i * 2.4, lane, got: false });
      }
    }

    // Läufer-KI
    const blocked = (l, z0, z1) => s.obs.some((o) => o.lane === l && o.z > z0 && o.z < z1);
    if (s.cool <= 0) {
      let ahead = null;
      for (const o of s.obs) {
        if (o.lane === s.target && o.z > 5 && o.z < 24 && (!ahead || o.z < ahead.z)) ahead = o;
      }
      if (ahead && ahead.wall) {
        const cands = [-1, 0, 1].sort((a, b) => Math.abs(a - s.target) - Math.abs(b - s.target));
        const free = cands.find((l) => l !== s.target && !blocked(l, 4, 26));
        if (free !== undefined) { s.target = free; s.cool = 0.3; }
      } else if (!ahead) {
        let best = null;
        for (const o of s.orbs) {
          if (!o.got && o.z > 8 && o.z < 34 && !blocked(o.lane, 4, 34) && (!best || o.z < best.z)) best = o;
        }
        if (best && best.lane !== s.target) { s.target = best.lane; s.cool = 0.3; }
      }
    }
    // Sprung vor niedrigen Blöcken (Spitze der Flugbahn trifft den Block)
    if (s.jt < 0) {
      for (const o of s.obs) {
        if (!o.wall && o.lane === s.target && o.z > ZP) {
          const tt = (o.z - ZP) / Math.max(s.speed, 1);
          if (tt < 0.31) { s.jt = 0; s.jamp = 1; break; }
        }
      }
      if (s.jt < 0 && beat && Math.random() < 0.25) { s.jt = 0; s.jamp = 0.5; }
    }
    s.lane += (s.target - s.lane) * Math.min(1, dt * 9);
    let jh = 0;
    if (s.jt >= 0) {
      s.jt += dt / 0.62;
      if (s.jt >= 1) s.jt = -1; else jh = Math.sin(s.jt * Math.PI) * s.jamp;
    }

    const pxScreen = cx + s.lane * laneW;
    const burst = (n) => {
      for (let i = 0; i < n && s.parts.length < 120; i++) {
        const a = Math.random() * Math.PI * 2, sp = 60 + Math.random() * 160;
        s.parts.push({ x: pxScreen, y: yp - laneW * 0.9, vx: Math.cos(a) * sp, vy: Math.sin(a) * sp - 60, life: 1, c: Math.random() });
      }
    };

    for (const o of s.obs) {
      o.z -= s.speed * dt;
      if (!o.hit && o.z < 4.4 && o.z > 3.4 && Math.abs(o.lane - s.lane) < 0.6) {
        if (!(!o.wall && jh > 0.35)) {
          o.hit = true;
          s.hurt = 1; s.shake = 1; s.combo = 0; s.score = Math.max(0, s.score - 20);
        }
      }
    }
    for (const o of s.orbs) {
      o.z -= s.speed * dt;
      if (!o.got && o.z < 4.5 && o.z > 3.3 && Math.abs(o.lane - s.lane) < 0.6) {
        o.got = true;
        s.combo++;
        s.score += 10 * (1 + Math.floor(s.combo / 5));
        burst(10);
      }
    }
    s.obs = s.obs.filter((o) => o.z > 2.6);
    s.orbs = s.orbs.filter((o) => o.z > 2.6 && !o.got);

    // ---------- Zeichnen ----------
    ctx.save();
    if (s.shake > 0) ctx.translate((Math.random() - 0.5) * 10 * s.shake, (Math.random() - 0.5) * 8 * s.shake);

    // Himmel
    const sky = ctx.createLinearGradient(0, 0, 0, hor);
    sky.addColorStop(0, 'rgb(9,8,22)');
    sky.addColorStop(1, amber.mix([32, 14, 42], acc2, 0.5));
    ctx.fillStyle = sky;
    ctx.fillRect(-20, -20, w + 40, hor + 20);

    // Sonne
    const sr = Math.min(w, h) * (0.17 + s.bass * 0.03), sy = hor - sr * 0.3;
    const sg = ctx.createLinearGradient(0, sy - sr, 0, sy + sr);
    sg.addColorStop(0, amber.rgba(acc, 1));
    sg.addColorStop(1, amber.rgba(acc2, 1));
    ctx.fillStyle = sg;
    ctx.beginPath();
    ctx.arc(cx, sy, sr, 0, Math.PI * 2);
    ctx.fill();
    ctx.fillStyle = 'rgb(20,13,32)';
    for (let i = 0; i < 5; i++) ctx.fillRect(cx - sr, sy + sr * (0.05 + i * 0.18), sr * 2, sr * 0.03 * (i + 1));

    // Berge aus dem Spektrum
    const nm = 40, bw = w / nm;
    ctx.fillStyle = 'rgb(20,13,32)';
    for (let i = 0; i < nm; i++) {
      s.mt[i] = amber.lerp(s.mt[i], frame.bands[Math.floor((i * frame.bands.length) / nm)] || 0, 0.2);
      const mh = 6 + s.mt[i] * h * 0.16;
      ctx.beginPath();
      ctx.moveTo(i * bw, hor + 1);
      ctx.lineTo(i * bw + bw / 2, hor - mh);
      ctx.lineTo((i + 1) * bw, hor + 1);
      ctx.fill();
    }

    // Boden und Straße in Streifen
    const m3 = s.dist % 3;
    for (let k = 0; k < 48; k++) {
      let z0 = k * 1.5 - m3;
      const z1 = (k + 1) * 1.5 - m3;
      if (z1 <= 2.4) continue;
      z0 = Math.max(z0, 2.4);
      const yb = Y(z0), yt = Y(z1), alt = k % 2 === 0;
      ctx.fillStyle = alt ? 'rgb(16,10,26)' : 'rgb(20,13,32)';
      ctx.fillRect(-20, yt, w + 40, yb - yt + 1);
      ctx.fillStyle = alt ? 'rgb(34,28,46)' : amber.mix([30, 24, 44], acc2, 0.12 + s.flash * 0.12);
      ctx.beginPath();
      ctx.moveTo(X(-1.5, z0), yb);
      ctx.lineTo(X(1.5, z0), yb);
      ctx.lineTo(X(1.5, z1), yt);
      ctx.lineTo(X(-1.5, z1), yt);
      ctx.fill();
      if (alt) {
        ctx.fillStyle = 'rgba(235,228,220,0.35)';
        for (const lx of [-0.5, 0.5]) {
          ctx.beginPath();
          ctx.moveTo(X(lx - 0.025, z0), yb);
          ctx.lineTo(X(lx + 0.025, z0), yb);
          ctx.lineTo(X(lx + 0.025, z1), yt);
          ctx.lineTo(X(lx - 0.025, z1), yt);
          ctx.fill();
        }
      }
    }
    // Neon-Ränder
    for (let pass = 0; pass < 2; pass++) {
      ctx.lineWidth = pass === 0 ? 6 : 2;
      ctx.strokeStyle = amber.rgba(pass === 0 ? acc2 : acc, pass === 0 ? 0.25 : 0.95);
      for (const lx of [-1.5, 1.5]) {
        ctx.beginPath();
        ctx.moveTo(X(lx, 2.4), Y(2.4));
        ctx.lineTo(X(lx, 70), Y(70));
        ctx.stroke();
      }
    }

    // Zeichenliste (hinten nach vorn): Säulen, Orbs, Hindernisse, Läufer
    const list = [];
    const m6 = s.dist % 6, base6 = Math.floor(s.dist / 6);
    for (let j = 1; j < 11; j++) {
      const z = j * 6 - m6;
      if (z < 2.6) continue;
      const id = base6 + j;
      const v = frame.bands[(id * 3) % 32] || 0;
      list.push({ z, f: () => {
        const sc = SC(z);
        for (const side of [-1, 1]) {
          const gx = X(side * 2.3, z), gy = Y(z);
          const ph = (0.9 + v * 2.4 + s.flash * 0.4) * laneW * sc;
          ctx.strokeStyle = amber.mix(acc, acc2, id % 2, 0.9);
          ctx.lineWidth = Math.max(1.5, 5 * sc);
          ctx.beginPath();
          ctx.moveTo(gx, gy);
          ctx.lineTo(gx, gy - ph);
          ctx.stroke();
          ctx.fillStyle = amber.rgba(acc, 0.95);
          ctx.beginPath();
          ctx.arc(gx, gy - ph, Math.max(2, 7 * sc), 0, Math.PI * 2);
          ctx.fill();
        }
      } });
    }
    for (const o of s.orbs) {
      list.push({ z: o.z, f: () => {
        const sc = SC(o.z), r = laneW * 0.22 * sc;
        const ox = X(o.lane, o.z), oy = Y(o.z) - laneW * (0.5 + 0.08 * Math.sin(s.t * 5 + o.z)) * sc;
        const g = ctx.createRadialGradient(ox, oy, 0, ox, oy, r * 2.4);
        g.addColorStop(0, amber.rgba(acc, 0.7));
        g.addColorStop(1, amber.rgba(acc, 0));
        ctx.fillStyle = g;
        ctx.fillRect(ox - r * 2.4, oy - r * 2.4, r * 4.8, r * 4.8);
        ctx.fillStyle = amber.mix(acc, [255, 255, 255], 0.35, 1);
        ctx.beginPath();
        ctx.arc(ox, oy, r, 0, Math.PI * 2);
        ctx.fill();
      } });
    }
    for (const o of s.obs) {
      list.push({ z: o.z, f: () => {
        const sc = SC(o.z), bwid = laneW * 0.78 * sc, bh = laneW * (o.wall ? 1.5 : 0.3) * sc;
        const ox = X(o.lane, o.z), oy = Y(o.z);
        const col = o.wall ? acc2 : acc;
        ctx.fillStyle = amber.mix([14, 10, 22], col, 0.22, 1);
        ctx.fillRect(ox - bwid / 2, oy - bh, bwid, bh);
        ctx.fillStyle = amber.rgba(col, 0.9);
        ctx.fillRect(ox - bwid / 2, oy - bh, bwid, Math.max(2, bh * 0.1));
        ctx.strokeStyle = amber.rgba(col, 1);
        ctx.lineWidth = Math.max(1.5, 3 * sc);
        ctx.strokeRect(ox - bwid / 2, oy - bh, bwid, bh);
        if (o.wall) {
          ctx.fillStyle = amber.rgba(col, 0.5);
          for (let i = 1; i < 4; i++) ctx.fillRect(ox - bwid / 2, oy - bh + (bh * i) / 4, bwid, Math.max(1, 2 * sc));
        }
      } });
    }
    list.push({ z: ZP, f: () => {
      const H = laneW * 1.3, u = H / 10;
      const px = pxScreen, fy = yp - jh * laneW * 1.1;
      // Schatten
      ctx.fillStyle = 'rgba(0,0,0,' + (0.4 - jh * 0.2) + ')';
      ctx.beginPath();
      ctx.ellipse(px, yp, u * (3 - jh), u * 0.8, 0, 0, Math.PI * 2);
      ctx.fill();
      // Glow
      const gl = ctx.createRadialGradient(px, fy - 5 * u, 0, px, fy - 5 * u, 8 * u);
      gl.addColorStop(0, amber.rgba(s.hurt > 0 ? acc2 : acc, 0.3 + s.hurt * 0.3));
      gl.addColorStop(1, amber.rgba(acc, 0));
      ctx.fillStyle = gl;
      ctx.fillRect(px - 8 * u, fy - 13 * u, 16 * u, 16 * u);

      const air = jh > 0.02;
      const hipY = fy - 4 * u;
      ctx.lineCap = 'round';
      // Beine
      for (const side of [-1, 1]) {
        const sw = air ? 0.8 : Math.sin(s.phase + (side > 0 ? 0 : Math.PI));
        ctx.strokeStyle = amber.rgba(acc2, 1);
        ctx.lineWidth = 1.8 * u;
        ctx.beginPath();
        ctx.moveTo(px + side * 1.0 * u, hipY);
        ctx.lineTo(px + side * 1.3 * u, fy - Math.max(0, sw) * 1.8 * u);
        ctx.stroke();
      }
      // Schal
      ctx.strokeStyle = amber.rgba(acc, 0.9);
      ctx.lineWidth = 1.1 * u;
      ctx.beginPath();
      ctx.moveTo(px, hipY - 4 * u);
      for (let i = 1; i <= 6; i++) {
        ctx.lineTo(px + Math.sin(s.t * 9 + i) * 0.8 * u * i * 0.5 - s.speed * 0.05 * u * 0, hipY - 4 * u + i * 0.9 * u);
      }
      ctx.stroke();
      // Oberkörper
      const tg = ctx.createLinearGradient(0, hipY - 4.4 * u, 0, hipY);
      tg.addColorStop(0, amber.rgba(acc, 1));
      tg.addColorStop(1, amber.rgba(acc2, 1));
      ctx.fillStyle = tg;
      ctx.fillRect(px - 2 * u, hipY - 4.2 * u, 4 * u, 4.4 * u);
      // Arme
      for (const side of [-1, 1]) {
        const sw = air ? -1 : Math.sin(s.phase + (side > 0 ? Math.PI : 0));
        ctx.strokeStyle = amber.rgba(acc, 1);
        ctx.lineWidth = 1.2 * u;
        ctx.beginPath();
        ctx.moveTo(px + side * 2.2 * u, hipY - 3.7 * u);
        ctx.lineTo(px + side * 2.8 * u, hipY - 1.0 * u - sw * 1.3 * u);
        ctx.stroke();
      }
      // Kopf
      ctx.fillStyle = amber.mix(acc, [255, 255, 255], 0.55, 1);
      ctx.beginPath();
      ctx.arc(px, hipY - 5.7 * u, 1.5 * u, 0, Math.PI * 2);
      ctx.fill();
      ctx.fillStyle = 'rgb(24,18,30)';
      ctx.beginPath();
      ctx.arc(px, hipY - 5.9 * u, 1.55 * u, Math.PI, Math.PI * 2);
      ctx.fill();
    } });
    list.sort((a, b) => b.z - a.z);
    for (const d of list) d.f();

    // Partikel
    for (let i = s.parts.length - 1; i >= 0; i--) {
      const p = s.parts[i];
      p.x += p.vx * dt;
      p.y += p.vy * dt;
      p.vy += 300 * dt;
      p.life -= dt * 1.4;
      if (p.life <= 0) { s.parts.splice(i, 1); continue; }
      ctx.fillStyle = amber.mix(acc, acc2, p.c, p.life);
      ctx.beginPath();
      ctx.arc(p.x, p.y, 1.5 + p.life * 2.5, 0, Math.PI * 2);
      ctx.fill();
    }
    ctx.restore();

    if (s.hurt > 0) {
      ctx.fillStyle = amber.rgba(acc2, s.hurt * 0.22);
      ctx.fillRect(0, 0, w, h);
    }

    // Anzeige
    const fs = Math.max(12, Math.min(26, h * 0.045));
    ctx.textAlign = 'left';
    ctx.font = 'bold ' + fs + 'px sans-serif';
    ctx.fillStyle = amber.rgba(acc, 0.95);
    ctx.fillText('PUNKTE ' + s.score, w * 0.04, h * 0.08);
    if (s.combo > 1) {
      ctx.font = 'bold ' + fs * 0.8 + 'px sans-serif';
      ctx.fillStyle = amber.rgba(acc2, 0.95);
      ctx.fillText('KOMBO x' + s.combo, w * 0.04, h * 0.08 + fs * 1.2);
    }
    ctx.textAlign = 'center';
    ctx.font = fs * 0.6 + 'px sans-serif';
    ctx.fillStyle = 'rgba(235,228,220,0.75)';
    ctx.fillText(playing ? (info.title || '') + (info.artist ? ' · ' + info.artist : '') : 'PAUSE', cx, h * 0.97);
  },
});
