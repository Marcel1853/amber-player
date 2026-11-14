// Amber Studio: Schallplatte mit Cover als Label und Tonarm.
amber.visualizer({
  id: 'amber-vinyl',
  name: 'Amber Schallplatte',
  author: 'Marcel1853',
  description: 'Drehende Schallplatte mit Cover-Label, Tonarm und Lichtreflexen',

  setup({ state }) {
    state.rot = 0;
    state.arm = 0;
    state.bass = 0;
  },

  draw({ ctx, width: w, height: h, frame, info, state, amber }) {
    const cx = w * 0.46, cy = h * 0.47, R = Math.min(w, h) * 0.38;
    state.bass = amber.lerp(state.bass, frame.bass, 0.3);
    state.rot += frame.dt * (frame.playing ? 3.5 : 0.4);
    state.arm = amber.lerp(state.arm, frame.playing ? 1 : 0, 0.04);

    ctx.fillStyle = 'rgb(22,17,14)';
    ctx.fillRect(0, 0, w, h);
    const g = ctx.createRadialGradient(cx, cy, R * 0.2, cx, cy, R * 1.6);
    g.addColorStop(0, amber.rgba(info.accent, 0.12 + state.bass * 0.1));
    g.addColorStop(1, amber.rgba(info.accent, 0));
    ctx.fillStyle = g;
    ctx.fillRect(0, 0, w, h);

    ctx.fillStyle = 'rgb(12,12,14)';
    ctx.beginPath();
    ctx.arc(cx, cy, R, 0, Math.PI * 2);
    ctx.fill();
    for (let i = 0; i < 22; i++) {
      ctx.strokeStyle = 'rgba(255,255,255,' + (i % 3 === 0 ? 0.09 : 0.04) + ')';
      ctx.lineWidth = 1;
      ctx.beginPath();
      ctx.arc(cx, cy, R * (0.4 + i * 0.027), 0, Math.PI * 2);
      ctx.stroke();
    }
    // Lichtreflexe drehen nicht mit
    for (const a0 of [-0.9, 2.24]) {
      ctx.fillStyle = 'rgba(255,255,255,0.07)';
      ctx.beginPath();
      ctx.moveTo(cx, cy);
      ctx.arc(cx, cy, R * 0.98, a0, a0 + 0.35);
      ctx.fill();
    }
    // Label mit Cover (dreht mit)
    const lr = R * 0.34;
    ctx.save();
    ctx.translate(cx, cy);
    ctx.rotate(state.rot);
    ctx.beginPath();
    ctx.arc(0, 0, lr, 0, Math.PI * 2);
    ctx.clip();
    if (info.cover) ctx.drawImage(info.cover, -lr, -lr, lr * 2, lr * 2);
    else { ctx.fillStyle = amber.rgba(info.accent, 0.9); ctx.fillRect(-lr, -lr, lr * 2, lr * 2); ctx.fillStyle = amber.rgba(info.accent2, 1); ctx.fillRect(-lr, 0, lr * 2, lr); }
    ctx.restore();
    ctx.fillStyle = 'rgb(20,16,14)';
    ctx.beginPath();
    ctx.arc(cx, cy, R * 0.035, 0, Math.PI * 2);
    ctx.fill();

    // Tonarm
    const px = w * 0.86, py = h * 0.14;
    const ang = 2.15 - state.arm * 0.0 + (1 - state.arm) * -0.5;
    const len = R * 1.15;
    const ex = px + Math.cos(ang) * len, ey = py + Math.sin(ang) * len;
    ctx.strokeStyle = 'rgb(200,190,180)';
    ctx.lineWidth = Math.max(3, R * 0.03);
    ctx.lineCap = 'round';
    ctx.beginPath();
    ctx.moveTo(px, py);
    ctx.lineTo(ex, ey);
    ctx.stroke();
    ctx.fillStyle = amber.rgba(info.accent, 1);
    ctx.fillRect(ex - 7, ey - 4, 14, 8);
    ctx.fillStyle = 'rgb(60,55,50)';
    ctx.beginPath();
    ctx.arc(px, py, R * 0.07, 0, Math.PI * 2);
    ctx.fill();

    const fs = Math.max(12, Math.min(20, h * 0.035));
    ctx.textAlign = 'center';
    ctx.fillStyle = 'rgba(235,228,220,0.85)';
    ctx.font = 'bold ' + fs + 'px sans-serif';
    ctx.fillText(info.title || '', cx, h * 0.93);
    ctx.fillStyle = amber.rgba(info.accent, 0.85);
    ctx.font = fs * 0.8 + 'px sans-serif';
    ctx.fillText(info.artist || '', cx, h * 0.93 + fs * 1.2);
  },
});
