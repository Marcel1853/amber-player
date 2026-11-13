// Beispiel für eine eigene Visualisierung: Sterne fliegen auf dich zu, der Bass gibt das Tempo vor.
// Vorlage für eigene Plugins – einfach kopieren, umbenennen und loslegen.
amber.visualizer({
  id: 'sternenfeld',
  name: 'Sternenfeld',
  author: 'Beispiel',
  description: 'Sterne fliegen auf dich zu, der Bass gibt das Tempo vor',

  setup({ state }) {
    state.stars = Array.from({ length: 400 }, () => ({ x: Math.random() * 2 - 1, y: Math.random() * 2 - 1, z: Math.random() }));
  },

  draw({ ctx, width: w, height: h, frame, info, state, amber }) {
    ctx.fillStyle = 'rgba(5, 5, 8, 0.35)';
    ctx.fillRect(0, 0, w, h);
    const speed = (0.05 + frame.bass * 0.9) * frame.dt * (frame.playing ? 1 : 0.2);
    for (const s of state.stars) {
      s.z -= speed;
      if (s.z <= 0.01) {
        s.x = Math.random() * 2 - 1;
        s.y = Math.random() * 2 - 1;
        s.z = 1;
      }
      const px = w / 2 + (s.x / s.z) * w * 0.4;
      const py = h / 2 + (s.y / s.z) * h * 0.4;
      const r = (1 - s.z) * (2 + frame.treble * 4);
      ctx.fillStyle = amber.mix(info.accent, [255, 255, 255], 1 - s.z, 1 - s.z * 0.6);
      ctx.beginPath();
      ctx.arc(px, py, Math.max(0.3, r), 0, Math.PI * 2);
      ctx.fill();
    }
  },
});
