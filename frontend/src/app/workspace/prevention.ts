// Prevention view (iteration 4, phase D): for every crop in the conversation, each pest gets a traffic light
// (high / medium / low / unknown) with a plain reason built from NASA degree-days vs normal, the GEOS-FP 10-day
// forecast and the weather that favours the pest (cited crop research). A summary first, details one click away:
// monitoring frequency, the favourable conditions with their source, the calendar (all year for a perennial, by
// stage for an annual) and a link to the natural allies in Organic. A model of pressure, never a count of insects.
import type { ClientAnalysis } from '../../shared/api';
import type { Bi } from '../../shared/contract';
import { esc } from '../../shared/dom';
import { pick, t } from '../../shared/i18n';
import { icon } from '../../shared/icons';
import { askBtn } from '../../shared/viewctx';
import './prevention.css';

type Light = { level: 'high' | 'medium' | 'low' | 'unknown'; reason: Bi; favourable?: Bi | null; citation?: { title: string; url: string } | null };
type PestRow = { name: Bi; scientific: string; light: Light; monitoring: Bi; practice?: Bi; release?: Bi; also_on?: string[]; generations_now?: number | null; generations_normal?: number | null; season?: string | null };
type Stage = { stage: Bi; action: Bi };
type CropBlock = {
  crop: { key: string; name: Bi; crop_type: string; role: string };
  pests: PestRow[];
  calendar: { kind: 'year-round' | 'by-stage'; note: Bi; stages?: Stage[]; watch: Bi[] };
  researched: boolean; reviewed: boolean; note?: Bi | null;
};
type PreventionContent = {
  title: Bi; summary: Bi; crops: CropBlock[]; notice: Bi;
  forecast?: { tmax: number; tmax_date: string; tmin: number; rain_mm: number; days: number; source: string; run: string } | null;
  temp_anomaly?: number | null; sources: { name: string | Bi; url: string }[]; counts?: { high: number; medium: number; low: number; unknown: number };
};

const cap = (x: string) => x.charAt(0).toUpperCase() + x.slice(1);

const LEVEL: Record<Light['level'], { k: 'pv_high' | 'pv_medium' | 'pv_low' | 'pv_unknown'; ic: 'alert' | 'info' | 'check' }> = {
  high: { k: 'pv_high', ic: 'alert' }, medium: { k: 'pv_medium', ic: 'info' }, low: { k: 'pv_low', ic: 'check' }, unknown: { k: 'pv_unknown', ic: 'info' },
};
const ROLE: Record<string, 'pv_role_current' | 'pv_role_previous' | 'pv_role_next' | 'pv_role_proposal'> = {
  current: 'pv_role_current', previous: 'pv_role_previous', next: 'pv_role_next', proposal: 'pv_role_proposal',
};

export function preventionContent(a: ClientAnalysis): PreventionContent | null {
  const s = a.sections.find((x) => (x.key as string) === 'prevention');
  const c = s?.content as unknown as PreventionContent | undefined;
  return c && Array.isArray(c.crops) && c.crops.length ? c : null;
}

