// Teil des Beispiel-Pakets „Nachtfahrt“: wehende Lichtbänder wie ein Polarlicht.
// Die Höhe der Bänder folgt dem Spektrum, die Farben der Akzentfarbe.
amber.visualizer({
  id: 'polarlicht',
  name: 'Polarlicht',
  author: 'Marcel1853',
  description: 'Wehende Lichtbänder, die mit der Musik leuchten',

  setup({ state }) {
    state.t = 0;
    state.level = 0;
  },

  draw({ ctx, width: w, height: h, frame, info, state, amber }) {
    state.t += frame.dt * (frame.playing ? 1 : 0.25);
    state.level = amber.lerp(state.level, frame.level, 0.08);

    // halbdurchsichtig übermalen → weiche Spuren
    ctx.fillStyle = 'rgba(4, 6, 12, 0.22)';
    ctx.fillRect(0, 0, w, h);

    const bands = 5;
    for (let b = 0; b < bands; b++) {
      const energy = amber.range(frame.bands, b / bands, (b + 1) / bands);
      const base = h * (0.25 + b * 0.1);
      const color = amber.mix(info.accent, info.accent2, b / (bands - 1), 0.18 + energy * 0.5);
      ctx.beginPath();
      ctx.moveTo(0, h);
      for (let x = 0; x <= w; x += 12) {
        const wave = Math.sin(x * 0.006 + state.t * (0.6 + b * 0.15) + b) * (30 + energy * 120);
        const ripple = Math.sin(x * 0.02 - state.t * 2 + b * 2) * 8 * state.level;
        ctx.lineTo(x, base + wave + ripple);
      }
      ctx.lineTo(w, h);
      ctx.closePath();
      ctx.fillStyle = color;
      ctx.fill();
    }
  },
});
