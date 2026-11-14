// Amber Studio: Sonnensystem – Planeten kreisen, ihre Größe folgt den Frequenzbändern.
amber.visualizer({
  id: 'amber-solar',
  name: 'Amber Sonnensystem',
  author: 'Marcel1853',
  description: 'Planeten auf Umlaufbahnen, Größe und Leuchten folgen dem Spektrum',

  setup({ state }) {
    state.t = 0;
    state.bass = 0;
    state.v = new Array(8).fill(0);
    state.ph = Array.from({ length: 8 }, () => Math.random() * 6.28);
  },

  draw({ ctx, width: w, height: h, frame, info, state, amber }) {
    state.t += frame.dt * (frame.playing ? 1 : 0.2);
    state.bass = amber.lerp(state.bass, frame.bass, 0.25);
    ctx.fillStyle = 'rgb(8,8,13)';
    ctx.fillRect(0, 0, w, h);

    const cx = w / 2, cy = h / 2, m = Math.min(w, h) * 0.46;
    const g = ctx.createRadialGradient(cx, cy, 0, cx, cy, m * 0.3);
    g.addColorStop(0, amber.rgba(info.accent, 0.9));
    g.addColorStop(1, amber.rgba(info.accent2, 0));
    ctx.fillStyle = g;
    ctx.beginPath();
    ctx.arc(cx, cy, m * (0.2 + state.bass * 0.05), 0, Math.PI * 2);
    ctx.fill();

    const list = [];
    for (let i = 0; i < 8; i++) {
      const orb = m * (0.22 + i * 0.11);
      const e = amber.range(frame.bands, i / 8, (i + 1) / 8);
      state.v[i] = amber.lerp(state.v[i], e, 0.2);
      ctx.strokeStyle = 'rgba(235,228,220,0.07)';
      ctx.lineWidth = 1;
      ctx.beginPath();
      ctx.ellipse(cx, cy, orb, orb * 0.42, 0, 0, Math.PI * 2);
      ctx.stroke();
      const a = state.ph[i] + state.t * (0.9 / (1 + i * 0.5));
      list.push({ x: cx + Math.cos(a) * orb, y: cy + Math.sin(a) * orb * 0.42, d: Math.sin(a), r: 3 + state.v[i] * m * 0.07, c: i / 7, v: state.v[i] });
    }
    list.sort((p, q) => p.d - q.d);
    for (const p of list) {
      ctx.fillStyle = amber.mix(info.accent2, info.accent, p.c, 0.5 + p.v * 0.5);
      ctx.beginPath();
      ctx.arc(p.x, p.y, p.r * (0.85 + 0.15 * p.d), 0, Math.PI * 2);
      ctx.fill();
    }
  },
});