function pestHtml(p: PestRow, crop: string, id: string): string {
  const L = LEVEL[p.light.level] ?? LEVEL.unknown;
  return `<li class="pv-pest pv-pest--${p.light.level}" id="${id}" tabindex="-1">
    <div class="pv-pest__h"><span class="pv-light" aria-hidden="true"></span><h4>${esc(pick(p.name))} <i>${esc(p.scientific)}</i></h4>
      <span class="pv-level">${icon(L.ic)}${esc(t(L.k))}</span></div>
    <p class="pv-pest__why">${esc(pick(p.light.reason))}</p>
    <details class="pv-more"><summary>${esc(t('pv_what_to_do'))}</summary>
      <ul class="pv-acts">
        <li>${icon('search')}<span>${esc(pick(p.monitoring))}</span></li>
        ${p.practice ? `<li>${icon('leaf')}<span>${esc(pick(p.practice))}</span></li>` : ''}
        ${p.release ? `<li>${icon('ladybug')}<span>${esc(pick(p.release))} <button type="button" class="pv-link" data-goto="organic">${esc(t('pv_allies'))}</button></span></li>` : ''}
        ${p.light.favourable ? `<li>${icon('thermo')}<span>${esc(t('pv_likes'))}: ${esc(pick(p.light.favourable))}${p.light.citation?.url ? ` · <a class="mono" href="${esc(p.light.citation.url)}" target="_blank" rel="noopener">${esc(p.light.citation.title || t('rt_sources'))}</a>` : ''}</span></li>` : ''}
        ${p.generations_now != null && p.generations_normal != null ? `<li>${icon('orbit')}<span class="mono pv-gens">${esc(t('pv_gens', { now: p.generations_now.toFixed(1), norm: p.generations_normal.toFixed(1) }))}${p.season ? ` · ${esc(p.season.replace(/-/g, '\u2011'))}` : ''}</span></li>` : ''}
        ${p.also_on?.length ? `<li>${icon('info')}<span>${esc(t('pv_also', { x: p.also_on.join(', ') }))}</span></li>` : ''}
      </ul>${askBtn(t('ask_pest', { x: `${pick(p.name)} (${crop})` }))}</details>
  </li>`;
}

const MONTHS = { en: 'JFMAMJJASOND', es: 'EFMAMJJASOND' };
function calendarHtml(c: CropBlock, a: ClientAnalysis): string {
  const cal = c.calendar;
  if (cal.kind === 'year-round') {
    const watch = cal.watch.map((w) => pick(w)).join(', ');
    // Warm months: NASA POWER 1985–2024 normal mean of 20 °C or more (pests develop faster); checks weekly then.
    const norm = new Map<number, number>();
    for (const m of a.evidence?.climate_monthly ?? []) norm.set(Number(m.month.slice(5, 7)), m.normal_mean);
    const letters = (pick({ en: MONTHS.en, es: MONTHS.es }) || MONTHS.en).split('');
    const strip = letters.map((l, i) => { const v = norm.get(i + 1); const warm = v != null && v >= 20;
      return `<li class="pv-m${warm ? ' is-warm' : ''}" title="${v != null ? `${v.toFixed(1)} °C` : ''}"><b>${l}</b><span>${esc(t(warm ? 'pv_weekly' : 'pv_monthly'))}</span></li>`; }).join('');
    return `<div class="pv-cal"><p class="pv-cal__n">${esc(pick(cal.note))}</p><ol class="pv-months" aria-label="${esc(t('pv_cal_year'))}">${strip}</ol>
      <p class="pv-cal__src mono">${esc(t('pv_cal_src'))}</p>${watch ? `<p class="pv-cal__w">${esc(t('pv_watch', { x: watch }))}</p>` : ''}</div>`;
  }
  return `<div class="pv-cal"><p class="pv-cal__n">${esc(pick(cal.note))}</p>
    <ol class="pv-stages">${(cal.stages ?? []).map((s) => `<li><b>${esc(pick(s.stage))}</b><span>${esc(pick(s.action))}</span></li>`).join('')}</ol></div>`;
}

/** The element id of the first pest at `level` (the counts and the action line jump to it). */
function firstId(c: PreventionContent, level: string): string {
  for (const b of c.crops) { const j = b.pests.findIndex((p) => p.light.level === level); if (j >= 0) return `pv-${b.crop.key}-${j}`; }
  return '';
}

/** Count tiles and "See it" scroll to the pest card and focus it. */
export function wirePrevention(root: HTMLElement): void {
  root.addEventListener('click', (e) => {
    const b = (e.target as HTMLElement).closest<HTMLElement>('[data-pv-jump]');
    const el = b?.dataset.pvJump ? root.querySelector<HTMLElement>(`#${CSS.escape(b.dataset.pvJump)}`) : null;
    if (!el) return;
    el.scrollIntoView({ block: 'center', behavior: matchMedia('(prefers-reduced-motion: reduce)').matches ? 'auto' : 'smooth' });
    el.focus({ preventScroll: true });
  });
}

