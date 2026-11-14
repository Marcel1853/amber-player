// Amber Studio: Drahtgitter-Landschaft im Stil von Joy Division, geformt vom Spektrum.
amber.visualizer({
  id: 'amber-terrain',
  name: 'Amber Gebirge',
  author: 'Marcel1853',
  description: 'Drahtgitter-Berge fließen auf dich zu, ihre Höhe kommt aus dem Spektrum',

  setup({ state }) {
    state.rows = [];
    state.acc = 0;
  },

  draw({ ctx, width: w, height: h, frame, info, state, amber }) {
    const N = 26, C = 40;
    state.acc += frame.dt;
    if (state.acc > 0.07) {
      state.acc = 0;
      const row = [];
      for (let c = 0; c < C; c++) {
        const k = Math.abs(c - (C - 1) / 2) / ((C - 1) / 2); // symmetrisch
        row.push(frame.playing ? (frame.bands[Math.floor(k * (frame.bands.length - 1))] || 0) * (1 - k * 0.5) : 0.03);
      }
      state.rows.unshift(row);
      if (state.rows.length > N) state.rows.pop();
    }

    ctx.fillStyle = 'rgb(9,9,13)';
    ctx.fillRect(0, 0, w, h);
    for (let r = state.rows.length - 1; r >= 0; r--) {
      const t = 1 - r / N; // 0 hinten, 1 vorn
      const base = h * 0.3 + Math.pow(t, 1.6) * h * 0.62;
      const wf = 0.35 + 0.75 * t;
      ctx.beginPath();
      for (let c = 0; c < C; c++) {
        const x = w / 2 + (c / (C - 1) - 0.5) * w * wf;
        const y = base - state.rows[r][c] * h * 0.32 * (0.35 + t);
        if (c === 0) ctx.moveTo(x, y); else ctx.lineTo(x, y);
      }
      ctx.lineTo(w / 2 + 0.5 * w * wf, h);
      ctx.lineTo(w / 2 - 0.5 * w * wf, h);
      ctx.closePath();
      ctx.fillStyle = 'rgb(9,9,13)';
      ctx.fill();
      ctx.lineWidth = 1 + t * 1.5;
      ctx.strokeStyle = amber.mix(info.accent2, info.accent, t, 0.3 + t * 0.7);
      ctx.stroke();
    }
  },
});
