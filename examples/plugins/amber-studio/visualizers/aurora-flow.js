// Amber Aurora FX: fließende Lichtwellen in den Akzentfarben des Players.
amber.visualizer({
  id: 'aurora-flow',
  name: 'Aurora Flow',
  author: 'Marcel1853',
  description: 'Fließende Lichtwellen mit dynamischen Farbverläufen',

  setup({ state }) {
    state.t = 0;
    state.level = 0;
    state.bass = 0;
  },

  draw({ ctx, width: w, height: h, frame, info, state, amber }) {
    state.t += frame.dt * (frame.playing ? 1 : 0.2);
    state.level = amber.lerp(state.level, frame.level, 0.1);
    state.bass = amber.lerp(state.bass, frame.bass, 0.2);

    ctx.fillStyle = 'rgba(5, 6, 14, 0.2)';
    ctx.fillRect(0, 0, w, h);

    const layers = 6;
    for (let l = 0; l < layers; l++) {
      const energy = amber.range(frame.bands, l / layers, (l + 1) / layers);
      const base = h * (0.3 + l * 0.08);
      const amp = 25 + energy * 130 + state.bass * 40;
      const grad = ctx.createLinearGradient(0, base - amp, 0, h);
      grad.addColorStop(0, amber.mix(info.accent, info.accent2, l / (layers - 1), 0.1 + energy * 0.5));
      grad.addColorStop(1, amber.rgba(info.accent2, 0));
      ctx.fillStyle = grad;
      ctx.beginPath();
      ctx.moveTo(0, h);
      for (let x = 0; x <= w; x += 10) {
        const y = base
          + Math.sin(x * 0.005 + state.t * (0.5 + l * 0.12) + l * 1.3) * amp
          + Math.sin(x * 0.013 - state.t * 1.4 + l) * amp * 0.25
          + Math.sin(x * 0.03 + state.t * 2.5) * 6 * state.level;
        ctx.lineTo(x, y);
      }
      ctx.lineTo(w, h);
      ctx.closePath();
      ctx.fill();
    }
  },
});
