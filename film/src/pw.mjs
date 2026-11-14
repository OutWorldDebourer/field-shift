// playwright-core from the Lemo-Opuscar library (no install in the project)
import { createRequire } from 'module';
const LIB = process.env.LIB || 'C:/Users/OutWorldDevourer/lemo-opuscar';
const req = createRequire(LIB + '/package.json');
export const { chromium } = req('playwright-core');
export const ARGS = ['--enable-gpu', '--ignore-gpu-blocklist', '--font-render-hinting=none', '--force-color-profile=srgb'];
