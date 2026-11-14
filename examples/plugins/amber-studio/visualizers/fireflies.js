// Amber Studio: Glühwürmchen, die im Takt der Musik leuchten und bei Bass-Schlägen zusammenrücken.
amber.visualizer({
  id: 'amber-fireflies',
  name: 'Amber Glühwürmchen',
  author: 'Marcel1853',
  description: 'Ein Schwarm Glühwürmchen, der mit der Musik leuchtet',

  setup({ state }) {
    state.f = Array.from({ length: 70 }, (_, i) => ({ x: Math.random(), y: 0.15 + Math.random() * 0.7, p: Math.random() * 6.28, s: 0.5 + Math.random(), b: i % 32 }));
    state.t = 0;
    state.pull = 0;
    state.prev = 0;
  },

  draw({ ctx, width: w, height: h, frame, info, state, amber }) {
    state.t += frame.dt;
    const hit = frame.playing && frame.bass > 0.45 && frame.bass - state.prev > 0.07;
    state.prev = frame.bass;
    if (hit) state.pull = 1;
    state.pull = Math.max(0, state.pull - frame.dt * 1.5);

    const g = ctx.createLinearGradient(0, 0, 0, h);
    g.addColorStop(0, 'rgb(6,9,12)');
    g.addColorStop(1, 'rgb(10,16,14)');
    ctx.fillStyle = g;
    ctx.fillRect(0, 0, w, h);
    ctx.fillStyle = 'rgb(5,8,7)';
    for (let i = 0; i < 14; i++) {
      const x = (i / 13) * w, th = h * (0.16 + 0.1 * Math.abs(Math.sin(i * 1.7)));
      ctx.beginPath();
      ctx.moveTo(x - w * 0.05, h);
      ctx.lineTo(x, h - th);
      ctx.lineTo(x + w * 0.05, h);
      ctx.fill();
    }

    for (const f of state.f) {
      f.x += Math.sin(state.t * 0.6 * f.s + f.p) * frame.dt * 0.03 + (0.5 - f.x) * state.pull * frame.dt * 0.6;
      f.y += Math.cos(state.t * 0.5 * f.s + f.p * 2) * frame.dt * 0.02 + (0.45 - f.y) * state.pull * frame.dt * 0.4;
      f.x = amber.clamp(f.x, 0.02, 0.98);
      f.y = amber.clamp(f.y, 0.08, 0.88);
      const e = frame.bands[f.b] || 0;
      const blink = 0.35 + 0.65 * Math.max(0, Math.sin(state.t * 2.2 * f.s + f.p));
      const a = amber.clamp(blink * (0.3 + e * 1.2), 0, 1);
      const x = f.x * w, y = f.y * h, r = 2 + e * 5;
      ctx.fillStyle = amber.rgba(info.accent, a * 0.22);
      ctx.beginPath();
      ctx.arc(x, y, r * 3.5, 0, Math.PI * 2);
      ctx.fill();
      ctx.fillStyle = amber.mix(info.accent, [255, 255, 220], 0.5, a);
      ctx.beginPath();
      ctx.arc(x, y, r, 0, Math.PI * 2);
      ctx.fill();
    }
  },
});
