// Amber Studio: Feuerwerk – jeder Bass-Schlag schickt eine Rakete los.
amber.visualizer({
  id: 'amber-fireworks',
  name: 'Amber Feuerwerk',
  author: 'Marcel1853',
  description: 'Raketen steigen bei Bass-Schlägen auf und explodieren in den Akzentfarben',

  setup({ state }) {
    state.rk = [];
    state.sp = [];
    state.prev = 0;
    state.idle = 0;
  },

  draw({ ctx, width: w, height: h, frame, info, state, amber }) {
    ctx.fillStyle = 'rgba(8,8,14,0.22)';
    ctx.fillRect(0, 0, w, h);

    const hit = frame.playing && frame.bass > 0.45 && frame.bass - state.prev > 0.07;
    state.prev = frame.bass;
    state.idle += frame.dt;
    if ((hit || state.idle > 2.5) && state.rk.length < 8) {
      state.idle = 0;
      state.rk.push({ x: w * (0.15 + Math.random() * 0.7), y: h, vy: -h * (0.7 + Math.random() * 0.35), big: 0.5 + frame.level });
    }

    for (let i = state.rk.length - 1; i >= 0; i--) {
      const r = state.rk[i];
      r.y += r.vy * frame.dt;
      r.vy += h * 0.55 * frame.dt;
      ctx.fillStyle = amber.rgba(info.accent, 0.9);
      ctx.fillRect(r.x - 1, r.y, 2, 8);
      if (r.vy > -h * 0.08) {
        const n = Math.min(45, Math.floor(20 + r.big * 20));
        const c = Math.random();
        for (let j = 0; j < n && state.sp.length < 450; j++) {
          const a = (j / n) * Math.PI * 2, sp = (0.1 + Math.random() * 0.25) * Math.min(w, h) * (0.6 + r.big);
          state.sp.push({ x: r.x, y: r.y, vx: Math.cos(a) * sp, vy: Math.sin(a) * sp, life: 1, c });
        }
        state.rk.splice(i, 1);
      }
    }

    for (let i = state.sp.length - 1; i >= 0; i--) {
      const p = state.sp[i];
      p.x += p.vx * frame.dt;
      p.y += p.vy * frame.dt;
      p.vy += h * 0.18 * frame.dt;
      p.vx *= 0.99;
      p.life -= frame.dt * 0.7;
      if (p.life <= 0) { state.sp.splice(i, 1); continue; }
      ctx.fillStyle = amber.mix(info.accent, info.accent2, p.c, p.life);
      ctx.beginPath();
      ctx.arc(p.x, p.y, 1 + p.life * 2.2, 0, Math.PI * 2);
      ctx.fill();
    }
  },
});
