// Amber Studio: Spektrogramm-Wasserfall – zeigt, wie sich der Klang über die Zeit verändert.
amber.visualizer({
  id: 'amber-spectrogram',
  name: 'Amber Wasserfall',
  author: 'Marcel1853',
  description: 'Spektrogramm: Frequenzen von unten nach oben, die Zeit läuft von rechts nach links',

  setup({ state }) {
    state.hist = [];
    state.acc = 0;
  },

  draw({ ctx, width: w, height: h, frame, info, state, amber }) {
    const cols = 110, rows = 32;
    state.acc += frame.dt;
    if (state.acc >= 0.05) {
      state.acc = 0;
      const col = new Array(rows);
      for (let r = 0; r < rows; r++) {
        col[r] = frame.playing ? (frame.bands[Math.floor((r * frame.bands.length) / rows)] || 0) : 0;
      }
      state.hist.push(col);
      if (state.hist.length > cols) state.hist.shift();
    }

    ctx.fillStyle = 'rgb(13,13,16)';
    ctx.fillRect(0, 0, w, h);

    const cw = w / cols, ch = (h * 0.8) / rows, top = h * 0.05;
    const off = cols - state.hist.length;
    for (let c = 0; c < state.hist.length; c++) {
      const col = state.hist[c];
      for (let r = 0; r < rows; r++) {
        const v = col[r];
        if (v < 0.06) continue;
        ctx.fillStyle = amber.mix(info.accent2, info.accent, v, 0.15 + v * 0.85);
        ctx.fillRect((c + off) * cw, top + (rows - 1 - r) * ch, cw + 0.5, ch + 0.5);
      }
    }

    const fs = Math.max(11, Math.min(18, h * 0.03));
    ctx.textAlign = 'center';
    ctx.fillStyle = 'rgba(235,228,220,0.8)';
    ctx.font = fs + 'px sans-serif';
    ctx.fillText((info.title || '') + (info.artist ? ' · ' + info.artist : ''), w / 2, h * 0.94);
  },
});
