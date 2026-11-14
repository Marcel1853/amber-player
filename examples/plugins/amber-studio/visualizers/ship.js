// Amber Studio: Amber Raumschiff – ein Mini-Shooter, der sich selbst spielt.
// Das Schiff weicht Asteroiden aus und schießt im Takt der Musik.
amber.visualizer({
  id: 'amber-ship',
  name: 'Amber Raumschiff',
  author: 'Marcel1853',
  description: 'Spiel-Optik: Ein Raumschiff weicht Asteroiden aus und schießt im Takt',

  setup({ state: s }) {
    Object.assign(s, { stars: Array.from({ length: 90 }, () => ({ x: Math.random(), y: Math.random(), z: 0.3 + Math.random() })),
      x: 0.5, tx: 0.5, ast: [], las: [], parts: [], score: 0, spawn: 1, shoot: 0, prev: 0, flame: 0, hurt: 0, t: 0 });
  },

  draw({ ctx, width: w, height: h, frame, info, state: s, amber }) {
    const dt = Math.min(frame.dt || 0.016, 0.05);
    const acc = info.accent, acc2 = info.accent2, play = frame.playing;
    const sy = h * 0.86, m = Math.min(w, h);
    s.t += dt;
    s.hurt = Math.max(0, s.hurt - dt * 2);
    const beat = play && frame.bass > 0.42 && frame.bass - s.prev > 0.06;
    s.prev = frame.bass;

    // Spawnen
    s.spawn -= dt;
    if (play && s.spawn <= 0 && s.ast.length < 12) {
      s.spawn = Math.max(0.3, 0.9 - frame.level * 0.5) * (0.6 + Math.random() * 0.8);
      s.ast.push({ x: 0.08 + Math.random() * 0.84, y: -0.08, r: 0.025 + Math.random() * 0.03, vy: 0.14 + Math.random() * 0.12 + frame.level * 0.12, rot: Math.random() * 6, hp: 1, seed: Math.random() * 3 });
    }

    // Ausweichen: der freieste Platz in der Schiffsspur
    let threat = false;
    for (const a of s.ast) if (a.y > 0.25 && a.y < 0.95 && Math.abs(a.x - s.x) < a.r + 0.07) threat = true;
    if (threat) {
      let best = s.x, bestScore = -1;
      for (let c = 0.06; c <= 0.94; c += 0.04) {
        let d = 1;
        for (const a of s.ast) if (a.y > 0.2 && a.y < 0.98) d = Math.min(d, Math.abs(a.x - c) - a.r - 0.05 + (0.98 - a.y) * 0.1);
        const sc = d - Math.abs(c - s.x) * 0.05;
        if (sc > bestScore) { bestScore = sc; best = c; }
      }
      s.tx = best;
    }
    s.x += (s.tx - s.x) * Math.min(1, dt * 7);
    s.flame = amber.lerp(s.flame, frame.level, 0.2);

    // Schießen
    s.shoot -= dt;
    if (play && (beat || s.shoot <= 0)) {
      s.shoot = 0.35;
      s.las.push({ x: s.x, y: sy / h - 0.05 });
    }

    // Bewegen und Treffer
    for (const l of s.las) l.y -= dt * 1.4;
    s.las = s.las.filter((l) => l.y > -0.05);
    for (const a of s.ast) {
      a.y += dt * a.vy * (0.7 + frame.level);
      a.rot += dt * a.vy * 4;
      for (const l of s.las) {
        if (!l.dead && Math.abs(l.x - a.x) < a.r + 0.01 && Math.abs(l.y - a.y) < a.r + 0.02) {
          l.dead = true; a.dead = true; s.score += 10;
          for (let i = 0; i < 14 && s.parts.length < 160; i++) {
            const an = Math.random() * 6.28, sp = 40 + Math.random() * 140;
            s.parts.push({ x: a.x * w, y: a.y * h, vx: Math.cos(an) * sp, vy: Math.sin(an) * sp, life: 1, c: Math.random() });
          }
        }
      }
      if (!a.dead && Math.abs(a.x - s.x) < a.r + 0.03 && Math.abs(a.y - sy / h) < a.r + 0.04) {
        a.dead = true; s.hurt = 1; s.score = Math.max(0, s.score - 15);
      }
    }
    s.ast = s.ast.filter((a) => !a.dead && a.y < 1.1);
    s.las = s.las.filter((l) => !l.dead);

    // Zeichnen
    ctx.fillStyle = 'rgb(7,7,13)';
    ctx.fillRect(0, 0, w, h);
    for (const st of s.stars) {
      st.y += dt * 0.08 * st.z * (0.6 + frame.level * 2);
      if (st.y > 1) { st.y = 0; st.x = Math.random(); }
      ctx.fillStyle = 'rgba(235,228,220,' + (0.2 + st.z * 0.5) + ')';
      ctx.fillRect(st.x * w, st.y * h, 1 + st.z, 1 + st.z * 3);
    }
    for (const a of s.ast) {
      const r = a.r * m;
      ctx.save();
      ctx.translate(a.x * w, a.y * h);
      ctx.rotate(a.rot);
      ctx.fillStyle = amber.mix([40, 34, 44], acc2, 0.25, 1);
      ctx.strokeStyle = amber.rgba(acc2, 0.95);
      ctx.lineWidth = 2;
      ctx.beginPath();
      for (let k = 0; k < 9; k++) {
        const an = (k / 9) * Math.PI * 2, rr = r * (0.8 + 0.25 * Math.sin(k * 2.7 + a.seed * 5));
        if (k === 0) ctx.moveTo(Math.cos(an) * rr, Math.sin(an) * rr); else ctx.lineTo(Math.cos(an) * rr, Math.sin(an) * rr);
      }
      ctx.closePath();
      ctx.fill();
      ctx.stroke();
      ctx.restore();
    }
    ctx.strokeStyle = amber.rgba(acc, 1);
    ctx.lineWidth = 3;
    ctx.lineCap = 'round';
    for (const l of s.las) {
      ctx.beginPath();
      ctx.moveTo(l.x * w, l.y * h);
      ctx.lineTo(l.x * w, l.y * h + 14);
      ctx.stroke();
    }
    // Schiff
    const px = s.x * w, u = m * 0.02;
    ctx.fillStyle = amber.rgba(acc2, 0.9);
    ctx.beginPath();
    ctx.moveTo(px - u * 0.7, sy + u * 1.8);
    ctx.lineTo(px, sy + u * (3.2 + s.flame * 4 + Math.random()));
    ctx.lineTo(px + u * 0.7, sy + u * 1.8);
    ctx.fill();
    ctx.fillStyle = s.hurt > 0 ? 'rgb(255,255,255)' : amber.mix(acc, [255, 255, 255], 0.3, 1);
    ctx.beginPath();
    ctx.moveTo(px, sy - u * 2.4);
    ctx.lineTo(px + u * 1.7, sy + u * 1.8);
    ctx.lineTo(px, sy + u * 1);
    ctx.lineTo(px - u * 1.7, sy + u * 1.8);
    ctx.closePath();
    ctx.fill();

    for (let i = s.parts.length - 1; i >= 0; i--) {
      const p = s.parts[i];
      p.x += p.vx * dt; p.y += p.vy * dt; p.life -= dt * 1.6;
      if (p.life <= 0) { s.parts.splice(i, 1); continue; }
      ctx.fillStyle = amber.mix(acc, acc2, p.c, p.life);
      ctx.beginPath();
      ctx.arc(p.x, p.y, 1 + p.life * 3, 0, Math.PI * 2);
      ctx.fill();
    }
    if (s.hurt > 0) { ctx.fillStyle = amber.rgba(acc2, s.hurt * 0.2); ctx.fillRect(0, 0, w, h); }

    const fs = Math.max(12, Math.min(26, h * 0.045));
    ctx.textAlign = 'left';
    ctx.font = 'bold ' + fs + 'px sans-serif';
    ctx.fillStyle = amber.rgba(acc, 0.95);
    ctx.fillText('PUNKTE ' + s.score, w * 0.04, h * 0.08);
    ctx.textAlign = 'center';
    ctx.font = fs * 0.6 + 'px sans-serif';
    ctx.fillStyle = 'rgba(235,228,220,0.75)';
    ctx.fillText(play ? (info.title || '') + (info.artist ? ' · ' + info.artist : '') : 'PAUSE', w / 2, h * 0.97);
  },
});
