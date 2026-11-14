// Amber Studio: Wabenmuster, das mit dem Spektrum und Bass-Wellen aufleuchtet.
amber.visualizer({
  id: 'amber-hex',
  name: 'Amber Waben',
  author: 'Marcel1853',
  description: 'Sechseck-Raster: Frequenzen leuchten von innen nach außen, Bass-Schläge senden Wellen',

  setup({ state }) {
    state.rip = [];
    state.prev = 0;
  },

  draw({ ctx, width: w, height: h, frame, info, state, amber }) {
    const hit = frame.playing && frame.bass > 0.45 && frame.bass - state.prev > 0.07;
    state.prev = frame.bass;
    if (hit && state.rip.length < 4) state.rip.push(0);

    ctx.fillStyle = 'rgb(11,11,15)';
    ctx.fillRect(0, 0, w, h);
    const cx = w / 2, cy = h / 2, maxD = Math.hypot(cx, cy);
    const sz = Math.min(w, h) * 0.05, hw = sz * 1.732, vh = sz * 1.5;
    for (let i = state.rip.length - 1; i >= 0; i--) {
      state.rip[i] += frame.dt * 0.9;
      if (state.rip[i] > 1.3) state.rip.splice(i, 1);
    }
    let row = 0;
    for (let y = -vh; y < h + vh; y += vh, row++) {
      for (let x = -hw + (row % 2) * hw / 2; x < w + hw; x += hw) {
        const d = Math.hypot(x - cx, y - cy) / maxD;
        let v = (frame.bands[Math.min(frame.bands.length - 1, Math.floor(d * frame.bands.length))] || 0) * 0.9;
        for (const r of state.rip) v += Math.max(0, 1 - Math.abs(d - r) * 7) * 0.8;
        v = Math.min(1, v);
        ctx.fillStyle = amber.mix(info.accent2, info.accent, v, 0.06 + v * 0.9);
        ctx.beginPath();
        for (let k = 0; k < 6; k++) {
          const a = Math.PI / 6 + (k * Math.PI) / 3;
          const px = x + Math.cos(a) * sz * 0.9, py = y + Math.sin(a) * sz * 0.9;
          if (k === 0) ctx.moveTo(px, py); else ctx.lineTo(px, py);
        }
        ctx.closePath();
        ctx.fill();
      }
    }
  },
});
