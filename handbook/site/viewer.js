// Handbook viewer: English by default, Spanish on demand. Data is embedded by build.mjs.
const DATA = JSON.parse(document.getElementById('pages').textContent);
const UI = {
  en: { tagline: 'Team handbook · NASA Space Apps 2026', built: 'Built', missing: 'This page has no English version yet.' },
  es: { tagline: 'Manual del equipo · NASA Space Apps 2026', built: 'Generado', missing: 'Esta página aún no tiene versión en español.' },
};
const store = {
  get(k) { try { return localStorage.getItem(k); } catch { return null; } },
  set(k, v) { try { localStorage.setItem(k, v); } catch { /* storage unavailable */ } },
};
let lang = store.get('handbook.lang') === 'es' ? 'es' : 'en';
let slug = (location.hash || '').slice(1) || DATA.order[0];

function render() {
  if (!DATA.order.includes(slug)) slug = DATA.order[0];
  document.documentElement.lang = lang;
  document.getElementById('tagline').textContent = UI[lang].tagline;
  document.querySelectorAll('.lang button').forEach((b) =>
    b.setAttribute('aria-pressed', String(b.dataset.lang === lang)));

  const nav = document.getElementById('nav');
  nav.replaceChildren(...DATA.order.map((s) => {
    const a = document.createElement('a');
    const p = DATA.pages[s][lang] || DATA.pages[s].en;
    a.href = '#' + s;
    a.textContent = p.title;
    if (s === slug) a.setAttribute('aria-current', 'page');
    return a;
  }));

  const page = DATA.pages[slug][lang];
  const el = document.getElementById('page');
  if (!page) {
    el.textContent = UI[lang].missing;
  } else if (window.marked) {
    el.innerHTML = window.marked.parse(page.md);
  } else {
    const pre = document.createElement('pre');
    pre.textContent = page.md;
    el.replaceChildren(pre);
  }
  el.querySelectorAll('table').forEach((t) => {
    const w = document.createElement('div');
    w.className = 'table-wrap';
    t.replaceWith(w);
    w.append(t);
  });
  el.querySelectorAll('a[href]').forEach((a) => {
    const href = a.getAttribute('href');
    const m = /^([\w-]+)\.md(#.*)?$/.exec(href);
    if (m && DATA.pages[m[1]]) a.setAttribute('href', '#' + m[1]);
    else if (/^https?:/.test(href)) { a.target = '_blank'; a.rel = 'noopener'; }
  });
  document.getElementById('foot').textContent = UI[lang].built + ' ' + DATA.built;
  document.title = (page ? page.title : slug) + ' · Field Shift';
}

document.querySelectorAll('.lang button').forEach((b) =>
  b.addEventListener('click', () => { lang = b.dataset.lang; store.set('handbook.lang', lang); render(); }));
window.addEventListener('hashchange', () => { slug = location.hash.slice(1); render(); window.scrollTo(0, 0); });
render();
