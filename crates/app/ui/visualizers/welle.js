// Welle: die Schwingung als leuchtender Strich mit Nachzieh-Spur – wie ein Oszilloskop.
amber.visualizer({
  id: 'welle',
  name: 'Welle',
  author: 'Amber Player',
  description: 'Leuchtende Schwingungslinie mit Nachzieh-Spur',

  setup({ ctx, width, height, state }) {
    state.gain = 1;
    ctx.fillStyle = '#0a0908';
    ctx.fillRect(0, 0, width, height);
  },

  draw({ ctx, width: w, height: h, frame, info, state, amber }) {
    // halbtransparent übermalen = kurze Spur der letzten Bilder
    ctx.fillStyle = 'rgba(10, 9, 8, 0.38)';
    ctx.fillRect(0, 0, w, h);

    // Wie ein Oszilloskop: am ersten steigenden Nulldurchgang beginnen, damit die Welle stillsteht.
    const src = frame.wave;
    const span = Math.floor(src.length / 2);
    let start = 0;
    for (let i = 1; i < src.length - span; i++) {
      if (src[i - 1] < 0 && src[i] >= 0) {
        start = i;
        break;
      }
    }
    // Auf ~180 Punkte glätten
    const points = 180;
    const step = span / points;
    const wave = [];
    for (let p = 0; p < points; p++) {
      let sum = 0;
      const a = start + Math.floor(p * step);
      const b = Math.max(a + 1, start + Math.floor((p + 1) * step));
      for (let i = a; i < b; i++) sum += src[i] || 0;
      wave.push(sum / (b - a));
    }

    let peak = 0.02;
    for (const s of wave) peak = Math.max(peak, Math.abs(s));
    // Lautstärke ausgleichen, damit auch leise Stellen sichtbar schwingen
    state.gain = amber.lerp(state.gain, Math.min(2.5, 0.8 / peak), 0.05);
    const amp = h * 0.26 * state.gain;
    const mid = h / 2;

    const line = (color, width, scale, phase) => {
      ctx.beginPath();
      for (let i = 0; i < wave.length; i++) {
        const x = (i / (wave.length - 1)) * w;
        const idle = frame.playing ? 0 : Math.sin(i / 12 + frame.time * 2 + phase) * 3;
        const y = mid + wave[i] * scale * amp + idle;
        i ? ctx.lineTo(x, y) : ctx.moveTo(x, y);
      }
      ctx.strokeStyle = color;
      ctx.lineWidth = width;
      ctx.stroke();
    };

    ctx.lineJoin = 'round';
    ctx.lineCap = 'round';
    ctx.shadowBlur = 22;
    ctx.shadowColor = amber.rgba(info.accent, 0.8);
    line(amber.rgba(info.accent2, 0.35), 2, -0.5, 1.5);
    line(amber.rgba(info.accent), 3.5, 1, 0);
    ctx.shadowBlur = 0;
  },
});
