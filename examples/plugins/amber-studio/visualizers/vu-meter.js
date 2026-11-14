// Amber Studio: zwei analoge Zeigerinstrumente (Pegel und Höhen).
amber.visualizer({
  id: 'amber-vu',
  name: 'Amber VU-Meter',
  author: 'Marcel1853',
  description: 'Analoge Zeigerinstrumente mit träger Nadel',

  setup({ state }) {
    state.a = [0, 0];
    state.vel = [0, 0];
  },

  draw({ ctx, width: w, height: h, frame, info, state, amber }) {
    ctx.fillStyle = 'rgb(20,17,14)';
    ctx.fillRect(0, 0, w, h);
    const target = [frame.level, amber.range(frame.bands, 0.4, 1) * 1.6];
    const labels = ['PEGEL', 'HÖHEN'];
    const mw = Math.min(w * 0.44, h * 0.9), mh = mw * 0.72;

    for (let m = 0; m < 2; m++) {
      // Feder-Dämpfer: leichtes Überschwingen wie bei einer echten Nadel
      const t = amber.clamp(target[m], 0, 1);
      state.vel[m] += (t - state.a[m]) * 90 * frame.dt;
      state.vel[m] *= Math.pow(0.0008, frame.dt);
      state.a[m] += state.vel[m] * frame.dt;
      const val = amber.clamp(state.a[m], -0.03, 1.05);

      const cx = w * (m === 0 ? 0.26 : 0.74), top = h * 0.5 - mh / 2, pivotY = top + mh * 0.95;
      ctx.fillStyle = 'rgb(232,220,196)';
      ctx.fillRect(cx - mw / 2, top, mw, mh);
      ctx.strokeStyle = 'rgb(60,50,40)';
      ctx.lineWidth = 3;
      ctx.strokeRect(cx - mw / 2, top, mw, mh);

      const R = mw * 0.8, a0 = -Math.PI * 0.72, a1 = -Math.PI * 0.28;
      for (let i = 0; i <= 20; i++) {
        const f = i / 20, ang = a0 + (a1 - a0) * f, long = i % 5 === 0;
        ctx.strokeStyle = f > 0.75 ? amber.rgba(info.accent2, 1) : 'rgb(50,42,34)';
        ctx.lineWidth = long ? 2.5 : 1.2;
        ctx.beginPath();
        ctx.moveTo(cx + Math.cos(ang) * R * (long ? 0.86 : 0.9), pivotY + Math.sin(ang) * R * (long ? 0.86 : 0.9));
        ctx.lineTo(cx + Math.cos(ang) * R, pivotY + Math.sin(ang) * R);
        ctx.stroke();
      }
      const ang = a0 + (a1 - a0) * val;
      ctx.strokeStyle = 'rgb(30,24,20)';
      ctx.lineWidth = 2.5;
      ctx.beginPath();
      ctx.moveTo(cx, pivotY);
      ctx.lineTo(cx + Math.cos(ang) * R * 0.95, pivotY + Math.sin(ang) * R * 0.95);
      ctx.stroke();
      ctx.save();
      ctx.beginPath();
      ctx.rect(cx - mw / 2, top, mw, mh);
      ctx.clip();
      ctx.fillStyle = 'rgb(40,32,26)';
      ctx.beginPath();
      ctx.arc(cx, pivotY, mw * 0.06, 0, Math.PI * 2);
      ctx.fill();
      ctx.restore();

      ctx.textAlign = 'center';
      ctx.fillStyle = 'rgb(60,50,40)';
      ctx.font = 'bold ' + Math.max(11, mw * 0.07) + 'px sans-serif';
      ctx.fillText(labels[m], cx, top + mh * 0.5);
    }
  },
});
