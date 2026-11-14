// "What does it mean?" — ⓘ buttons on data rows and tiles open a bilingual glossary panel (GET /api/glossary,
// bundled copy as fallback). Opening from a row scrolls to that term. Buttons render only after enableGlossary().
import { getGlossary } from './api';
import type { GlossaryTerm } from './contract';
import { esc, reducedMotion } from './dom';
import { onLang, pick, t } from './i18n';
import { icon } from './icons';
import { GLOSSARY, mergeGlossary } from './glossary';
import './glosspanel.css';

let enabled = false;
let terms: GlossaryTerm[] = GLOSSARY;
let loaded = false;
let panel: HTMLElement | null = null;
let returnTo: HTMLElement | null = null;

/** Small ⓘ button; empty string until the glossary is enabled on this page. */
export function glossBtn(key: string, about?: string): string {
  if (!enabled) return '';
  const label = about ? t('gl_btn_for', { x: about }) : t('gl_btn');
  return `<button type="button" class="gl-i" data-gloss="${esc(key)}" aria-label="${esc(label)}" title="${esc(label)}">${icon('info')}</button>`;
}

/** Labelled button for headers: ⓘ What does it mean? */
export function glossHeaderBtn(key = ''): string {
  return `<button type="button" class="btn btn--glass btn--sm gl-head" data-gloss="${esc(key)}">${icon('book')}<span>${esc(t('gl_btn'))}</span></button>`;
}

function find(key: string): GlossaryTerm | undefined {
  if (!key) return undefined;
  const k = key.toLowerCase();
  return terms.find((x) => x.key.toLowerCase() === k) ?? terms.find((x) => x.key.toLowerCase().includes(k) || x.term.en.toLowerCase().includes(k));
}

function render(active?: string) {
  if (!panel) return;
  panel.innerHTML = `<header class="gl__head"><div><h2 class="serif" id="gl-t">${esc(t('gl_title'))}</h2><p>${esc(t('gl_lede'))}</p></div>
      <button type="button" class="icon-btn" data-gl-close aria-label="${esc(t('close'))}">${icon('close')}</button></header>
    <dl class="gl__list">${terms.map((x) => `<div class="gl__term${x.key === active ? ' is-on' : ''}" id="gl-${esc(x.key)}">
      <dt>${esc(pick(x.term))}</dt><dd>${esc(pick(x.short))}${x.scale ? `<span class="gl__scale"><b class="mono">${esc(t('gl_scale'))}</b> ${esc(pick(x.scale))}</span>` : ''}</dd></div>`).join('')}</dl>`;
}

export function openGlossary(key: string, from: HTMLElement | null): void {
  if (!panel) return;
  returnTo = from;
  const hit = find(key);
  render(hit?.key);
  panel.hidden = false;
  document.querySelector('.gl-scrim')?.removeAttribute('hidden');
  requestAnimationFrame(() => {
    panel!.classList.add('is-open');
    (panel!.querySelector('[data-gl-close]') as HTMLElement | null)?.focus();
    if (hit) panel!.querySelector(`#gl-${CSS.escape(hit.key)}`)?.scrollIntoView({ block: 'center', behavior: reducedMotion() ? 'auto' : 'smooth' });
  });
  if (!loaded) {
    loaded = true;
    void getGlossary().then((r) => { terms = mergeGlossary(r); if (!panel!.hidden) { render(find(key)?.key); panel!.querySelector(`#gl-${CSS.escape(find(key)?.key ?? '')}`)?.scrollIntoView({ block: 'center' }); } });
  }
}

export function closeGlossary(): void {
  if (!panel || panel.hidden) return;
  panel.classList.remove('is-open');
  document.querySelector('.gl-scrim')?.setAttribute('hidden', '');
  setTimeout(() => { if (panel) panel.hidden = true; }, 260);
  returnTo?.focus();
}

/** Turn the ⓘ buttons on and wire the panel once for this page. */
export function enableGlossary(root: HTMLElement): void {
  enabled = true;
  if (panel) return;
  root.insertAdjacentHTML('beforeend', `<div class="gl-scrim" hidden></div><aside class="gl glass" role="dialog" aria-modal="true" aria-labelledby="gl-t" hidden></aside>`);
  panel = root.querySelector('.gl');
  root.addEventListener('click', (e) => {
    const el = e.target as HTMLElement;
    const b = el.closest<HTMLElement>('[data-gloss]');
    if (b) { e.preventDefault(); e.stopPropagation(); openGlossary(b.dataset.gloss ?? '', b); return; }
    if (el.closest('[data-gl-close]') || el.classList.contains('gl-scrim')) closeGlossary();
  }, true);
  root.addEventListener('keydown', (e) => { if (e.key === 'Escape' && panel && !panel.hidden) { e.stopPropagation(); closeGlossary(); } });
  onLang(() => { if (panel && !panel.hidden) render(panel.querySelector('.gl__term.is-on')?.id.slice(3)); });
}
