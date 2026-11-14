// Amber Studio: Flug durch einen Tunnel aus Ringen, jeder Bass-Schlag schickt einen neuen los.
amber.visualizer({
  id: 'amber-tunnel',
  name: 'Amber Tunnel',
  author: 'Marcel1853',
  description: 'Flug durch einen Ringtunnel, der mit dem Bass beschleunigt',

  setup({ state }) {
    state.rings = [];
    state.spawn = 0;
    state.prev = 0;
    state.speed = 0.4;
    state.rot = 0;
  },

  draw({ ctx, width: w, height: h, frame, info, state, amber }) {
    const cx = w / 2, cy = h / 2, m = Math.max(w, h);
    state.speed = amber.lerp(state.speed, frame.playing ? 0.35 + frame.level * 0.9 : 0.1, 0.05);
    state.rot += frame.dt * 0.25;
    state.spawn += frame.dt * state.speed;
    const hit = frame.playing && frame.bass > 0.4 && frame.bass - state.prev > 0.07;
    state.prev = frame.bass;

    if ((state.spawn > 0.22 || hit) && state.rings.length < 40) {
      state.spawn = 0;
      state.rings.push({ z: 0, hit, sides: 6, c: Math.random() });
    }

    ctx.fillStyle = 'rgba(10,10,14,0.35)';
    ctx.fillRect(0, 0, w, h);

    for (let i = state.rings.length - 1; i >= 0; i--) {
      const r = state.rings[i];
      r.z += frame.dt * state.speed;
      if (r.z > 1.2) { state.rings.splice(i, 1); continue; }
      const radius = m * 0.02 + Math.pow(r.z, 2.2) * m * 0.75;
      const alpha = amber.clamp(Math.sin(Math.min(r.z, 1) * Math.PI), 0, 1) * (r.hit ? 0.95 : 0.55);
      ctx.strokeStyle = amber.mix(info.accent2, info.accent, r.hit ? 1 : r.c, alpha);
      ctx.lineWidth = 1 + r.z * (r.hit ? 6 : 3);
      ctx.beginPath();
      for (let s = 0; s <= r.sides; s++) {
        const a = state.rot * (1 + r.z) + (s / r.sides) * Math.PI * 2;
        const x = cx + Math.cos(a) * radius;
        const y = cy + Math.sin(a) * radius;
        if (s === 0) ctx.moveTo(x, y); else ctx.lineTo(x, y);
      }
      ctx.stroke();
    }

    const g = ctx.createRadialGradient(cx, cy, 0, cx, cy, m * 0.12);
    g.addColorStop(0, amber.rgba(info.accent, 0.3 + frame.bass * 0.5));
    g.addColorStop(1, amber.rgba(info.accent, 0));
    ctx.fillStyle = g;
    ctx.fillRect(0, 0, w, h);
  },
});
