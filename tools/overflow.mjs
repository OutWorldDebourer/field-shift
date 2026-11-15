// Finds what really makes a page wider than a phone screen: elements whose right edge passes the viewport and
// that no ancestor clips (overflow hidden/auto/clip). Scrolls through the page so lazy sections are measured.
//   node tools/overflow.mjs <url> "<device>"
import { createRequire } from 'node:module';
const require = createRequire('C:/Users/OutWorldDevourer/lemo-opuscar/package.json');
const { chromium, webkit, devices } = require('playwright-core');
const fs = require('node:fs');
const [url, devName = 'iPhone SE (3rd gen)'] = process.argv.slice(2);
const d = devices[devName];
const wk = process.env.LOCALAPPDATA + '/ms-playwright/webkit-2370/Playwright.exe';
const engine = d.defaultBrowserType === 'webkit' ? webkit : chromium;
const browser = await engine.launch(engine === webkit && fs.existsSync(wk) ? { executablePath: wk } : {});
const p = await (await browser.newContext({ ...d })).newPage();
await p.goto(url, { waitUntil: 'networkidle', timeout: 45000 }).catch(() => {});
await p.waitForTimeout(2000);
const found = new Map();
for (let i = 0; i < 30; i++) {
  const r = await p.evaluate(() => {
    const W = document.documentElement.clientWidth;
    const clipped = (el) => { for (let a = el.parentElement; a && a !== document.body; a = a.parentElement) { const s = getComputedStyle(a); if (/(hidden|auto|scroll|clip)/.test(s.overflowX)) return true; } return false; };
    const out = [];
    for (const el of document.querySelectorAll('body *')) {
      const b = el.getBoundingClientRect();
      if (!b.width || b.right <= W + 0.5) continue;
      if (clipped(el)) continue;
      const path = []; for (let a = el; a && a !== document.body && path.length < 4; a = a.parentElement) path.unshift(a.tagName.toLowerCase() + (a.id ? '#' + a.id : '') + (a.classList.length ? '.' + [...a.classList].slice(0, 2).join('.') : ''));
      out.push({ path: path.join(' > '), right: Math.round(b.right), W, text: (el.textContent || '').trim().slice(0, 40) });
    }
    const moved = scrollY; scrollBy(0, innerHeight * 0.9);
    return { out, more: scrollY !== moved, scrollW: document.documentElement.scrollWidth };
  });
  for (const o of r.out) if (!found.has(o.path)) found.set(o.path, o);
  if (!r.more) break;
  await p.waitForTimeout(500);
}
console.log([...found.values()].map((o) => `${o.right}>${o.W}  ${o.path}  "${o.text}"`).join('\n') || 'no unclipped overflow');
await browser.close();
