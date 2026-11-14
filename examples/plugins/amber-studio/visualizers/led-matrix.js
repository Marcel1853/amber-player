// Amber Studio: LED-Matrix mit Spitzenmarken wie bei einem Hi-Fi-Gerät.
amber.visualizer({
  id: 'amber-led',
  name: 'Amber LED-Matrix',
  author: 'Marcel1853',
  description: 'Leuchtpunkte in Spalten und Zeilen wie an einer Hi-Fi-Anlage',

  setup({ state }) {
    state.v = new Array(32).fill(0);
    state.pk = new Array(32).fill(0);
  },

  draw({ ctx, width: w, height: h, frame, info, state, amber }) {
    const cols = 32, rows = 16;
    ctx.fillStyle = 'rgb(12,12,15)';
    ctx.fillRect(0, 0, w, h);
    const cw = (w * 0.86) / cols, ch = (h * 0.7) / rows;
    const x0 = w * 0.07, y0 = h * 0.12;
    const d = Math.min(cw, ch) * 0.72;
    for (let c = 0; c < cols; c++) {
      const raw = frame.bands[Math.floor((c * frame.bands.length) / cols)] || 0;
      state.v[c] = raw > state.v[c] ? amber.lerp(state.v[c], raw, 0.6) : amber.lerp(state.v[c], raw, 0.12);
      state.pk[c] = Math.max(state.pk[c] - frame.dt * 0.5, state.v[c]);
      const lit = Math.round(state.v[c] * rows), pk = Math.min(rows - 1, Math.round(state.pk[c] * rows));
      for (let r = 0; r < rows; r++) {
        const t = r / (rows - 1);
        const on = r < lit || r === pk;
        ctx.fillStyle = on ? amber.mix(info.accent, info.accent2, t, r === pk ? 1 : 0.95) : amber.rgba(info.accent, 0.07);
        ctx.beginPath();
        ctx.arc(x0 + c * cw + cw / 2, y0 + (rows - 1 - r) * ch + ch / 2, d / 2, 0, Math.PI * 2);
        ctx.fill();
      }
    }
    const fs = Math.max(11, Math.min(18, h * 0.03));
    ctx.textAlign = 'center';
    ctx.fillStyle = 'rgba(235,228,220,0.75)';
    ctx.font = fs + 'px sans-serif';
    ctx.fillText((info.title || '') + (info.artist ? ' · ' + info.artist : ''), w / 2, h * 0.94);
  },
});
