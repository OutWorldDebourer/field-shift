// Deterministic frames of the product's own globe (frontend/globe.html on a local Vite dev server).
// A virtual clock replaces performance.now and requestAnimationFrame, so every frame is the same on every run.
// usage: node globe-frames.mjs <outDir> <schedule: dive|climb|test> [--port 5190]
import fs from 'fs'; import path from 'path';
import { chromium, ARGS } from './pw.mjs';
const [outDir, sched = 'test'] = process.argv.slice(2);
const PORT = process.env.GLOBE_PORT || 5190, FPS = 30;
fs.mkdirSync(outDir, { recursive: true });
const ss = (a, b, x) => { const t = Math.min(1, Math.max(0, (x - a) / (b - a))); return t * t * (3 - 2 * t); };
const S = {
  // film seconds -> dive progress; lockAt = when the reticle re-snaps (film s)
  dive: { t0: 0, t1: 9.6, p: t => ss(1.1, 8.3, t), lockAt: null },
  climb: { t0: 25.2, t1: 30.0, p: t => 1 - ss(25.5, 28.9, t) ** 1.0, lockAt: null },
  test: { list: [0, 0.15, 0.3, 0.5, 0.7, 0.85, 0.93, 1] },
}[sched];
const INIT = `(() => {
  let vt = 0, id = 0; const cbs = new Map();
  performance.now = () => vt;
  window.requestAnimationFrame = cb => { cbs.set(++id, cb); return id; };
  window.cancelAnimationFrame = i => cbs.delete(i);
  window.__tick = ms => { vt += ms; const run = [...cbs.values()]; cbs.clear(); for (const cb of run) cb(vt); return vt; };
})();`;
const b = await chromium.launch({ args: ARGS });
const page = await b.newPage({ viewport: { width: 1920, height: 1080 }, deviceScaleFactor: 1 });
page.on('pageerror', e => { console.error('pageerror', e.message); process.exit(1); });
await page.addInitScript(INIT);
await page.goto(`http://localhost:${PORT}/globe.html?p=0&bare=1&pr=1`, { waitUntil: 'networkidle' });
// let every texture arrive and fade on (virtual time), then reset nothing: the creep just continues
for (let i = 0; i < 40; i++) { await page.evaluate(() => window.__tick(1000 / 30)); }
await page.waitForLoadState('networkidle'); await page.waitForTimeout(1500);
for (let i = 0; i < 160; i++) { await page.evaluate(() => window.__tick(1000 / 30)); }
const cams = [];
const shot = async (file) => {
  // two ticks per frame are not needed; one tick = one frame. Wait for the compositor.
  await page.screenshot({ path: file, type: 'jpeg', quality: 92 });
};
if (sched === 'test') {
  for (const p of S.list) {
    await page.evaluate(p => window.__globe.setProgress(p), p);
    for (let i = 0; i < 45; i++) await page.evaluate(() => window.__tick(1000 / 30));
    await page.waitForLoadState('networkidle'); await page.waitForTimeout(300);
    for (let i = 0; i < 30; i++) await page.evaluate(() => window.__tick(1000 / 30));
    await shot(path.join(outDir, `p${p.toFixed(2)}.jpg`));
  }
} else {
  // pre-roll: visit the whole dive once so every plate is loaded, then settle at the first progress
  for (const p of [0.3, 0.6, 0.9, 1]) {
    await page.evaluate(p => window.__globe.setProgress(p), p);
    for (let i = 0; i < 40; i++) await page.evaluate(() => window.__tick(1000 / 30));
    await page.waitForLoadState('networkidle'); await page.waitForTimeout(400);
    for (let i = 0; i < 40; i++) await page.evaluate(() => window.__tick(1000 / 30));
  }
  await page.evaluate(p => window.__globe.setProgress(p), S.p(S.t0));
  for (let i = 0; i < 90; i++) await page.evaluate(() => window.__tick(1000 / 30));
  for (let i = 0; i < 60; i++) await page.evaluate(() => window.__tick(1000 / 30));
  const N = Math.round((S.t1 - S.t0) * FPS); let locked = false;
  for (let f = 0; f < N; f++) {
    const t = S.t0 + f / FPS;
    if (S.lockAt != null && !locked && t >= S.lockAt) { locked = true; await page.evaluate(() => window.__globe.lockOn({ lat: -11.4729, lon: -77.2061, label: 'Huaral valley, Peru', labelEs: 'Valle de Huaral, Perú', ring: [[-77.207403, -11.471998], [-77.206417, -11.471659], [-77.204745, -11.472721], [-77.205638, -11.473648], [-77.206692, -11.474191], [-77.207426, -11.473287]] })); }
    await page.evaluate(p => window.__globe.setProgress(p), S.p(t));
    await page.evaluate(() => window.__tick(1000 / 30));
    // close plates stream in as the camera descends: give the network a moment when it is busy
    const busy = await page.evaluate(() => performance.getEntriesByType('resource').length);
    await shot(path.join(outDir, `f${String(f).padStart(4, '0')}.jpg`));
    cams.push({ f, t: +t.toFixed(3), cam: await page.evaluate(() => window.__cam || null) });
    if (f % 30 === 0) console.log(sched, t.toFixed(2), busy);
  }
  fs.writeFileSync(path.join(outDir, 'cams.json'), JSON.stringify(cams));
}
await b.close();
