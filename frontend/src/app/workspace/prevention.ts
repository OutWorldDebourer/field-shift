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
type PestRow = { name: Bi; scientific: string; light: Light; monitoring: Bi; generations_now?: number | null; generations_normal?: number | null };
type Stage = { stage: Bi; action: Bi };
type CropBlock = {
  crop: { key: string; name: Bi; crop_type: string; role: string };
  pests: PestRow[];
  calendar: { kind: 'year-round' | 'by-stage'; note: Bi; stages?: Stage[]; watch: Bi[] };
  researched: boolean; reviewed: boolean;
};
type PreventionContent = {
  title: Bi; summary: Bi; crops: CropBlock[]; notice: Bi;
  forecast?: { tmax: number; tmax_date: string; tmin: number; rain_mm: number; days: number; source: string; run: string } | null;
  temp_anomaly?: number | null; sources: { name: string; url: string }[];
};

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

function pestHtml(p: PestRow, crop: string): string {
  const L = LEVEL[p.light.level] ?? LEVEL.unknown;
  return `<li class="pv-pest pv-pest--${p.light.level}">
    <div class="pv-pest__h"><span class="pv-light" aria-hidden="true"></span><h4>${esc(pick(p.name))} <i>${esc(p.scientific)}</i></h4>
      <span class="pv-level">${icon(L.ic)}${esc(t(L.k))}</span></div>
    <p class="pv-pest__why">${esc(pick(p.light.reason))}</p>
    <details class="pv-more"><summary>${esc(t('pv_what_to_do'))}</summary>
      <ul class="pv-acts">
        <li>${icon('search')}<span>${esc(pick(p.monitoring))}</span></li>
        ${p.light.favourable ? `<li>${icon('thermo')}<span>${esc(t('pv_likes'))}: ${esc(pick(p.light.favourable))}${p.light.citation?.url ? ` · <a class="mono" href="${esc(p.light.citation.url)}" target="_blank" rel="noopener">${esc(p.light.citation.title || t('rt_sources'))}</a>` : ''}</span></li>` : ''}
        ${p.generations_now != null && p.generations_normal != null ? `<li>${icon('orbit')}<span class="mono">${esc(t('pv_gens', { now: p.generations_now.toFixed(1), norm: p.generations_normal.toFixed(1) }))}</span></li>` : ''}
        <li>${icon('ladybug')}<span><button type="button" class="pv-link" data-goto="organic">${esc(t('pv_allies'))}</button></span></li>
      </ul></details>
    ${askBtn(t('ask_pest', { x: `${pick(p.name)} (${crop})` }))}
  </li>`;
}

function calendarHtml(c: CropBlock): string {
  const cal = c.calendar;
  if (cal.kind === 'year-round') {
    const watch = cal.watch.map((w) => pick(w)).join(', ');
    return `<div class="pv-cal"><p class="pv-cal__n">${esc(pick(cal.note))}</p>${watch ? `<p class="pv-cal__w">${esc(t('pv_watch', { x: watch }))}</p>` : ''}</div>`;
  }
  return `<div class="pv-cal"><p class="pv-cal__n">${esc(pick(cal.note))}</p>
    <ol class="pv-stages">${(cal.stages ?? []).map((s) => `<li><b>${esc(pick(s.stage))}</b><span>${esc(pick(s.action))}</span></li>`).join('')}</ol></div>`;
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
      <ul class="pv-counts" aria-label="${esc(pick(c.summary))}">
        <li class="pv-c pv-c--high"><b>${n('high')}</b><span>${esc(t('pv_high'))}</span></li>
        <li class="pv-c pv-c--medium"><b>${n('medium')}</b><span>${esc(t('pv_medium'))}</span></li>
        <li class="pv-c pv-c--low"><b>${n('low')}</b><span>${esc(t('pv_low'))}</span></li>
      </ul>
      ${top ? `<p class="pv-top">${esc(t('pv_top', { p: pick(top.name) }))} ${esc(pick(top.light.reason))}</p>` : ''}
      ${f ? `<p class="pv-fc mono">${esc(t('pv_fc', { t: Math.round(f.tmax), d: f.tmax_date, r: f.rain_mm, n: f.days }))}</p>` : ''}
      <p class="notice og-notice">${icon('info')}<span>${esc(pick(c.notice))}</span></p>
      <p class="pv-src mono">${c.sources.map((s) => `<a href="${esc(s.url)}" target="_blank" rel="noopener">${esc(s.name)}</a>`).join(' · ')}</p>
    </section>
    <div class="pv-crops">${c.crops.map((b) => `<section class="pv-crop glass" aria-labelledby="pv-${esc(b.crop.key)}">
      <header class="pv-crop__h"><h3 class="serif" id="pv-${esc(b.crop.key)}">${esc(pick(b.crop.name))}</h3>
        <span class="pv-role mono">${esc(t(ROLE[b.crop.role] ?? 'pv_role_current'))}${b.crop.crop_type === 'perennial' ? ` · ${esc(t('pv_perennial'))}` : ''}</span>
        ${b.researched && !b.reviewed ? `<span class="pv-ai mono">${esc(t('og_ai_verify'))}</span>` : ''}</header>
      <ul class="pv-pests">${b.pests.map((p) => pestHtml(p, pick(b.crop.name))).join('')}</ul>
      <details class="pv-more pv-more--cal"><summary>${esc(t(b.calendar.kind === 'year-round' ? 'pv_cal_year' : 'pv_cal_stage'))}</summary>${calendarHtml(b)}</details>
    </section>`).join('')}</div>
  </div>`;
}
