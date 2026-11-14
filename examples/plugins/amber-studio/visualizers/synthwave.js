// Amber Studio: Synthwave – Sonne, Gitterstraße und Berge aus dem Spektrum.
amber.visualizer({
  id: 'amber-synthwave',
  name: 'Amber Synthwave',
  author: 'Marcel1853',
  description: 'Retro-Sonnenuntergang mit Gitterstraße, die Berge folgen dem Spektrum',

  setup({ state }) {
    state.z = 0;
    state.bass = 0;
    state.mt = new Array(48).fill(0);
  },

  draw({ ctx, width: w, height: h, frame, info, state, amber }) {
    const hor = h * 0.55;
    state.bass = amber.lerp(state.bass, frame.bass, 0.2);
    state.z = (state.z + frame.dt * (frame.playing ? 0.4 + frame.level * 1.2 : 0.1)) % 1;

    const sky = ctx.createLinearGradient(0, 0, 0, hor);
    sky.addColorStop(0, 'rgb(10,8,24)');
    sky.addColorStop(1, amber.mix([30, 14, 40], info.accent2, 0.5));
    ctx.fillStyle = sky;
    ctx.fillRect(0, 0, w, hor);

    // Sonne mit Streifen
    const sr = Math.min(w, h) * (0.2 + state.bass * 0.03), sx = w / 2, sy = hor - sr * 0.35;
    const sg = ctx.createLinearGradient(0, sy - sr, 0, sy + sr);
    sg.addColorStop(0, amber.rgba(info.accent, 1));
    sg.addColorStop(1, amber.rgba(info.accent2, 1));
    ctx.fillStyle = sg;
    ctx.beginPath();
    ctx.arc(sx, sy, sr, 0, Math.PI * 2);
    ctx.fill();
    ctx.fillStyle = 'rgb(18,12,30)';
    for (let i = 0; i < 6; i++) ctx.fillRect(sx - sr, sy + sr * (0.0 + i * 0.17), sr * 2, sr * 0.03 * (i + 1));

    // Berge aus dem Spektrum
    const n = 48, bw = w / n;
    ctx.fillStyle = 'rgb(18,12,30)';
    for (let i = 0; i < n; i++) {
      const v = frame.bands[Math.floor((i * frame.bands.length) / n)] || 0;
      state.mt[i] = amber.lerp(state.mt[i], v, 0.2);
      const mh = 6 + state.mt[i] * h * 0.2;
      ctx.beginPath();
      ctx.moveTo(i * bw, hor);
      ctx.lineTo(i * bw + bw / 2, hor - mh);
      ctx.lineTo((i + 1) * bw, hor);
      ctx.fill();
    }

    // Boden und Gitter
    ctx.fillStyle = 'rgb(14,9,24)';
    ctx.fillRect(0, hor, w, h - hor);
    ctx.strokeStyle = amber.rgba(info.accent2, 0.8);
    ctx.lineWidth = 1.5;
    for (let i = 0; i < 12; i++) {
      const f = Math.pow((i + state.z) / 12, 2.2);
      const y = hor + f * (h - hor);
      ctx.beginPath();
      ctx.moveTo(0, y);
      ctx.lineTo(w, y);
      ctx.stroke();
    }
    for (let i = -10; i <= 10; i++) {
      ctx.beginPath();
      ctx.moveTo(w / 2 + i * w * 0.012, hor);
      ctx.lineTo(w / 2 + i * w * 0.14, h);
      ctx.stroke();
    }
  },
});
