// Amber Studio: Spiralgalaxie, die Sterne reagieren auf ihr Frequenzband.
amber.visualizer({
  id: 'amber-galaxy',
  name: 'Amber Galaxie',
  author: 'Marcel1853',
  description: 'Rotierende Spiralgalaxie, Sterne leuchten mit ihrem Frequenzband',

  setup({ state }) {
    state.rot = 0;
    state.bass = 0;
    state.s = Array.from({ length: 320 }, () => {
      const r = Math.pow(Math.random(), 0.7);
      return { r, arm: Math.floor(Math.random() * 3), off: (Math.random() - 0.5) * 0.5, c: Math.random() };
    });
  },

  draw({ ctx, width: w, height: h, frame, info, state, amber }) {
    const cx = w / 2, cy = h / 2, m = Math.min(w, h) * 0.46;
    state.bass = amber.lerp(state.bass, frame.bass, 0.2);
    state.rot += frame.dt * (frame.playing ? 0.12 + frame.level * 0.3 : 0.03);
    ctx.fillStyle = 'rgba(7,7,12,0.3)';
    ctx.fillRect(0, 0, w, h);

    const core = ctx.createRadialGradient(cx, cy, 0, cx, cy, m * (0.25 + state.bass * 0.15));
    core.addColorStop(0, amber.rgba(info.accent, 0.55));
    core.addColorStop(1, amber.rgba(info.accent2, 0));
    ctx.fillStyle = core;
    ctx.fillRect(0, 0, w, h);

    for (const s of state.s) {
      const e = frame.bands[Math.min(frame.bands.length - 1, Math.floor(s.r * frame.bands.length))] || 0;
      const ang = s.arm * (Math.PI * 2 / 3) + s.r * 4.2 + s.off + state.rot / (0.25 + s.r);
      const rr = s.r * m * (1 + state.bass * 0.12);
      ctx.fillStyle = amber.mix(info.accent, info.accent2, s.c, 0.25 + e * 0.75);
      ctx.beginPath();
      ctx.arc(cx + Math.cos(ang) * rr, cy + Math.sin(ang) * rr * 0.62, 0.8 + e * 3 + (1 - s.r) * 1.2, 0, Math.PI * 2);
      ctx.fill();
    }
  },
});
