// Bundles handbook/en/*.md and handbook/es/*.md into one self-contained index.html.
// Usage: node handbook/build.mjs
import { readFileSync, writeFileSync, readdirSync, existsSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

const root = dirname(fileURLToPath(import.meta.url));
const read = (...p) => readFileSync(join(root, ...p), 'utf8');
const LANGS = ['en', 'es'];

const order = readdirSync(join(root, 'en')).filter((f) => f.endsWith('.md')).sort().map((f) => f.slice(0, -3));
const pages = {};
const missing = [];
for (const slug of order) {
  pages[slug] = {};
  for (const lang of LANGS) {
    const file = join(root, lang, slug + '.md');
    if (!existsSync(file)) { missing.push(`${lang}/${slug}.md`); continue; }
    const md = readFileSync(file, 'utf8');
    const title = (/^#\s+(.+)$/m.exec(md) || [, slug])[1].trim();
    pages[slug][lang] = { title, md };
  }
}

const data = JSON.stringify({ order, pages, built: new Date().toISOString().slice(0, 10) })
  .replace(/</g, '\\u003c');
const html = read('site', 'template.html')
  .replace('/*__CSS__*/', () => read('site', 'viewer.css'))
  .replace('/*__DATA__*/', () => data)
  .replace('/*__JS__*/', () => read('site', 'viewer.js'));

writeFileSync(join(root, 'index.html'), html);
console.log(`handbook: ${order.length} pages -> index.html`);
if (missing.length) console.warn('missing translations:', missing.join(', '));
