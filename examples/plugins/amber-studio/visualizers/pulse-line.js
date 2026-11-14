// Amber Studio: Puls-Linie wie ein Herzmonitor, jeder Bass-Schlag ist ein Ausschlag.
amber.visualizer({
  id: 'amber-pulse-line',
  name: 'Amber Puls',
  author: 'Marcel1853',
  description: 'Herzmonitor-Linie: Bass-Schläge werden zu Ausschlägen',

  setup({ state }) {
    state.hist = new Array(240).fill(0);
    state.acc = 0;
    state.prev = 0;
    state.spike = 0;
  },

  draw({ ctx, width: w, height: h, frame, info, state, amber }) {
    ctx.fillStyle = 'rgb(10,13,12)';
    ctx.fillRect(0, 0, w, h);
    ctx.strokeStyle = amber.rgba(info.accent, 0.07);
    ctx.lineWidth = 1;
    for (let x = 0; x < w; x += w / 24) { ctx.beginPath(); ctx.moveTo(x, 0); ctx.lineTo(x, h); ctx.stroke(); }
    for (let y = 0; y < h; y += h / 12) { ctx.beginPath(); ctx.moveTo(0, y); ctx.lineTo(w, y); ctx.stroke(); }

    const hit = frame.playing && frame.bass > 0.42 && frame.bass - state.prev > 0.06;
    state.prev = frame.bass;
    if (hit) state.spike = 1;
    state.acc += frame.dt;
    while (state.acc >= 1 / 60) {
      state.acc -= 1 / 60;
      let v = (frame.wave[0] || 0) * 0.04 * frame.level;
      if (state.spike > 0) {
        v += Math.sin(state.spike * Math.PI * 3) * state.spike * (0.6 + frame.bass);
        state.spike -= 0.07;
      }
      state.hist.push(v);
      state.hist.shift();
    }

    const n = state.hist.length, mid = h * 0.5, amp = h * 0.32;
    for (let pass = 0; pass < 2; pass++) {
      ctx.beginPath();
      for (let i = 0; i < n; i++) {
        const x = (i / (n - 1)) * w, y = mid - state.hist[i] * amp;
        if (i === 0) ctx.moveTo(x, y); else ctx.lineTo(x, y);
      }
      ctx.lineJoin = 'round';
      ctx.lineWidth = pass === 0 ? 7 : 2;
      ctx.strokeStyle = amber.rgba(pass === 0 ? info.accent2 : info.accent, pass === 0 ? 0.2 : 1);
      ctx.stroke();
    }
    ctx.fillStyle = amber.rgba(info.accent, 1);
    ctx.beginPath();
    ctx.arc(w, mid - state.hist[n - 1] * amp, 4, 0, Math.PI * 2);
    ctx.fill();
  },
});
