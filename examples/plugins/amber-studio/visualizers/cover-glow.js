// Amber Studio: Cover im Mittelpunkt, Lichthof und Staub reagieren auf die Musik.
amber.visualizer({
  id: 'amber-cover-glow',
  name: 'Amber Cover Glow',
  author: 'Marcel1853',
  description: 'Pulsierendes Cover mit Lichthof und schwebendem Staub',

  setup({ state }) {
    state.bass = 0;
    state.dust = [];
    for (let i = 0; i < 70; i++) {
      state.dust.push({ x: Math.random(), y: Math.random(), s: 0.3 + Math.random(), z: Math.random() });
    }
  },

  draw({ ctx, width: w, height: h, frame, info, state, amber }) {
    const cx = w / 2, cy = h * 0.46, m = Math.min(w, h);
    state.bass = amber.lerp(state.bass, frame.bass, 0.2);

    ctx.fillStyle = 'rgb(15,15,18)';
    ctx.fillRect(0, 0, w, h);

    const g = ctx.createRadialGradient(cx, cy, m * 0.05, cx, cy, m * (0.5 + state.bass * 0.2));
    g.addColorStop(0, amber.rgba(info.accent, 0.35 + frame.level * 0.25));
    g.addColorStop(1, amber.rgba(info.accent2, 0));
    ctx.fillStyle = g;
    ctx.fillRect(0, 0, w, h);

    for (const d of state.dust) {
      d.y -= frame.dt * 0.02 * d.s * (frame.playing ? 1 + frame.level * 3 : 0.3);
      if (d.y < 0) { d.y = 1; d.x = Math.random(); }
      ctx.fillStyle = amber.rgba(info.accent, 0.1 + d.z * 0.3);
      ctx.beginPath();
      ctx.arc(d.x * w, d.y * h, 1 + d.z * 2, 0, Math.PI * 2);
      ctx.fill();
    }

    const size = m * (0.3 + state.bass * 0.04);
    if (info.cover) {
      ctx.drawImage(info.cover, cx - size / 2, cy - size / 2, size, size);
    } else {
      ctx.fillStyle = amber.rgba(info.accent, 0.3);
      ctx.fillRect(cx - size / 2, cy - size / 2, size, size);
    }
    ctx.strokeStyle = amber.rgba(info.accent, 0.5);
    ctx.lineWidth = 2;
    ctx.strokeRect(cx - size / 2, cy - size / 2, size, size);

    const by = cy + size / 2 + m * 0.04;
    ctx.fillStyle = amber.rgba(info.accent, 0.15);
    ctx.fillRect(cx - size / 2, by, size, 4);
    ctx.fillStyle = amber.rgba(info.accent, 0.9);
    ctx.fillRect(cx - size / 2, by, size * amber.clamp(frame.level, 0, 1), 4);

    const fs = Math.max(12, Math.min(22, h * 0.04));
    ctx.textAlign = 'center';
    ctx.fillStyle = 'rgba(235,228,220,0.9)';
    ctx.font = 'bold ' + fs + 'px sans-serif';
    ctx.fillText(info.title || '', cx, by + fs * 2);
    ctx.fillStyle = amber.rgba(info.accent, 0.85);
    ctx.font = fs * 0.8 + 'px sans-serif';
    ctx.fillText(info.artist || '', cx, by + fs * 3.3);
  },
});
