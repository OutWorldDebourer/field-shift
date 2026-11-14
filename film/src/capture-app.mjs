// Screenshots of the REAL app (served by the backend from frontend/dist) at 2x, plus the measured boxes
// (CSS px in a 1440x900 viewport) of every element the film's cursor and reticle go to.
// usage: node capture-app.mjs <outDir> [analysisId]
import fs from 'fs'; import path from 'path';
import { chromium, ARGS } from './pw.mjs';
const [outDir, id = fs.readFileSync(new URL('../../research/cache/last_id', import.meta.url), 'utf8').trim()] = process.argv.slice(2);
const BASE = process.env.APP_BASE || 'http://127.0.0.1:8787/';
fs.mkdirSync(outDir, { recursive: true });
const b = await chromium.launch({ args: ARGS });
const page = await b.newPage({ viewport: { width: 1440, height: 900 }, deviceScaleFactor: 2 });
page.on('pageerror', e => console.error('pageerror', e.message));
// smallest visible element whose text includes `text` (optionally inside `within`)
const rect = (text, sel = '*', within = 'body') => page.evaluate(([text, sel, within]) => {
  const root = document.querySelector(within) || document.body;
  let best = null;
  for (const e of root.querySelectorAll(sel)) {
    const r = e.getBoundingClientRect(); if (r.width < 2 || r.height < 2) continue;
    if (text && !e.textContent.includes(text)) continue;
    if (!best || r.width * r.height < best.w * best.h) best = { x: r.x, y: r.y, w: r.width, h: r.height };
  }
  return best;
}, [text, sel, within]);
const box = (sel, text = null) => page.evaluate(([sel, text]) => {
  for (const e of document.querySelectorAll(sel)) {
    if (text && !e.textContent.toLowerCase().includes(text.toLowerCase())) continue;
    const r = e.getBoundingClientRect(); if (r.width < 2 || r.height < 2) continue;
    return { x: r.x, y: r.y, w: r.width, h: r.height };
  }
  return null;
}, [sel, text]);
const takes = {
  field: { url: 'app.html', boxes: async () => ({
    use: await rect('Use this field', 'button,a'), area: await rect('5.1 ha', 'b,strong,span,div,p'),
    example: await rect('Example site', 'button'), ret: await box('.reticle') }) },
  chat: { url: 'app.html?a=demo-huaral', boxes: async () => ({
    photo3: await page.evaluate(() => { const t = [...document.querySelectorAll('.chat__log figure')]; const r = t[2]?.getBoundingClientRect(); return r && { x: r.x, y: r.y, w: r.width, h: r.height }; }),
    photos: await page.evaluate(() => { const t = [...document.querySelectorAll('.chat__log figure')].map(f => f.getBoundingClientRect()); if (!t.length) return null; const x = Math.min(...t.map(r => r.x)), y = Math.min(...t.map(r => r.y)), X = Math.max(...t.map(r => r.right)), Y = Math.max(...t.map(r => r.bottom)); return { x, y, w: X - x, h: Y - y }; }),
    reply: await rect('Photo 3 is blurry', '.bubble'), temp: await box('aside .drow', 'temperature'),
    dashTab: await rect('Dashboard', 'button,a'), panel: await box('aside') }) },
  dash: { url: `app.html?a=${id}&tab=dashboard`, boxes: async () => ({
    temp: await box('.dash__tiles > li', 'temperature'), aphid: await box('.dash__tiles > li', 'aphid'),
    chart: await box('.dash__chart'), rotTab: await rect('Rotation', 'button,a'), map: await box('.dash__map') }) },
  rot: { url: `app.html?a=${id}&tab=rotation`, boxes: async () => ({
    cmp: await box('.rot__cmp'), strats: await box('.strats'), best: await box('.strat.is-best'),
    dx: await box('.rot__dx'), confirm: await box('.rot__confirm'), export: await rect('Export technical sheet', 'button,a'),
    rotTab: await rect('Rotation', 'button,a'), cites: await box('.rot__foot p') }) },
  // the same chat a moment before the photos arrive: the photo message and everything after it not yet in the log
  chatA: { url: 'app.html?a=demo-huaral', prep: () => page.evaluate(() => {
      const msgs = [...document.querySelectorAll('.chat__log > *')];
      const i = msgs.findIndex(m => m.querySelector('figure') || /Here are my plants/.test(m.textContent));
      if (i >= 0) msgs.slice(i).forEach(m => { m.style.display = 'none'; });
    }), boxes: async () => ({ camera: await page.evaluate(() => { const b = [...document.querySelectorAll('button,label,[role=button],input[type=file]+*')].map(b => b.getBoundingClientRect()).find(r => r.x < 140 && r.y > 650 && r.width > 20); return b && { x: b.x, y: b.y, w: b.width, h: b.height }; }), input: await box('textarea, input[type=text]') }) },
};
const meta = {};
for (const [name, tk] of Object.entries(takes)) {
  await page.goto(BASE + tk.url, { waitUntil: 'networkidle' });
  await page.waitForTimeout(2500);
  await page.mouse.move(-10, -10);
  await page.evaluate(() => { const l = document.querySelector('.chat__log'); if (l) l.scrollTop = 0; });
  if (tk.prep) await tk.prep();
  await page.waitForTimeout(600);
  await page.screenshot({ path: path.join(outDir, name + '.jpg'), type: 'jpeg', quality: 90 });
  meta[name] = await tk.boxes();
  console.log(name, JSON.stringify(meta[name]));
}
fs.writeFileSync(path.join(outDir, 'boxes.json'), JSON.stringify(meta, null, 1));
await b.close();
