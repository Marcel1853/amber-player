// Amber Studio: Regen am Fenster – Intensität folgt der Musik, Bass-Schläge erzeugen Wellen.
amber.visualizer({
  id: 'amber-rain',
  name: 'Amber Regen',
  author: 'Marcel1853',
  description: 'Regentropfen, die mit der Lautstärke stärker werden, und Wellen bei Bass-Schlägen',

  setup({ state }) {
    state.d = Array.from({ length: 160 }, () => ({ x: Math.random(), y: Math.random(), s: 0.6 + Math.random() * 0.8 }));
    state.rip = [];
    state.prev = 0;
    state.inten = 0.3;
  },

  draw({ ctx, width: w, height: h, frame, info, state, amber }) {
    state.inten = amber.lerp(state.inten, frame.playing ? 0.35 + frame.level * 0.9 : 0.2, 0.05);
    const hit = frame.playing && frame.bass > 0.45 && frame.bass - state.prev > 0.07;
    state.prev = frame.bass;
    ctx.fillStyle = 'rgba(10,12,18,0.4)';
    ctx.fillRect(0, 0, w, h);

    const floor = h * 0.86;
    ctx.lineWidth = 1.2;
    const active = Math.floor(state.d.length * amber.clamp(state.inten, 0.2, 1));
    for (let i = 0; i < active; i++) {
      const d = state.d[i];
      d.y += frame.dt * 0.9 * d.s * (0.7 + state.inten);
      const y = d.y * floor;
      ctx.strokeStyle = amber.rgba(info.accent, 0.25 + d.s * 0.3);
      ctx.beginPath();
      ctx.moveTo(d.x * w, y - 14 * d.s);
      ctx.lineTo(d.x * w, y);
      ctx.stroke();
      if (d.y >= 1) {
        if (state.rip.length < 40) state.rip.push({ x: d.x * w, r: 2, a: 0.5 });
        d.y = 0;
        d.x = Math.random();
      }
    }
    if (hit) for (let k = 0; k < 3; k++) state.rip.push({ x: w * (0.2 + Math.random() * 0.6), r: 4, a: 0.9 });

    ctx.fillStyle = 'rgba(18,18,24,0.7)';
    ctx.fillRect(0, floor, w, h - floor);
    for (let i = state.rip.length - 1; i >= 0; i--) {
      const r = state.rip[i];
      r.r += frame.dt * 90;
      r.a -= frame.dt * 0.6;
      if (r.a <= 0) { state.rip.splice(i, 1); continue; }
      ctx.strokeStyle = amber.rgba(info.accent2, r.a);
      ctx.beginPath();
      ctx.ellipse(r.x, floor + (h - floor) * 0.3, r.r, r.r * 0.25, 0, 0, Math.PI * 2);
      ctx.stroke();
    }
  },
});
