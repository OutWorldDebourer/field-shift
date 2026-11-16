// The opening of the tutorial film on the product's own globe (frontend/globe.html on a local Vite dev server):
// orbit at sunrise over Peru, a tour of four farming regions (lockOn glides, sped up), back to the Huaral valley,
// a hold, then the dive to the field. A virtual clock makes every frame deterministic.
// usage: node globe-world.mjs <outDir> <en|es>     -> f0000.jpg … and cams.json (camera per frame)
import fs from 'fs'; import path from 'path';
import { chromium, ARGS } from '../../src/pw.mjs';
const [outDir, lang = 'en'] = process.argv.slice(2);
const PORT = process.env.GLOBE_PORT || 5190, FPS = 30, DUR = 23.5;
fs.mkdirSync(outDir, { recursive: true });
const ss = (a, b, x) => { const t = Math.min(1, Math.max(0, (x - a) / (b - a))); return t * t * (3 - 2 * t); };
const HUARAL = { lat: -11.4729, lon: -77.2061, label: 'Huaral valley, Peru', labelEs: 'Valle de Huaral, Perú',
  ring: [[-77.207403, -11.471998], [-77.206417, -11.471659], [-77.204745, -11.472721], [-77.205638, -11.473648], [-77.206692, -11.474191], [-77.207426, -11.473287]] };
// film second -> where the globe cuts to (a farming region held for one second each, then back to Huaral)
const HOPS = [
  [1.0, { lat: 42.0, lon: -93.5, label: 'US Midwest', labelEs: 'Medio Oeste de EE. UU.' }],
  [2.0, { lat: 30.9, lon: 75.8, label: 'Punjab, India', labelEs: 'Punyab, India' }],
  [3.0, { lat: -0.3, lon: 36.1, label: 'Rift Valley, Kenya', labelEs: 'Valle del Rift, Kenia' }],
  [4.0, { lat: -34.6, lon: -61.0, label: 'Pampas, Argentina', labelEs: 'La Pampa, Argentina' }],
  [5.0, HUARAL],
];
const SPEED = () => 1;
const P = (t) => ss(13.0, 21.5, t);
const INIT = `(() => {
  let vt = 0, id = 0; const cbs = new Map();
  performance.now = () => vt;
  window.requestAnimationFrame = cb => { cbs.set(++id, cb); return id; };
  window.cancelAnimationFrame = i => cbs.delete(i);
  window.__tick = ms => { vt += ms; const run = [...cbs.values()]; cbs.clear(); for (const cb of run) cb(vt); return vt; };
})();`;
const b = await chromium.launch({ args: ARGS });
const page = await b.newPage({ viewport: { width: 1920, height: 1080 }, deviceScaleFactor: 1 });
page.on('pageerror', (e) => { console.error('pageerror', e.message); process.exit(1); });
await page.addInitScript(INIT);
await page.goto(`http://localhost:${PORT}/globe.html?p=0&bare=1&pr=1&lang=${lang}`, { waitUntil: 'networkidle' });
const tick = async (ms, n = 1) => { for (let i = 0; i < n; i++) await page.evaluate((ms) => window.__tick(ms), ms); };
await tick(1000 / 30, 40); await page.waitForLoadState('networkidle'); await page.waitForTimeout(1500); await tick(1000 / 30, 160);
// pre-roll: visit the dive and every hop once so all plates are loaded, then settle at orbit over Huaral
for (const p of [0.3, 0.6, 0.9, 1]) { await page.evaluate((p) => window.__globe.setProgress(p), p); await tick(1000 / 30, 40); await page.waitForLoadState('networkidle'); await page.waitForTimeout(400); await tick(1000 / 30, 40); }
await page.evaluate(() => window.__globe.setProgress(0)); await tick(1000 / 30, 90);
for (const [, tg] of HOPS) { await page.evaluate((tg) => window.__globe.lockOn(tg), tg); await tick(1000 / 30, 70); await page.waitForLoadState('networkidle'); await page.waitForTimeout(300); }
await tick(1000 / 30, 120);
const cams = [];
const N = Math.round(DUR * FPS); let hop = 0;
for (let f = 0; f < N; f++) {
  const t = f / FPS;
  // each hop is a cut: the glide is run to its end inside one frame (the film adds a whip-pan blur on it)
  while (hop < HOPS.length && t >= HOPS[hop][0]) { await page.evaluate((tg) => window.__globe.lockOn(tg), HOPS[hop][1]); await tick(4200); hop++; }
  await page.evaluate((p) => window.__globe.setProgress(p), P(t));
  await tick((1000 / FPS) * SPEED(t));
  await page.screenshot({ path: path.join(outDir, `f${String(f).padStart(4, '0')}.jpg`), type: 'jpeg', quality: 92 });
  cams.push({ f, t: +t.toFixed(3), cam: await page.evaluate(() => window.__cam || null) });
  if (f % 60 === 0) console.log(lang, t.toFixed(1));
}
fs.writeFileSync(path.join(outDir, 'cams.json'), JSON.stringify(cams));
await b.close();
