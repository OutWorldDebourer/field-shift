// Screen-by-screen phone capture (what a person sees while scrolling): one screenshot per viewport height.
//   node tools/phone-scroll.mjs <url> <out-prefix> "<device>" [max screens] [scroll selector]
import { createRequire } from 'node:module';
const require = createRequire('C:/Users/OutWorldDevourer/lemo-opuscar/package.json');
const { chromium, webkit, devices } = require('playwright-core');
const fs = require('node:fs');

const [url, prefix, devName = 'iPhone SE (3rd gen)', max = '8', sel = ''] = process.argv.slice(2);
const d = devices[devName];
const wk = process.env.LOCALAPPDATA + '/ms-playwright/webkit-2370/Playwright.exe';
const engine = d.defaultBrowserType === 'webkit' ? webkit : chromium;
const browser = await engine.launch(engine === webkit && fs.existsSync(wk) ? { executablePath: wk } : {});
const p = await (await browser.newContext({ ...d })).newPage();
await p.goto(url, { waitUntil: 'networkidle', timeout: 45000 }).catch(() => {});
await p.waitForTimeout(2500);
for (let i = 0; i < +max; i++) {
  await p.screenshot({ path: `${prefix}-${String(i).padStart(2, '0')}.jpg`, quality: 70, type: 'jpeg' });
  const moved = await p.evaluate((s) => {
    const el = s ? document.querySelector(s) : null;
    const t = el ?? document.scrollingElement;
    const before = t.scrollTop;
    t.scrollBy(0, innerHeight * 0.9);
    return t.scrollTop !== before;
  }, sel);
  if (!moved) break;
  await p.waitForTimeout(1200);
}
await browser.close();
