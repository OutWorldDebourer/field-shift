// Screenshots of the REAL live app (fieldshift.earth, bundled example analysis `demo-huaral`) at 2x in a 1440x900 viewport,
// plus the measured boxes (CSS px) of every element the film's cursor and reticle go to.
// usage: node capture.mjs <outDir> [en|es]      (APP_BASE overrides the site)
import fs from 'fs'; import path from 'path';
import { chromium, ARGS } from '../../src/pw.mjs';
const outDir = process.argv[2], LANG = process.argv[3] === 'es' ? 'es' : 'en';
// the UI text each box is found by, per language (frontend/src/shared/strings.ts)
const TX = {
  en: { gps: 'My GPS', example: 'Example site', point: 'Drop a point', outline: 'Draw outline', use: 'Use this field', loadInfo: 'Load information', loadThese: 'Load these',
    dunno: "I don't know", crop: 'Crop now', before: 'Crops before', water: 'How you water', dashTab: 'Dashboard', temp: 'Temperature', soil: 'Soil moisture', rain: 'Rain',
    vigor: 'Crop vigor', wbal: 'Water balance', gens: 'generations', smBtn: 'Soil moisture', meaning: 'What does it mean', fungi: 'fungi', download: 'Download', sheet: 'sheet', blurry: 'blurry' },
  es: { gps: 'Mi GPS', example: 'Sitio de ejemplo', point: 'Marcar un punto', outline: 'Dibujar contorno', use: 'Usar este campo', loadInfo: 'Cargar información', loadThese: 'Cargar estas',
    dunno: 'No sé', crop: 'Cultivo actual', before: 'Cultivos anteriores', water: 'Cómo riegas', dashTab: 'Panel', temp: 'Temperatura', soil: 'Humedad del suelo', rain: 'Lluvia',
    vigor: 'Vigor del cultivo', wbal: 'Balance hídrico', gens: 'generaciones', smBtn: 'Humedad', meaning: 'Qué significa', fungi: 'Hongos', download: 'Descargar', sheet: 'Ficha', blurry: 'borrosa' },
}[LANG];
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
const go = async (u) => { await page.goto(BASE + u, { waitUntil: 'networkidle' }); await page.waitForTimeout(3000); };
const shot = async (name, boxes) => {
  await page.mouse.move(-10, -10); await page.waitForTimeout(500);
  await page.screenshot({ path: path.join(outDir, name + '.jpg'), type: 'jpeg', quality: 90 });
  meta[name] = boxes ? await boxes() : {};
  console.log(name, JSON.stringify(meta[name]));
};
const meta = {};
const A = `app.html?a=demo-huaral&lang=${LANG}`;
const nth = (sel, i) => page.evaluate(([sel, i]) => { const e = document.querySelectorAll(sel)[i]; const r = e && e.getBoundingClientRect(); return r && { x: r.x, y: r.y, w: r.width, h: r.height }; }, [sel, i]);

// 01 field picker: as it opens, then with the example field chosen
await go(`app.html?lang=${LANG}`);
await shot('pick0', async () => ({
  gps: await box('button', TX.gps), example: await box('button', TX.example),
  card: await box('.fs__examples li', 'Huaral'), search: await box('input'), point: await box('button', TX.point),
  outline: await box('button', TX.outline), use: await box('button', TX.use) }));
await page.click('.fs__examples li');
await page.waitForTimeout(4500);
await shot('pick1', async () => ({
  use: await box('button', TX.use), area: await box('*', ' ha', true), ret: await box('.reticle'),
  outline: await box('button', TX.outline), point: await box('button', TX.point) }));

// 02 load information (the intake form, pre-filled with the example's answers)
await go(A + '&tab=chat');
await page.click(`button:has-text("${TX.loadInfo}")`);
await page.waitForTimeout(2500);
await shot('intake', async () => ({
  crop: await box('label, .intake *', TX.crop, true), before: await box('label, .intake *', TX.before, true),
  dunno: await box('button', TX.dunno), water: await box('label, .intake *', TX.water, true),
  load: await box('button', TX.loadThese), panel: await box('[role=dialog], .intake') }));

