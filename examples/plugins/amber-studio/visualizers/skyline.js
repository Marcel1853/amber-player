// Amber Studio: Stadt-Skyline bei Nacht, die Hochhäuser wachsen mit den Frequenzen.
amber.visualizer({
  id: 'amber-skyline',
  name: 'Amber Skyline',
  author: 'Marcel1853',
  description: 'Nächtliche Skyline: Gebäudehöhe und beleuchtete Fenster folgen dem Spektrum',

  setup({ state }) {
    state.n = 40;
    state.h = new Array(state.n).fill(0);
    state.w = Array.from({ length: state.n }, () => 0.7 + Math.random() * 0.6);
    state.stars = Array.from({ length: 60 }, () => ({ x: Math.random(), y: Math.random() * 0.6, p: Math.random() * 6 }));
    state.t = 0;
  },

  draw({ ctx, width: w, height: h, frame, info, state, amber }) {
    state.t += frame.dt;
    const n = state.n, floor = h * 0.9;

    const sky = ctx.createLinearGradient(0, 0, 0, h);
    sky.addColorStop(0, 'rgb(8,9,18)');
    sky.addColorStop(1, amber.mix([14, 14, 20], info.accent2, 0.35));
    ctx.fillStyle = sky;
    ctx.fillRect(0, 0, w, h);

    for (const s of state.stars) {
      ctx.fillStyle = 'rgba(235,228,220,' + (0.25 + 0.25 * Math.sin(state.t * 1.5 + s.p)) + ')';
      ctx.fillRect(s.x * w, s.y * h, 1.5, 1.5);
    }

    // Mond
    ctx.fillStyle = amber.rgba(info.accent, 0.85);
    ctx.beginPath();
    ctx.arc(w * 0.82, h * 0.16, Math.min(w, h) * 0.04, 0, Math.PI * 2);
    ctx.fill();

    const total = state.w.reduce((a, b) => a + b, 0);
    let x = 0;
    for (let i = 0; i < n; i++) {
      const bw = (state.w[i] / total) * w;
      const v = Math.pow(frame.bands[Math.floor((i * frame.bands.length) / n)] || 0, 0.85);
      state.h[i] = amber.lerp(state.h[i], v, 0.2);
      const bh = h * 0.1 + state.h[i] * h * 0.55;
      const y = floor - bh;

      ctx.fillStyle = 'rgb(' + (18 + i % 3 * 3) + ',' + (18 + i % 3 * 3) + ',26)';
      ctx.fillRect(x + 1, y, bw - 2, bh);

      // Fenster: je höher der Pegel, desto mehr leuchten
      const ww = 4, wh = 6, gx = 8, gy = 11;
      const lit = state.h[i];
      for (let wy = y + 6; wy < floor - 8; wy += gy) {
        for (let wx = x + 5; wx < x + bw - ww - 3; wx += gx) {
          const k = Math.sin(wx * 12.9898 + wy * 78.233 + i) * 43758.5453;
          if (k - Math.floor(k) < lit * 0.9 + 0.05) {
            ctx.fillStyle = amber.rgba(info.accent, 0.35 + lit * 0.55);
            ctx.fillRect(wx, wy, ww, wh);
          }
        }
      }
      x += bw;
    }

    ctx.fillStyle = 'rgb(10,10,14)';
    ctx.fillRect(0, floor, w, h - floor);
    const fs = Math.max(11, Math.min(18, h * 0.03));
    ctx.textAlign = 'center';
    ctx.fillStyle = 'rgba(235,228,220,0.8)';
    ctx.font = fs + 'px sans-serif';
    ctx.fillText((info.title || '') + (info.artist ? ' · ' + info.artist : ''), w / 2, h * 0.96);
  },
});
