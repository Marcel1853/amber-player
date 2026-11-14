// Amber Studio: Spirograph – Formen, die sich mit Bass und Pegel verändern.
amber.visualizer({
  id: 'amber-spiro',
  name: 'Amber Spirograph',
  author: 'Marcel1853',
  description: 'Spirograph-Kurven, deren Form vom Klang gesteuert wird',

  setup({ state }) {
    state.t = 0;
    state.rot = 0;
    state.bass = 0;
  },

  draw({ ctx, width: w, height: h, frame, info, state, amber }) {
    state.t += frame.dt * (frame.playing ? 1 : 0.2);
    state.rot += frame.dt * 0.2;
    state.bass = amber.lerp(state.bass, frame.bass, 0.2);
    ctx.fillStyle = 'rgba(10,10,14,0.14)';
    ctx.fillRect(0, 0, w, h);

    const cx = w / 2, cy = h / 2, m = Math.min(w, h) * 0.4;
    const R = 1, r = 0.28 + 0.12 * Math.sin(state.t * 0.13), d = 0.6 + state.bass * 0.7 + frame.level * 0.3;
    const k = (R - r) / r;
    ctx.beginPath();
    const N = 520;
    for (let i = 0; i <= N; i++) {
      const a = (i / N) * Math.PI * 2 * 9;
      const x = (R - r) * Math.cos(a) + d * r * Math.cos(k * a);
      const y = (R - r) * Math.sin(a) - d * r * Math.sin(k * a);
      const rx = x * Math.cos(state.rot) - y * Math.sin(state.rot);
      const ry = x * Math.sin(state.rot) + y * Math.cos(state.rot);
      if (i === 0) ctx.moveTo(cx + rx * m, cy + ry * m); else ctx.lineTo(cx + rx * m, cy + ry * m);
    }
    ctx.lineWidth = 1.5 + frame.level * 2;
    ctx.strokeStyle = amber.mix(info.accent, info.accent2, 0.5 + 0.5 * Math.sin(state.t * 0.5), 0.9);
    ctx.stroke();
  },
});
