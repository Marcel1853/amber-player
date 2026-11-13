// Balken: Frequenzbänder als Säulen mit Spiegelung und langsam fallenden Spitzen.
amber.visualizer({
  id: 'balken',
  name: 'Balken',
  author: 'Amber Player',
  description: 'Frequenzbalken mit Spiegelung und fallenden Spitzen',

  setup({ state }) {
    state.v = [];
    state.peak = [];
  },

  draw({ ctx, width: w, height: h, frame, info, state, amber }) {
    const n = frame.bands.length;
    const bg = ctx.createLinearGradient(0, 0, 0, h);
    bg.addColorStop(0, '#120f0d');
    bg.addColorStop(1, '#070606');
    ctx.fillStyle = bg;
    ctx.fillRect(0, 0, w, h);

    const gap = Math.max(2, w / n / 5);
    const barW = (w * 0.86) / n - gap;
    const x0 = (w - n * (barW + gap)) / 2;
    const base = h * 0.66;
    const maxH = h * 0.52;

    const grad = ctx.createLinearGradient(0, base - maxH, 0, base);
    grad.addColorStop(0, amber.rgba(info.accent2));
    grad.addColorStop(1, amber.rgba(info.accent));

    for (let i = 0; i < n; i++) {
      const target = frame.bands[i];
      const v = state.v[i] || 0;
      state.v[i] = target > v ? v + (target - v) * 0.6 : v * Math.pow(0.04, frame.dt);
      state.peak[i] = Math.max(state.v[i], (state.peak[i] || 0) - frame.dt * 0.35);

      const x = x0 + i * (barW + gap);
      const bh = Math.max(2, state.v[i] * maxH);
      ctx.fillStyle = grad;
      ctx.fillRect(x, base - bh, barW, bh);

      // Spiegelung auf dem „Boden“
      ctx.globalAlpha = 0.16;
      ctx.fillRect(x, base + 4, barW, bh * 0.35);
      ctx.globalAlpha = 1;

      // Spitze
      ctx.fillStyle = 'rgba(255, 245, 230, 0.85)';
      ctx.fillRect(x, base - state.peak[i] * maxH - 6, barW, 3);
    }
  },
});
