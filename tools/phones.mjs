// Phone check: opens the app on real phone profiles (iPhone on WebKit = Safari's engine, Android on Chromium),
// takes full-page screenshots and reports anything wider than the screen (the usual cause of "broken margins").
//
//   node tools/phones.mjs <base-url> <out-dir> [analysis-id] [lang]
//
// Pages: index, field picker (empty and with the example field), the five workspace views of an analysis, and the
// technical sheet. Output: <out-dir>/<device>-<page>.jpg and a JSON report of horizontal overflow per page.
import { createRequire } from 'node:module';
import { mkdirSync, writeFileSync } from 'node:fs';
const require = createRequire('C:/Users/OutWorldDevourer/lemo-opuscar/package.json');
const { chromium, webkit, devices } = require('playwright-core');

const [base = 'http://127.0.0.1:8787', out = 'phone-shots', id = '', lang = 'es'] = process.argv.slice(2);
mkdirSync(out, { recursive: true });
const DEVICES = ['iPhone 15', 'iPhone SE (3rd gen)', 'Pixel 7', 'Galaxy A55'];
const pages = [
  ['index', `/?lang=${lang}`, null],
  ['picker', `/app.html?demo=1&lang=${lang}`, null],
  ['picker-chosen', `/app.html?demo=1&lang=${lang}`, '[data-ex]'],
  ...(id ? ['chat', 'dashboard', 'rotation', 'prevention', 'organic'].map((v) => [v, `/app.html?demo=1&a=${id}&tab=${v}&lang=${lang}`, null]) : []),
  ...(id ? [['sheet', `/api/export/${id}/sheet?lang=${lang}`, null]] : []),
];

const report = {};
for (const name of DEVICES) {
  const d = devices[name];
  const engine = d.defaultBrowserType === 'webkit' ? webkit : chromium;
  // Use an installed WebKit build when the exact revision Playwright expects is not downloaded.
  const wk = process.env.LOCALAPPDATA + '/ms-playwright/webkit-2370/Playwright.exe';
  const browser = await engine.launch(engine === webkit && require('node:fs').existsSync(wk) ? { executablePath: wk } : {});
  const ctx = await browser.newContext({ ...d });
  for (const [key, path, click] of pages) {
    const p = await ctx.newPage();
    const errors = [];
    p.on('pageerror', (e) => errors.push(e.message));
    p.on('console', (m) => { if (m.type() === 'error') errors.push(m.text()); });
    try {
      await p.goto(base + path, { waitUntil: 'networkidle', timeout: 45000 }).catch(() => {});
      await p.waitForTimeout(2500);
      if (click) { await p.click(click).catch(() => {}); await p.waitForTimeout(1500); }
      const r = await p.evaluate(() => {
        const W = document.documentElement.clientWidth;
        const off = [];
        for (const el of document.querySelectorAll('body *')) {
          const b = el.getBoundingClientRect();
          if (!b.width || getComputedStyle(el).position === 'fixed' && b.right <= W + 1) continue;
          if (b.right > W + 1 || b.left < -1) {
            // keep only the outermost offender of a branch
            if (!off.some((o) => o.el.contains(el))) off.push({ el, d: `${el.tagName.toLowerCase()}${el.id ? '#' + el.id : ''}${el.classList.length ? '.' + [...el.classList].slice(0, 3).join('.') : ''}`, l: Math.round(b.left), r: Math.round(b.right) });
          }
        }
        return { W, scrollW: document.documentElement.scrollWidth, bodyScrollW: document.body.scrollWidth, offenders: off.slice(0, 12).map(({ d, l, r }) => ({ d, l, r })) };
      });
      const file = `${out}/${name.replace(/[^a-z0-9]+/gi, '_')}-${key}.jpg`;
      await p.screenshot({ path: file, fullPage: true, quality: 70, type: 'jpeg' }).catch(async () => p.screenshot({ path: file, quality: 70, type: 'jpeg' }));
      (report[name] ??= {})[key] = { ...r, errors: errors.slice(0, 5) };
      console.log(`${name.padEnd(20)} ${key.padEnd(14)} width ${r.W} scroll ${r.scrollW}${r.scrollW > r.W ? '  <-- OVERFLOW' : ''}${r.offenders.length ? '  offenders: ' + r.offenders.map((o) => o.d).slice(0, 4).join(', ') : ''}${errors.length ? '  errors: ' + errors.length : ''}`);
    } catch (e) { console.log(name, key, 'failed', e.message.slice(0, 120)); }
    await p.close();
  }
  await browser.close();
}
writeFileSync(`${out}/report.json`, JSON.stringify(report, null, 1));
