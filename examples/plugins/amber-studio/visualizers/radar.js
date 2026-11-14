// Amber Studio: Radar – der Strahl lässt die Frequenzen als Punkte aufleuchten.
amber.visualizer({
  id: 'amber-radar',
  name: 'Amber Radar',
  author: 'Marcel1853',
  description: 'Radarschirm mit kreisendem Strahl, der Frequenzen als Ziele aufleuchten lässt',

  setup({ state }) {
    state.a = 0;
    state.blips = [];
    state.last = 0;
  },

  draw({ ctx, width: w, height: h, frame, info, state, amber }) {
    const cx = w / 2, cy = h * 0.48, R = Math.min(w, h) * 0.42;
    const prev = state.a;
    state.a = (state.a + frame.dt * (frame.playing ? 1.6 : 0.4)) % (Math.PI * 2);

    ctx.fillStyle = 'rgb(8,12,11)';
    ctx.fillRect(0, 0, w, h);
    ctx.strokeStyle = amber.rgba(info.accent, 0.25);
    ctx.lineWidth = 1;
    for (let i = 1; i <= 4; i++) { ctx.beginPath(); ctx.arc(cx, cy, (R * i) / 4, 0, Math.PI * 2); ctx.stroke(); }
    ctx.beginPath();
    ctx.moveTo(cx - R, cy); ctx.lineTo(cx + R, cy);
    ctx.moveTo(cx, cy - R); ctx.lineTo(cx, cy + R);
    ctx.stroke();

    // Beim Überstreichen eines Sektors entsteht ein Ziel
    const n = 24;
    for (let i = 0; i < n; i++) {
      const ang = (i / n) * Math.PI * 2;
      const crossed = prev <= ang && state.a > ang || (state.a < prev && (ang >= prev || ang < state.a));
      if (crossed) {
        const v = frame.bands[Math.floor((i * frame.bands.length) / n)] || 0;
        if (v > 0.12 && state.blips.length < 80) state.blips.push({ a: ang + (Math.random() - 0.5) * 0.1, r: 0.15 + v * 0.8, life: 1, v });
      }
    }

    // Strahl
    for (let k = 0; k < 24; k++) {
      ctx.fillStyle = amber.rgba(info.accent, 0.16 * (1 - k / 24));
      ctx.beginPath();
      ctx.moveTo(cx, cy);
      ctx.arc(cx, cy, R, state.a - (k + 1) * 0.03, state.a - k * 0.03);
      ctx.fill();
    }
    ctx.strokeStyle = amber.rgba(info.accent, 0.95);
    ctx.lineWidth = 2;
    ctx.beginPath();
    ctx.moveTo(cx, cy);
    ctx.lineTo(cx + Math.cos(state.a) * R, cy + Math.sin(state.a) * R);
    ctx.stroke();

    for (let i = state.blips.length - 1; i >= 0; i--) {
      const b = state.blips[i];
      b.life -= frame.dt * 0.45;
      if (b.life <= 0) { state.blips.splice(i, 1); continue; }
      ctx.fillStyle = amber.mix(info.accent2, info.accent, b.v, b.life);
      ctx.beginPath();
      ctx.arc(cx + Math.cos(b.a) * b.r * R, cy + Math.sin(b.a) * b.r * R, 2 + b.v * 5 * b.life, 0, Math.PI * 2);
      ctx.fill();
    }
    const fs = Math.max(11, Math.min(18, h * 0.03));
    ctx.textAlign = 'center';
    ctx.fillStyle = amber.rgba(info.accent, 0.8);
    ctx.font = fs + 'px monospace';
    ctx.fillText((info.title || '') + (info.artist ? ' · ' + info.artist : ''), cx, h * 0.96);
  },
});
