// Amber Studio: Kaleidoskop – das Spektrum wird achtfach gespiegelt.
amber.visualizer({
  id: 'amber-kaleido',
  name: 'Amber Kaleidoskop',
  author: 'Marcel1853',
  description: 'Achtfach gespiegelte Muster, geformt vom Spektrum',

  setup({ state }) {
    state.t = 0;
    state.v = new Array(32).fill(0);
  },

  draw({ ctx, width: w, height: h, frame, info, state, amber }) {
    const m = Math.min(w, h), seg = 8;
    state.t += frame.dt * (frame.playing ? 0.25 + frame.level * 0.6 : 0.05);
    ctx.fillStyle = 'rgba(10,10,14,0.18)';
    ctx.fillRect(0, 0, w, h);
    for (let i = 0; i < 32; i++) state.v[i] = amber.lerp(state.v[i], frame.bands[Math.floor((i * frame.bands.length) / 32)] || 0, 0.25);

    ctx.save();
    ctx.translate(w / 2, h / 2);
    for (let s = 0; s < seg * 2; s++) {
      ctx.save();
      ctx.rotate((s >> 1) * (Math.PI * 2 / seg) + state.t * 0.2);
      if (s & 1) ctx.scale(1, -1);
      for (let i = 0; i < 32; i++) {
        const f = i / 31, v = state.v[i];
        const r = m * (0.04 + f * 0.42);
        const a = Math.sin(state.t * (1 + f) + i) * 0.22 * (0.4 + v);
        ctx.fillStyle = amber.mix(info.accent, info.accent2, f, 0.15 + v * 0.7);
        ctx.beginPath();
        ctx.arc(Math.cos(a) * r, Math.sin(a) * r + v * m * 0.04, 1.5 + v * m * 0.03, 0, Math.PI * 2);
        ctx.fill();
      }
      ctx.restore();
    }
    ctx.restore();
  },
});
