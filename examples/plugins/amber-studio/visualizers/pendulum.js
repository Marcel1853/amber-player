// Amber Studio: Pendelwelle – 16 Pendel mit leicht verschiedener Frequenz bilden Muster.
amber.visualizer({
  id: 'amber-pendulum',
  name: 'Amber Pendelwelle',
  author: 'Marcel1853',
  description: '16 Pendel schwingen mit verschiedenen Frequenzen und bilden wandernde Wellen',

  setup({ state }) {
    state.t = 0;
    state.v = new Array(16).fill(0);
    state.bass = 0;
  },

  draw({ ctx, width: w, height: h, frame, info, state, amber }) {
    state.t += frame.dt * (frame.playing ? 1 : 0.25);
    state.bass = amber.lerp(state.bass, frame.bass, 0.3);
    ctx.fillStyle = 'rgb(13,12,16)';
    ctx.fillRect(0, 0, w, h);

    const n = 16, top = h * 0.12, L = h * 0.62;
    ctx.fillStyle = amber.rgba(info.accent, 0.4 + state.bass * 0.5);
    ctx.fillRect(w * 0.05, top - 5, w * 0.9, 5);
    for (let i = 0; i < n; i++) {
      const x0 = w * (0.1 + (0.8 * i) / (n - 1));
      const f = (24 + i) / 60; // Schwingungen pro Sekunde
      const a = Math.sin(state.t * Math.PI * 2 * f) * 0.5;
      const bx = x0 + Math.sin(a) * L * 0.55, by = top + Math.cos(a) * L * 0.9;
      const e = amber.range(frame.bands, i / n, (i + 1) / n);
      state.v[i] = amber.lerp(state.v[i], e, 0.25);
      const col = amber.mix(info.accent, info.accent2, i / (n - 1), 1);
      ctx.strokeStyle = 'rgba(235,228,220,0.25)';
      ctx.lineWidth = 1;
      ctx.beginPath();
      ctx.moveTo(x0, top);
      ctx.lineTo(bx, by);
      ctx.stroke();
      const r = Math.max(6, w * 0.014) * (1 + state.v[i] * 1.2);
      const g = ctx.createRadialGradient(bx, by, 0, bx, by, r * 2.2);
      g.addColorStop(0, col);
      g.addColorStop(1, amber.mix(info.accent, info.accent2, i / (n - 1), 0));
      ctx.fillStyle = g;
      ctx.beginPath();
      ctx.arc(bx, by, r * 2.2, 0, Math.PI * 2);
      ctx.fill();
      ctx.fillStyle = col;
      ctx.beginPath();
      ctx.arc(bx, by, r, 0, Math.PI * 2);
      ctx.fill();
    }
  },
});
