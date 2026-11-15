// Captures design-audit screenshots (desktop 1440x900 and mobile 390x844) of one URL with headless Chromium.
// Uses playwright-core from the Lemo-Opuscar library (no project dependency).
// Full-page captures taller than Chromium's ~8000 px limit are saved as slices: <prefix>-<device>-p1.jpg, -p2.jpg, ...
// Usage: node tools/screens.mjs <url> <out-prefix> [waitMs] [fullPage:0|1] [scrollY] [scrollSelector] [clickSelectors]
// clickSelectors: comma-separated selectors clicked in order (900 ms apart) after the wait; prefix one with hover: to hover it.
// With a scrollSelector, the page is captured as viewport slices while scrolling that inner element: -s1.jpg, -s2.jpg, ...
import { createRequire } from 'node:module';
const require = createRequire('C:/Users/OutWorldDevourer/lemo-opuscar/package.json');
const { chromium } = require('playwright-core');

const [url, prefix, waitMs = '2500', full = '0', scrollY = '0', scrollSel = '', clicks = ''] = process.argv.slice(2);
if (!url || !prefix) { console.error('usage: node tools/screens.mjs <url> <out-prefix> [waitMs] [fullPage] [scrollY]'); process.exit(2); }
const SLICE = 7600;

const browser = await chromium.launch({ args: ['--use-angle=swiftshader', '--enable-unsafe-swiftshader', '--ignore-gpu-blocklist'] });
for (const [name, vp, mobile] of [['desktop', { width: 1440, height: 900 }, false], ['mobile', { width: 390, height: 844 }, true]]) {
  const ctx = await browser.newContext({ viewport: vp, deviceScaleFactor: mobile && full !== '1' ? 2 : 1, isMobile: mobile, hasTouch: mobile });
  const page = await ctx.newPage();
  const errors = [];
  page.on('pageerror', (e) => errors.push(String(e)));
  page.on('console', (m) => { if (m.type() === 'error') errors.push(m.text()); });
  await page.goto(url, { waitUntil: 'networkidle', timeout: 60000 }).catch((e) => errors.push(String(e)));
  if (Number(scrollY)) await page.evaluate((y) => window.scrollTo(0, y), Number(scrollY));
  await page.waitForTimeout(Number(waitMs));
  for (const sel of clicks.split(',').map((x) => x.trim()).filter(Boolean)) {
    const hover = sel.startsWith('hover:');
    const target = hover ? sel.slice(6) : sel;
    await (hover ? page.hover(target, { timeout: 5000 }) : page.click(target, { timeout: 5000 })).catch((e) => errors.push(`${hover ? 'hover' : 'click'} ${target}: ${String(e).slice(0, 80)}`));
    await page.waitForTimeout(900);
  }
  const outs = [];
  if (scrollSel) {
    const n = await page.evaluate((sel) => { const e = document.querySelector(sel); return e ? Math.ceil(e.scrollHeight / Math.max(1, e.clientHeight * 0.85)) : 0; }, scrollSel);
    if (!n) errors.push(`no element ${scrollSel}`);
    for (let i = 0; i < Math.min(n, 8); i++) {
      await page.evaluate(([sel, k]) => { const e = document.querySelector(sel); e.scrollTop = k * e.clientHeight * 0.85; }, [scrollSel, i]);
      await page.waitForTimeout(700);
      const out = `${prefix}-${name}-s${i + 1}.jpg`;
      await page.screenshot({ path: out, type: 'jpeg', quality: 82 });
      outs.push(out);
    }
  } else if (full === '1') {
    const h = await page.evaluate(() => document.documentElement.scrollHeight);
    const w = await page.evaluate(() => document.documentElement.scrollWidth);
    if (w > vp.width + 1) errors.push(`horizontal overflow: page is ${w}px wide`);
    if (h <= SLICE) {
      outs.push(`${prefix}-${name}.jpg`);
      await page.screenshot({ path: outs[0], type: 'jpeg', quality: 82, fullPage: true });
    } else {
      for (let y = 0, i = 1; y < h; y += SLICE, i++) {
        const out = `${prefix}-${name}-p${i}.jpg`;
        await page.screenshot({ path: out, type: 'jpeg', quality: 82, fullPage: true, clip: { x: 0, y, width: vp.width, height: Math.min(SLICE, h - y) } });
        outs.push(out);
      }
    }
  } else {
    outs.push(`${prefix}-${name}.jpg`);
    await page.screenshot({ path: outs[0], type: 'jpeg', quality: 82 });
  }
  console.log(outs.join(' '), errors.length ? `errors: ${errors.slice(0, 5).join(' | ')}` : 'no console errors');
  await ctx.close();
}
await browser.close();
