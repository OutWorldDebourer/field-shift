// EN | ES switch: English by default, `?lang=es` or the saved choice override it. Strings live in ./strings.
import type { Bi, Lang } from './contract';
import { STRINGS, type StrKey } from './strings';

const KEY = 'fieldshift.lang';
type Listener = (l: Lang) => void;
const listeners = new Set<Listener>();

function initial(): Lang {
  try {
    const q = new URLSearchParams(location.search).get('lang');
    if (q === 'es' || q === 'en') return q;
  } catch { /* ignore */ }
  try {
    const s = localStorage.getItem(KEY);
    if (s === 'es' || s === 'en') return s;
  } catch { /* storage blocked */ }
  return 'en';
}

let current: Lang = initial();

export const lang = (): Lang => current;

export function setLang(l: Lang): void {
  if (l === current) return;
  current = l;
  try { localStorage.setItem(KEY, l); } catch { /* storage blocked */ }
  try {
    const u = new URL(location.href);
    if (u.searchParams.has('lang')) { u.searchParams.set('lang', l); history.replaceState(history.state, '', u); }
  } catch { /* ignore */ }
  document.documentElement.lang = l;
  applyI18n(document);
  listeners.forEach((fn) => fn(l));
}

export function onLang(fn: Listener): () => void {
  listeners.add(fn);
  return () => listeners.delete(fn);
}

/** Translate a UI key; `{name}` placeholders are filled from vars. */
export function t(key: StrKey, vars?: Record<string, string | number>): string {
  const s = STRINGS[key];
  let out = s ? s[current] : key;
  if (vars) for (const [k, v] of Object.entries(vars)) out = out.split(`{${k}}`).join(String(v));
  return out;
}

/** Pick the current language from a bilingual value sent by the backend. */
export const pick = (b: Bi | undefined | null): string => (b ? b[current] || b.en : '');

/** Update static markup: data-i18n (text) and data-i18n-attr="aria-label:key;placeholder:key". */
export function applyI18n(root: ParentNode): void {
  root.querySelectorAll<HTMLElement>('[data-i18n]').forEach((el) => { el.textContent = t(el.dataset.i18n as StrKey); });
  root.querySelectorAll<HTMLElement>('[data-i18n-html]').forEach((el) => { el.innerHTML = t(el.dataset.i18nHtml as StrKey); });
  root.querySelectorAll<HTMLElement>('[data-i18n-attr]').forEach((el) => {
    for (const pair of (el.dataset.i18nAttr || '').split(';')) {
      const [attr, key] = pair.split(':');
      if (attr && key) el.setAttribute(attr.trim(), t(key.trim() as StrKey));
    }
  });
}

document.documentElement.lang = current;

/** Format a number with the locale's decimal mark. */
export function fmt(n: number, digits = 1): string {
  return n.toLocaleString(current === 'es' ? 'es-PE' : 'en-US', { minimumFractionDigits: digits, maximumFractionDigits: digits });
}

/** Format an ISO date (or pass a period string through). */
export function fmtDate(iso: string): string {
  const range = iso.match(/^(\d{4}-\d{2}-\d{2})\s*(?:to|→|–|-{1,2})\s*(\d{4}-\d{2}-\d{2})$/);
  if (range) return `${fmtDate(range[1])} – ${fmtDate(range[2])}`;
  if (!/^\d{4}-\d{2}-\d{2}(T.*)?$/.test(iso)) return iso;
  const d = new Date(iso.slice(0, 10) + 'T12:00:00Z');
  if (isNaN(d.getTime())) return iso;
  return d.toLocaleDateString(current === 'es' ? 'es-PE' : 'en-GB', { day: 'numeric', month: 'short', year: 'numeric', timeZone: 'UTC' });
}

export function monthShort(i: number): string {
  const d = new Date(Date.UTC(2026, i, 15));
  return d.toLocaleDateString(current === 'es' ? 'es-PE' : 'en-US', { month: 'short', timeZone: 'UTC' }).replace('.', '');
}
