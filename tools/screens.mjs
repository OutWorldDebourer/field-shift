// Captures design-audit screenshots (desktop 1440x900 and mobile 390x844) of one URL with headless Chromium.
// Uses playwright-core from the Lemo-Opuscar library (no project dependency).
// Usage: node tools/screens.mjs <url> <out-prefix> [waitMs] [fullPage:0|1] [scrollY]
import { createRequire } from 'node:module';
const require = createRequire('C:/Users/OutWorldDevourer/lemo-opuscar/package.json');
const { chromium } = require('playwright-core');

const [url, prefix, waitMs = '2500', full = '0', scrollY = '0'] = process.argv.slice(2);
if (!url || !prefix) { console.error('usage: node tools/screens.mjs <url> <out-prefix> [waitMs] [fullPage] [scrollY]'); process.exit(2); }

const browser = await chromium.launch({ args: ['--use-angle=swiftshader', '--enable-unsafe-swiftshader', '--ignore-gpu-blocklist'] });
for (const [name, vp, mobile] of [['desktop', { width: 1440, height: 900 }, false], ['mobile', { width: 390, height: 844 }, true]]) {
  const ctx = await browser.newContext({ viewport: vp, deviceScaleFactor: mobile ? 2 : 1, isMobile: mobile, hasTouch: mobile });
  const page = await ctx.newPage();
  const errors = [];
  page.on('pageerror', (e) => errors.push(String(e)));
  page.on('console', (m) => { if (m.type() === 'error') errors.push(m.text()); });
  await page.goto(url, { waitUntil: 'networkidle', timeout: 60000 }).catch((e) => errors.push(String(e)));
  if (Number(scrollY)) await page.evaluate((y) => window.scrollTo(0, y), Number(scrollY));
  await page.waitForTimeout(Number(waitMs));
  const out = `${prefix}-${name}.jpg`;
  await page.screenshot({ path: out, type: 'jpeg', quality: 82, fullPage: full === '1' });
  console.log(out, errors.length ? `errors: ${errors.slice(0, 5).join(' | ')}` : 'no console errors');
  await ctx.close();
}
await browser.close();
