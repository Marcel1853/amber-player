// Ring: das Cover dreht sich in der Mitte, die Frequenzen strahlen kreisförmig ab.
amber.visualizer({
  id: 'ring',
  name: 'Ring',
  author: 'Amber Player',
  description: 'Drehendes Cover mit kreisförmigem Spektrum, pulsiert mit dem Bass',

  setup({ state }) {
    state.angle = 0;
    state.v = [];
    state.pulse = 0;
  },

  draw({ ctx, width: w, height: h, frame, info, state, amber }) {
    ctx.fillStyle = '#0b0a09';
    ctx.fillRect(0, 0, w, h);

    const cx = w / 2;
    const cy = h / 2;
    const unit = Math.min(w, h);
    state.pulse = amber.lerp(state.pulse, frame.bass, 0.25);
    const r = unit * 0.17 * (1 + state.pulse * 0.18);
    if (frame.playing) state.angle += frame.dt * 0.25;

    // Glühen hinter dem Ring
    const glow = ctx.createRadialGradient(cx, cy, r * 0.8, cx, cy, r * 2.6);
    glow.addColorStop(0, amber.rgba(info.accent, 0.18 + frame.level * 0.25));
    glow.addColorStop(1, amber.rgba(info.accent, 0));
    ctx.fillStyle = glow;
    ctx.fillRect(0, 0, w, h);

    // Strahlen: Bänder gespiegelt einmal rundherum
    const n = frame.bands.length;
    ctx.lineCap = 'round';
    ctx.lineWidth = Math.max(2, (2 * Math.PI * r) / (n * 2) * 0.55);
    for (let i = 0; i < n * 2; i++) {
      const k = i < n ? i : 2 * n - 1 - i;
      const target = frame.bands[k];
      const v = state.v[i] || 0;
      state.v[i] = target > v ? v + (target - v) * 0.5 : v * Math.pow(0.05, frame.dt);
      const a = (i / (n * 2)) * Math.PI * 2 - Math.PI / 2 + state.angle * 0.2;
      const len = 4 + state.v[i] * unit * 0.2;
      const r0 = r + 10;
      ctx.strokeStyle = amber.mix(info.accent, info.accent2, k / n, 0.95);
      ctx.beginPath();
      ctx.moveTo(cx + Math.cos(a) * r0, cy + Math.sin(a) * r0);
      ctx.lineTo(cx + Math.cos(a) * (r0 + len), cy + Math.sin(a) * (r0 + len));
      ctx.stroke();
    }

    // Cover als drehende Scheibe
    ctx.save();
    ctx.beginPath();
    ctx.arc(cx, cy, r, 0, Math.PI * 2);
    ctx.clip();
    ctx.translate(cx, cy);
    ctx.rotate(state.angle);
    if (info.cover) {
      ctx.drawImage(info.cover, -r, -r, r * 2, r * 2);
    } else {
      const g = ctx.createLinearGradient(-r, -r, r, r);
      g.addColorStop(0, amber.rgba(info.accent));
      g.addColorStop(1, amber.rgba(info.accent2));
      ctx.fillStyle = g;
      ctx.fillRect(-r, -r, r * 2, r * 2);
    }
    ctx.restore();

    // Mittelloch wie bei einer Platte
    ctx.fillStyle = '#0b0a09';
    ctx.beginPath();
    ctx.arc(cx, cy, r * 0.09, 0, Math.PI * 2);
    ctx.fill();
    ctx.strokeStyle = 'rgba(255, 255, 255, 0.15)';
    ctx.lineWidth = 2;
    ctx.beginPath();
    ctx.arc(cx, cy, r, 0, Math.PI * 2);
    ctx.stroke();
  },
});
