// Nebel: weiche Farbwolken, die mit Bass, Mitten und Höhen wachsen und treiben.
amber.visualizer({
  id: 'nebel',
  name: 'Nebel',
  author: 'Amber Player',
  description: 'Fließende Farbwolken, die auf Bass, Mitten und Höhen reagieren',

  setup({ ctx, width, height, state }) {
    state.t = 0;
    state.energy = [0, 0, 0];
    ctx.fillStyle = '#080706';
    ctx.fillRect(0, 0, width, height);
  },

  draw({ ctx, width: w, height: h, frame, info, state, amber }) {
    ctx.globalCompositeOperation = 'source-over';
    ctx.fillStyle = 'rgba(8, 7, 6, 0.24)';
    ctx.fillRect(0, 0, w, h);

    const e = state.energy;
    [frame.bass, frame.mid, frame.treble].forEach((v, i) => (e[i] = amber.lerp(e[i], v, 0.12)));
    state.t += frame.dt * (0.15 + e[0] * 0.6);

    const unit = Math.min(w, h);
    const blobs = [
      [0.0, 1.0, info.accent, 0],
      [2.1, 0.8, info.accent2, 0],
      [4.2, 1.2, info.accent, 1],
      [1.3, 0.6, info.accent2, 1],
      [3.3, 0.9, [255, 236, 214], 2],
      [5.4, 1.1, info.accent, 2],
    ];
    ctx.globalCompositeOperation = 'lighter';
    for (const [phase, speed, color, band] of blobs) {
      const t = state.t * speed + phase;
      const x = w / 2 + Math.cos(t * 0.7) * w * 0.32 + Math.sin(t * 1.3) * w * 0.06;
      const y = h / 2 + Math.sin(t * 0.9) * h * 0.28;
      const r = unit * (0.12 + e[band] * 0.35);
      const g = ctx.createRadialGradient(x, y, 0, x, y, r);
      g.addColorStop(0, amber.rgba(color, 0.13 + e[band] * 0.22));
      g.addColorStop(0.5, amber.rgba(color, 0.05 + e[band] * 0.09));
      g.addColorStop(1, amber.rgba(color, 0));
      ctx.fillStyle = g;
      ctx.beginPath();
      ctx.arc(x, y, r, 0, Math.PI * 2);
      ctx.fill();
    }
    ctx.globalCompositeOperation = 'source-over';
  },
});
