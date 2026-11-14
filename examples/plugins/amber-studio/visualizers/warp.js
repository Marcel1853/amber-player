// Amber Studio: Sternenreise – Sterne ziehen Lichtspuren, die Geschwindigkeit folgt dem Pegel.
amber.visualizer({
  id: 'amber-warp',
  name: 'Amber Sternenreise',
  author: 'Marcel1853',
  description: 'Sterne fliegen als Lichtspuren auf dich zu, schneller bei lauter Musik',

  setup({ state }) {
    state.s = Array.from({ length: 220 }, () => ({ x: Math.random() * 2 - 1, y: Math.random() * 2 - 1, z: Math.random(), c: Math.random() }));
    state.v = 0.1;
  },

  draw({ ctx, width: w, height: h, frame, info, state, amber }) {
    const cx = w / 2, cy = h / 2, k = Math.max(w, h) * 0.45;
    state.v = amber.lerp(state.v, frame.playing ? 0.12 + frame.level * 0.9 + frame.bass * 0.5 : 0.04, 0.06);
    ctx.fillStyle = 'rgba(8,8,13,0.35)';
    ctx.fillRect(0, 0, w, h);
    ctx.lineCap = 'round';
    for (const s of state.s) {
      const zPrev = s.z;
      s.z -= frame.dt * state.v;
      if (s.z <= 0.03) { s.x = Math.random() * 2 - 1; s.y = Math.random() * 2 - 1; s.z = 1; continue; }
      const x1 = cx + (s.x / s.z) * k * 0.3, y1 = cy + (s.y / s.z) * k * 0.3;
      const x0 = cx + (s.x / zPrev) * k * 0.3, y0 = cy + (s.y / zPrev) * k * 0.3;
      ctx.strokeStyle = amber.mix(info.accent2, info.accent, s.c, 1 - s.z);
      ctx.lineWidth = 0.6 + (1 - s.z) * 2.2;
      ctx.beginPath();
      ctx.moveTo(x0, y0);
      ctx.lineTo(x1, y1);
      ctx.stroke();
    }
  },
});
