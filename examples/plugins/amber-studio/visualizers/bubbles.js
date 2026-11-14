// Amber Studio: Seifenblasen steigen auf, Bass-Schläge erzeugen Schübe.
amber.visualizer({
  id: 'amber-bubbles',
  name: 'Amber Seifenblasen',
  author: 'Marcel1853',
  description: 'Schimmernde Blasen steigen auf, Größe und Anzahl folgen der Musik',

  setup({ state }) {
    state.b = [];
    state.t = 0;
    state.prev = 0;
    state.acc = 0;
  },

  draw({ ctx, width: w, height: h, frame, info, state, amber }) {
    state.t += frame.dt;
    const hit = frame.playing && frame.bass > 0.45 && frame.bass - state.prev > 0.07;
    state.prev = frame.bass;
    state.acc += frame.dt * (frame.playing ? 2 + frame.level * 10 : 0.5);
    while (state.acc > 1 && state.b.length < 70) {
      state.acc -= 1;
      state.b.push({ x: Math.random(), y: 1.05, r: 0.012 + Math.random() * 0.03 + frame.level * 0.02, s: 0.06 + Math.random() * 0.1, p: Math.random() * 6, c: Math.random() });
    }
    if (hit) for (let i = 0; i < 6 && state.b.length < 70; i++) state.b.push({ x: Math.random(), y: 1.05, r: 0.02 + Math.random() * 0.03, s: 0.2 + Math.random() * 0.15, p: Math.random() * 6, c: Math.random() });

    const g = ctx.createLinearGradient(0, 0, 0, h);
    g.addColorStop(0, amber.mix([12, 16, 30], info.accent2, 0.15));
    g.addColorStop(1, amber.mix([10, 22, 34], info.accent, 0.2));
    ctx.fillStyle = g;
    ctx.fillRect(0, 0, w, h);

    const m = Math.min(w, h);
    for (let i = state.b.length - 1; i >= 0; i--) {
      const b = state.b[i];
      b.y -= frame.dt * b.s * (0.7 + frame.level);
      b.x += Math.sin(state.t * 1.3 + b.p) * frame.dt * 0.03;
      if (b.y < -0.05) { state.b.splice(i, 1); continue; }
      const x = b.x * w, y = b.y * h, r = b.r * m;
      ctx.fillStyle = amber.mix(info.accent, info.accent2, b.c, 0.1);
      ctx.beginPath();
      ctx.arc(x, y, r, 0, Math.PI * 2);
      ctx.fill();
      ctx.strokeStyle = amber.mix(info.accent, info.accent2, b.c, 0.7);
      ctx.lineWidth = 1.5;
      ctx.stroke();
      ctx.fillStyle = 'rgba(255,255,255,0.55)';
      ctx.beginPath();
      ctx.arc(x - r * 0.35, y - r * 0.35, r * 0.2, 0, Math.PI * 2);
      ctx.fill();
    }
  },
});
