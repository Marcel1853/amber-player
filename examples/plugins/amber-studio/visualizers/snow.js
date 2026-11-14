// Amber Studio: Schneefall, Wind und Flockengröße folgen der Musik.
amber.visualizer({
  id: 'amber-snow',
  name: 'Amber Schnee',
  author: 'Marcel1853',
  description: 'Sanfter Schneefall in der Nacht, Bass-Schläge wirbeln Windstöße auf',

  setup({ state }) {
    state.f = Array.from({ length: 180 }, () => ({ x: Math.random(), y: Math.random(), s: 0.4 + Math.random(), p: Math.random() * 6 }));
    state.wind = 0;
    state.prev = 0;
    state.t = 0;
  },

  draw({ ctx, width: w, height: h, frame, info, state, amber }) {
    state.t += frame.dt;
    const hit = frame.playing && frame.bass > 0.45 && frame.bass - state.prev > 0.07;
    state.prev = frame.bass;
    if (hit) state.wind = (Math.random() < 0.5 ? -1 : 1) * (0.15 + frame.bass * 0.25);
    state.wind *= Math.pow(0.4, frame.dt);

    const g = ctx.createLinearGradient(0, 0, 0, h);
    g.addColorStop(0, 'rgb(8,10,22)');
    g.addColorStop(1, amber.mix([18, 20, 34], info.accent2, 0.3));
    ctx.fillStyle = g;
    ctx.fillRect(0, 0, w, h);

    for (const f of state.f) {
      f.y += frame.dt * (0.05 + f.s * 0.12) * (0.6 + frame.level * 1.6);
      f.x += frame.dt * (state.wind + Math.sin(state.t * 0.8 + f.p) * 0.02);
      if (f.y > 1.02) { f.y = -0.02; f.x = Math.random(); }
      if (f.x > 1.02) f.x = -0.02;
      if (f.x < -0.02) f.x = 1.02;
      ctx.fillStyle = amber.mix([235, 235, 245], info.accent, 0.15, 0.4 + f.s * 0.5);
      ctx.beginPath();
      ctx.arc(f.x * w, f.y * h, 1 + f.s * 2.2 + frame.level * 1.2, 0, Math.PI * 2);
      ctx.fill();
    }
    ctx.fillStyle = 'rgb(226,228,238)';
    ctx.fillRect(0, h * 0.95, w, h * 0.05);
  },
});
