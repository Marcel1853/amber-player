// Amber Studio: zwei ineinander drehende Drahtwürfel, pulsierend mit dem Bass.
amber.visualizer({
  id: 'amber-cube',
  name: 'Amber Würfel',
  author: 'Marcel1853',
  description: 'Rotierende 3D-Drahtwürfel, die mit dem Bass pulsieren',

  setup({ state }) {
    state.a = 0;
    state.b = 0;
    state.bass = 0;
  },

  draw({ ctx, width: w, height: h, frame, info, state, amber }) {
    const sp = frame.playing ? 0.4 + frame.level * 1.4 : 0.08;
    state.a += frame.dt * sp;
    state.b += frame.dt * sp * 0.7;
    state.bass = amber.lerp(state.bass, frame.bass, 0.3);
    ctx.fillStyle = 'rgba(10,10,14,0.3)';
    ctx.fillRect(0, 0, w, h);

    const cx = w / 2, cy = h / 2, m = Math.min(w, h);
    const V = [];
    for (let i = 0; i < 8; i++) V.push([(i & 1) * 2 - 1, ((i >> 1) & 1) * 2 - 1, ((i >> 2) & 1) * 2 - 1]);
    const E = [[0,1],[2,3],[4,5],[6,7],[0,2],[1,3],[4,6],[5,7],[0,4],[1,5],[2,6],[3,7]];

    const cube = (size, ax, ay, col, wid) => {
      const P = V.map(([x, y, z]) => {
        let y1 = y * Math.cos(ax) - z * Math.sin(ax), z1 = y * Math.sin(ax) + z * Math.cos(ax);
        let x1 = x * Math.cos(ay) + z1 * Math.sin(ay), z2 = -x * Math.sin(ay) + z1 * Math.cos(ay);
        const f = 3.2 / (3.2 + z2);
        return [cx + x1 * size * f, cy + y1 * size * f, f];
      });
      ctx.strokeStyle = col;
      ctx.lineWidth = wid;
      ctx.lineJoin = 'round';
      for (const [i, j] of E) {
        ctx.beginPath();
        ctx.moveTo(P[i][0], P[i][1]);
        ctx.lineTo(P[j][0], P[j][1]);
        ctx.stroke();
      }
      ctx.fillStyle = col;
      for (const p of P) { ctx.beginPath(); ctx.arc(p[0], p[1], 2 + p[2] * 2, 0, Math.PI * 2); ctx.fill(); }
    };
    const big = m * (0.16 + state.bass * 0.05);
    cube(big, state.a, state.b, amber.rgba(info.accent, 0.95), 2);
    cube(big * 0.55, -state.b * 1.3, state.a * 1.2, amber.rgba(info.accent2, 0.95), 1.6);
  },
});
