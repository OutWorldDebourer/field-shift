// Screenshots of the REAL app at 2x (live site by default, bundled example analysis `demo-huaral`), plus the measured
// boxes (CSS px in a 1440x900 viewport) of every element the film's cursor and reticle go to.
// The box names are the ones comp.html reads (field / chatA / chat / dash / rot).
// usage: node capture-app.mjs <outDir>      (APP_BASE overrides the site)
import fs from 'fs'; import path from 'path';
import { chromium, ARGS } from './pw.mjs';
const [outDir] = process.argv.slice(2);
const BASE = process.env.APP_BASE || 'https://fieldshift.earth/';
fs.mkdirSync(outDir, { recursive: true });
const b = await chromium.launch({ args: ARGS });
const page = await b.newPage({ viewport: { width: 1440, height: 900 }, deviceScaleFactor: 2 });
page.on('pageerror', (e) => console.error('pageerror', e.message));
// first visible element matching `sel` whose text contains `text` (smallest one when `small`)
const box = (sel, text = null, small = false) => page.evaluate(([sel, text, small]) => {
  let best = null;
  for (const e of document.querySelectorAll(sel)) {
    const r = e.getBoundingClientRect();
    if (r.width < 2 || r.height < 2 || r.bottom < 0 || r.top > 900 || r.right < 0 || r.left > 1440) continue;
    if (text && !(e.innerText || e.textContent || '').toLowerCase().includes(text.toLowerCase())) continue;
    const bx = { x: r.x, y: r.y, w: r.width, h: r.height };
    if (!small) return bx;
    if (!best || bx.w * bx.h < best.w * best.h) best = bx;
  }
  return best;
}, [sel, text, small]);
const nth = (sel, i) => page.evaluate(([sel, i]) => { const e = document.querySelectorAll(sel)[i]; const r = e && e.getBoundingClientRect(); return r && { x: r.x, y: r.y, w: r.width, h: r.height }; }, [sel, i]);
const A = 'app.html?a=demo-huaral&lang=en';
const go = async (u) => { await page.goto(BASE + u, { waitUntil: 'networkidle' }); await page.waitForTimeout(3000); };
const meta = {};
const shot = async (name, boxes) => {
  await page.mouse.move(-10, -10); await page.waitForTimeout(500);
  await page.screenshot({ path: path.join(outDir, name + '.jpg'), type: 'jpeg', quality: 90 });
  meta[name] = await boxes();
  console.log(name, JSON.stringify(meta[name]));
};

// 01 the field picker with the example field chosen
await go('app.html?lang=en');
await page.click('.fs__examples li');
await page.waitForTimeout(4500);
await shot('field', async () => ({
  use: await box('button', 'Use this field'), area: await box('*', ' ha', true), example: await box('button', 'Example site'), ret: await box('.reticle') }));

// 02 the chat: the photo message and the consultant's answer; then the same chat a moment before the photos arrive
await go(A + '&tab=chat');
const center = () => page.evaluate(() => { const t = document.querySelector('.chat__log .tiles'); if (t) t.scrollIntoView({ block: 'center' }); });
await center(); await page.waitForTimeout(1200);
await shot('chat', async () => ({
  photo3: await nth('.chat__log .tiles .tile', 2), photos: await box('.chat__log .tiles'), reply: await box('.bubble', 'blurry'),
  temp: await box('.secs .drow', 'temperature'), dashTab: await box('button, a', 'Dashboard', true), panel: await box('.secs') }));
await page.evaluate(() => {
  const msgs = [...document.querySelectorAll('.chat__log > *')];
  const i = msgs.findIndex((m) => m.querySelector('.tiles'));
  if (i >= 0) msgs.slice(i).forEach((m) => { m.style.visibility = 'hidden'; });
});
await page.waitForTimeout(400);
await shot('chatA', async () => ({ camera: await box('.composer .icon-btn'), input: await box('.composer textarea, .composer input[type=text]') }));

// 03 the dashboard, zoomed out a little so the tiles and the 40-year chart share one frame
await go(A + '&tab=dashboard');
await page.evaluate(() => { document.documentElement.style.zoom = '0.86'; });
await page.waitForTimeout(3500);
await shot('dash', async () => ({
  temp: await box('.tile-m', 'Temperature'), aphid: await box('.tile-m', 'generations'), chart: await box('.dash__chartwrap'),
  rotTab: await box('button, a', 'Rotation', true), map: await box('.dash__map') }));

// 04 rotation: strategies, the pre-diagnosis and its "confirm" line
await go(A + '&tab=rotation');
await shot('rot', async () => ({
  cmp: await box('.rot__cmp'), strats: await box('.strats'), best: await nth('.strat', 1), dx: await box('.rot__dx'), confirm: await box('.rot__confirm'),
  export: await box('button, a', 'Download', true), rotTab: await box('button, a', 'Rotation', true), cites: await box('.rot__cmphead') }));

fs.writeFileSync(path.join(outDir, 'boxes.json'), JSON.stringify(meta, null, 1));
await b.close();