export function preventionHtml(a: ClientAnalysis): string {
  const c = preventionContent(a);
  if (!c) {
    return `<div class="pv pv--empty"><section class="og-empty glass"><span class="og-empty__ico" aria-hidden="true">${icon('alert')}</span>
      <h2 class="serif">${esc(t('pv_empty_t'))}</h2><p>${esc(t('pv_empty_b'))}</p>
      <div class="og-empty__a"><button type="button" class="btn btn--solid btn--sm" data-goto="chat">${icon('chat')}${esc(t('og_go_chat'))}</button></div></section></div>`;
  }
  const f = c.forecast;
  const all = c.crops.flatMap((b) => b.pests);
  const n = (l: string) => all.filter((p) => p.light.level === l).length;
  const top = c.crops[0]?.pests.find((p) => p.light.level === 'high') ?? c.crops[0]?.pests.find((p) => p.light.level === 'medium');
  return `<div class="pv">
    <section class="pv-sum glass" aria-labelledby="pv-h">
      <h2 class="serif" id="pv-h">${esc(t('pv_title'))}</h2>
      <ul class="pv-counts" aria-label="${esc(pick(c.summary))}">${(['high', 'medium', 'low', 'unknown'] as const).map((l) => {
        const id = firstId(c, l);
        return `<li><button type="button" class="pv-c pv-c--${l}"${id ? ` data-pv-jump="${id}"` : ' disabled'}><b>${n(l)}</b><span>${esc(t(LEVEL[l].k))}</span></button></li>`; }).join('')}</ul>
      ${top ? `<p class="pv-top">${esc(t(top.light.level === 'high' ? 'pv_do_high' : 'pv_do_medium', { p: pick(top.name) }))} <button type="button" class="pv-link" data-pv-jump="${firstId(c, top.light.level)}">${esc(t('pv_see'))}</button></p>` : ''}
      ${f ? `<p class="pv-fc mono">${esc(t('pv_fc', { t: Math.round(f.tmax), d: f.tmax_date, r: f.rain_mm, n: f.days }))}</p>` : ''}
      <p class="notice og-notice">${icon('info')}<span>${esc(pick(c.notice))}</span></p>
      <p class="pv-src mono">${c.sources.map((s) => `<a href="${esc(s.url)}" target="_blank" rel="noopener">${esc(typeof s.name === 'string' ? s.name : pick(s.name))}</a>`).join(' · ')}</p>
    </section>
    <div class="pv-crops">${c.crops.map((b) => `<section class="pv-crop glass" aria-labelledby="pv-${esc(b.crop.key)}">
      <header class="pv-crop__h"><h3 class="serif" id="pv-${esc(b.crop.key)}">${esc(cap(pick(b.crop.name)))}</h3>
        <span class="pv-role mono">${esc(t(ROLE[b.crop.role] ?? 'pv_role_current'))}${b.crop.crop_type === 'perennial' ? ` · ${esc(t('pv_perennial'))}` : ''}</span>
        <span class="pv-ai mono">${esc(t(b.reviewed ? 'pv_reviewed' : b.researched ? 'og_ai_verify' : 'pv_not_researched'))}</span></header>
      ${b.note ? `<p class="pv-crop__note">${esc(pick(b.note))}</p>` : ''}
      <ul class="pv-pests">${b.pests.map((p, j) => pestHtml(p, pick(b.crop.name), `pv-${b.crop.key}-${j}`)).join('')}</ul>
      <details class="pv-more pv-more--cal"><summary>${esc(t(b.calendar.kind === 'year-round' ? 'pv_cal_year' : 'pv_cal_stage'))}</summary>${calendarHtml(b, a)}</details>
    </section>`).join('')}</div>
  </div>`;
}
