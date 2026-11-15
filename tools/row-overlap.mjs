// Checks the compact data rows (value + sparkline) for overlaps on phones, also with enlarged text as Android's
// font scaling does. Prints every row where the value's box crosses the sparkline or the label.
//   node tools/row-overlap.mjs <url> "<device>" [text scale, e.g. 1.3]
import { createRequire } from 'node:module';
const require = createRequire('C:/Users/OutWorldDevourer/lemo-opuscar/package.json');
const { chromium, webkit, devices } = require('playwright-core');
const fs = require('node:fs');
const [url, devName = 'Pixel 7', scale = '1'] = process.argv.slice(2);
const d = devices[devName];
const wk = process.env.LOCALAPPDATA + '/ms-playwright/webkit-2370/Playwright.exe';
const engine = d.defaultBrowserType === 'webkit' ? webkit : chromium;
const browser = await engine.launch(engine === webkit && fs.existsSync(wk) ? { executablePath: wk } : {});
const p = await (await browser.newContext({ ...d })).newPage();
await p.goto(url, { waitUntil: 'networkidle', timeout: 45000 }).catch(() => {});
if (+scale !== 1) await p.addStyleTag({ content: `html { font-size: ${Math.round(+scale * 100)}% !important; }` });
await p.waitForTimeout(2500);
const res = await p.evaluate(() => {
  const hit = (a, b) => a.right > b.left + 1 && a.left < b.right - 1 && a.bottom > b.top + 1 && a.top < b.bottom - 1;
  return [...document.querySelectorAll('.drow--compact')].filter((r) => r.getBoundingClientRect().width).map((r) => {
    // the value's text extent (its box can be narrower than the text when a grid column collapses)
    const ve = r.querySelector('.drow__value');
    let v;
    if (ve) { const rg = document.createRange(); rg.selectNodeContents(ve); const t = rg.getBoundingClientRect(), bx = ve.getBoundingClientRect(); v = { left: Math.min(t.left, bx.left), right: Math.max(t.right, bx.right), top: Math.min(t.top, bx.top), bottom: Math.max(t.bottom, bx.bottom) }; }
    const s = r.querySelector('svg.spark, .spark, svg:last-of-type')?.getBoundingClientRect();
    const l = r.querySelector('.drow__label')?.getBoundingClientRect();
    return { id: r.dataset.reading, value: r.querySelector('.drow__value')?.textContent.trim(), spark: !!(v && s && hit(v, s)), label: !!(v && l && hit(v, l)) };
  });
});
const bad = res.filter((r) => r.spark || r.label);
console.log(`${devName} x${scale}: ${res.length} rows, ${bad.length} overlapping${bad.length ? ' → ' + bad.map((b) => `${b.id} "${b.value}" ${b.spark ? '[spark]' : ''}${b.label ? '[label]' : ''}`).join(', ') : ''}`);
if (process.env.SHOT) await p.screenshot({ path: process.env.SHOT, quality: 75, type: 'jpeg' });
await browser.close();
