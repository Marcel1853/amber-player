// Amber Aurora FX: Partikel, die auf Bass und Lautstärke reagieren.
amber.visualizer({
  id: 'particle-pulse',
  name: 'Particle Pulse',
  author: 'Marcel1853',
  description: 'Bewegliche Partikel, die auf Bass und Lautstärke reagieren',

  setup({ state }) {
    state.p = [];
    state.bass = 0;
    state.prevBass = 0;
    state.ring = 0;
  },

  draw({ ctx, width: w, height: h, frame, info, state, amber }) {
    const cx = w / 2;
    const cy = h / 2;
    const maxP = 300; // mobilfreundlich
    state.bass = amber.lerp(state.bass, frame.bass, 0.3);

    ctx.fillStyle = 'rgba(4, 5, 10, 0.28)';
    ctx.fillRect(0, 0, w, h);

    // Bass-Schlag erkennen: Partikel ausstoßen
    const hit = frame.playing && frame.bass > 0.45 && frame.bass - state.prevBass > 0.08;
    state.prevBass = frame.bass;
    const spawn = hit ? 24 : (frame.playing ? Math.floor(frame.level * 3) : 0);
    for (let i = 0; i < spawn && state.p.length < maxP; i++) {
      const a = Math.random() * Math.PI * 2;
      const sp = (hit ? 160 : 50) * (0.4 + Math.random()) * (0.5 + frame.level);
      state.p.push({ x: cx, y: cy, vx: Math.cos(a) * sp, vy: Math.sin(a) * sp, life: 1, c: Math.random() });
    }
    if (hit) state.ring = 1;

    // Kern
    const r = 18 + state.bass * 70;
    ctx.fillStyle = amber.rgba(info.accent, 0.35 + state.bass * 0.4);
    ctx.beginPath();
    ctx.arc(cx, cy, r, 0, Math.PI * 2);
    ctx.fill();

    // Schockring
    if (state.ring > 0) {
      ctx.strokeStyle = amber.rgba(info.accent2, state.ring * 0.6);
      ctx.lineWidth = 2;
      ctx.beginPath();
      ctx.arc(cx, cy, r + (1 - state.ring) * Math.min(w, h) * 0.4, 0, Math.PI * 2);
      ctx.stroke();
      state.ring = Math.max(0, state.ring - frame.dt * 1.6);
    }

    // Partikel
    const dt = frame.dt;
    for (let i = state.p.length - 1; i >= 0; i--) {
      const q = state.p[i];
      q.x += q.vx * dt;
      q.y += q.vy * dt;
      q.vx *= 0.985;
      q.vy *= 0.985;
      q.life -= dt * 0.45;
      if (q.life <= 0) {
        state.p.splice(i, 1);
        continue;
      }
      ctx.fillStyle = amber.mix(info.accent, info.accent2, q.c, q.life * 0.9);
      ctx.beginPath();
      ctx.arc(q.x, q.y, 1.5 + q.life * 2.5, 0, Math.PI * 2);
      ctx.fill();
    }
  },
});
