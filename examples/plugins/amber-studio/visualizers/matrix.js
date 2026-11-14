// Amber Studio: Matrix-Regen aus Zeichen, schneller bei lauter Musik.
amber.visualizer({
  id: 'amber-matrix',
  name: 'Amber Matrix',
  author: 'Marcel1853',
  description: 'Fallende Zeichenspalten in den Akzentfarben, Bass-Schläge lassen sie aufblitzen',

  setup({ state }) {
    state.cols = [];
    state.flash = 0;
    state.prev = 0;
  },

  draw({ ctx, width: w, height: h, frame, info, state, amber }) {
    const cs = Math.max(12, Math.floor(w / 48));
    const n = Math.floor(w / cs);
    while (state.cols.length < n) state.cols.push({ y: Math.random() * h, v: 0.5 + Math.random() });
    const hit = frame.playing && frame.bass > 0.45 && frame.bass - state.prev > 0.07;
    state.prev = frame.bass;
    if (hit) state.flash = 1;
    state.flash = Math.max(0, state.flash - frame.dt * 3);

    ctx.fillStyle = 'rgba(6,9,8,0.2)';
    ctx.fillRect(0, 0, w, h);
    ctx.font = cs + 'px monospace';
    ctx.textAlign = 'center';
    const chars = '0123456789ABCDEFXYZ+*#<>';
    for (let i = 0; i < n; i++) {
      const c = state.cols[i];
      const e = frame.bands[Math.floor((i * frame.bands.length) / n)] || 0;
      c.y += frame.dt * h * (0.15 + e * 0.9 + frame.level * 0.4) * c.v * (frame.playing ? 1 : 0.2);
      if (c.y > h + cs * 6) { c.y = -cs * Math.random() * 10; c.v = 0.5 + Math.random(); }
      const x = i * cs + cs / 2;
      ctx.fillStyle = amber.rgba(info.accent2, 0.55);
      ctx.fillText(chars[Math.floor(Math.random() * chars.length)], x, c.y - cs);
      ctx.fillStyle = amber.mix(info.accent, [255, 255, 255], state.flash * 0.7, 0.95);
      ctx.fillText(chars[Math.floor(Math.random() * chars.length)], x, c.y);
    }
  },
});
