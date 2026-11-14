// Amber Studio: Gewitter – jeder Bass-Schlag ein Blitz.
amber.visualizer({
  id: 'amber-storm',
  name: 'Amber Gewitter',
  author: 'Marcel1853',
  description: 'Dunkler Himmel, bei Bass-Schlägen zucken verzweigte Blitze',

  setup({ state }) {
    state.bolts = [];
    state.flash = 0;
    state.prev = 0;
  },

  draw({ ctx, width: w, height: h, frame, info, state, amber }) {
    const hit = frame.playing && frame.bass > 0.5 && frame.bass - state.prev > 0.08;
    state.prev = frame.bass;
    state.flash = Math.max(0, state.flash - frame.dt * 4);

    const g = ctx.createLinearGradient(0, 0, 0, h);
    g.addColorStop(0, amber.mix([8, 8, 16], info.accent2, state.flash * 0.5));
    g.addColorStop(1, amber.mix([16, 16, 26], info.accent, 0.12 + state.flash * 0.3));
    ctx.fillStyle = g;
    ctx.fillRect(0, 0, w, h);

    // Wolken
    for (let i = 0; i < 8; i++) {
      ctx.fillStyle = 'rgba(30,30,42,0.5)';
      ctx.beginPath();
      ctx.arc(w * (i / 7), h * 0.05, h * (0.14 + 0.03 * Math.sin(i * 2)), 0, Math.PI * 2);
      ctx.fill();
    }

    if (hit && state.bolts.length < 3) {
      const pts = [];
      let x = w * (0.15 + Math.random() * 0.7), y = h * 0.06;
      const branches = [];
      while (y < h * 0.9) {
        pts.push([x, y]);
        x += (Math.random() - 0.5) * w * 0.08;
        y += h * (0.04 + Math.random() * 0.05);
        if (Math.random() < 0.2) branches.push([x, y, x + (Math.random() - 0.5) * w * 0.25, y + h * 0.15]);
      }
      state.bolts.push({ pts, branches, life: 1 });
      state.flash = 1;
    }

    for (let i = state.bolts.length - 1; i >= 0; i--) {
      const b = state.bolts[i];
      b.life -= frame.dt * 2.5;
      if (b.life <= 0) { state.bolts.splice(i, 1); continue; }
      for (let pass = 0; pass < 2; pass++) {
        ctx.strokeStyle = pass === 0 ? amber.rgba(info.accent2, b.life * 0.4) : 'rgba(255,255,255,' + b.life + ')';
        ctx.lineWidth = pass === 0 ? 8 : 2.2;
        ctx.lineJoin = 'round';
        ctx.beginPath();
        b.pts.forEach((p, k) => (k === 0 ? ctx.moveTo(p[0], p[1]) : ctx.lineTo(p[0], p[1])));
        ctx.stroke();
        for (const br of b.branches) {
          ctx.beginPath();
          ctx.moveTo(br[0], br[1]);
          ctx.lineTo(br[2], br[3]);
          ctx.stroke();
        }
      }
    }
    ctx.fillStyle = 'rgb(8,8,12)';
    ctx.fillRect(0, h * 0.92, w, h * 0.08);
  },
});