// 02 chat: the photo message and the consultant asking for a closer leaf
await go(A + '&tab=chat');
await page.evaluate(() => { const t = document.querySelector('.chat__log .tiles'); if (t) t.scrollIntoView({ block: 'center' }); });
await page.waitForTimeout(1200);
await shot('chat', async () => ({
  tiles: await box('.chat__log .tiles'), photo3: await page.evaluate(() => { const t = [...document.querySelectorAll('.chat__log .tiles .tile')][2]; const r = t && t.getBoundingClientRect(); return r && { x: r.x, y: r.y, w: r.width, h: r.height }; }),
  blurry: await box('.bubble', TX.blurry), camera: await box('.composer .icon-btn'), mic: await box('.composer__mic'),
  send: await box('.composer__send'), log: await box('.chat__log'), secs: await box('.secs'), dashTab: await box('button, a', TX.dashTab, true) }));

// 03 dashboard: tiles and map, then the soil-moisture layer, then the 40-year chart
await go(A + '&tab=dashboard');
const tiles = async () => ({
  temp: await box('.tile-m', TX.temp), soil: await box('.tile-m', TX.soil), rain: await box('.tile-m', TX.rain),
  vigor: await box('.tile-m', TX.vigor), water: await box('.tile-m', TX.wbal), aphid: await box('.tile-m', TX.gens),
  map: await box('.dash__map'), ret: await box('.dash__map .reticle'), legend: await box('.dmap__legend'),
  layers: await box('.dmap__layers'), smBtn: await box('.dmap__layers button', TX.smBtn), meaning: await box('button', TX.meaning) });
await shot('dash', tiles);
await page.click(`.dmap__layers button:has-text("${TX.smBtn}")`);
await page.waitForTimeout(3500);
await shot('dashSm', async () => ({ map: await box('.dash__map'), legend: await box('.dmap__legend'), ret: await box('.dash__map .reticle') }));
await page.evaluate(() => { const c = document.querySelector('.dash__chartwrap'); if (c) c.scrollIntoView({ block: 'center' }); });
await page.waitForTimeout(1500);
await shot('dashChart', async () => ({ chart: await box('.dash__chartwrap'), head: await box('.dash__ch'), legend: await box('.legend') }));

// 04 plan: prevention, organic, rotation (with the "What if I grow…?" avocado card)
await go(A + '&tab=prevention');
await shot('prev', async () => ({
  sum: await box('.pv-sum'), counts: await box('.pv-counts'), fc: await box('.pv-fc'), top: await box('.pv-top'),
  pest: await nth('.pv-pest', 0), level: await box('.pv-level'), notice: await box('.pv-sum .notice') }));
await go(A + '&tab=organic');
await shot('org', async () => ({
  week: await box('.og-week'), aphids: await nth('.og-cat', 0), fungi: await box('.og-cat', TX.fungi),
  tricho: await box('.og-week__l li, .og-week__l *', 'Trichoderma', true), gaps: await box('.og-gaps') }));
await go(A + '&tab=rotation');
await shot('rot', async () => ({
  strats: await box('.strats'), cur: await nth('.strat', 0), best: await nth('.strat', 1),
  green: await nth('.strat', 2), avo: await nth('.strat', 3), evalBox: await box('.rot__eval'),
  input: await box('.rot__eval input'), dx: await box('.rot__dx'), confirm: await box('.rot__confirm'), head: await box('.rot__cmphead'),
  download: await box('button, a', TX.download, true) }));

// 05 download menu
await page.click(`button:has-text("${TX.download}")`);
await page.waitForTimeout(1500);
await shot('dl', async () => ({ menu: await box('[role=menu], .dlmenu, .downloadmenu, .dl'), sheet: await box('[role=menuitem], button, a', TX.sheet, true) }));

fs.writeFileSync(path.join(outDir, 'boxes.json'), JSON.stringify(meta, null, 1));
await b.close();
