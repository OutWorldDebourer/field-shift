// The section column (Your field, Soil, Pest care, Nutrition, Rotation, Technical sheet) with progress rings.
// States follow the guided steps when the backend sends them (pending / current topic / done, plus "with gaps"),
// else the section states; a done card shows the step summary. On phone, round buttons open a bottom sheet.
// Iteration 3: "What you told us" (facts) in Your field, and an Organic card below Pest care (not a guided topic).
import type { ClientAnalysis } from '../../shared/api';
import type { Bi, Section, SectionKey, SectionState, Step } from '../../shared/contract';
import { dataRow, dataRowCompact } from '../../shared/datarow';
import { esc } from '../../shared/dom';
import { pick, t } from '../../shared/i18n';
import { icon, type IconName } from '../../shared/icons';
import type { StrKey } from '../../shared/strings';
import { exportButtons } from './export';
import { keyReadings, shortLabel } from '../../shared/evidence';
import { displayState, getSection, ORDER, SHORT, stepsOf, TITLE } from './progress';
import { SEC_ICON } from './rail';
import { factsHtml } from './facts';
import { organicBody, organicState } from './organic';

export { getSection, ORDER };
const FRACTION: Record<SectionState, number> = { empty: 0, progress: 0.5, gaps: 1, ready: 1, locked: 0 };
const STATE_KEY: Record<SectionState, StrKey> = { empty: 'st_empty', progress: 'st_progress', ready: 'st_ready', gaps: 'st_gaps', locked: 'st_locked' };

export const sectionTitle = (k: SectionKey) => t(TITLE[k]);

export function ring(state: SectionState, size = 40, ic?: IconName): string {
  const r = size / 2 - 3, c = 2 * Math.PI * r, f = FRACTION[state];
  const inner = state === 'locked' ? icon('lock', 'ring__ico') : ic ? icon(ic, 'ring__ico') : '';
  return `<span class="ring ring--${state}" style="width:${size}px;height:${size}px" aria-hidden="true">
    <svg viewBox="0 0 ${size} ${size}"><circle class="ring__bg" cx="${size / 2}" cy="${size / 2}" r="${r}"/>
    ${f > 0 ? `<circle class="ring__fg" cx="${size / 2}" cy="${size / 2}" r="${r}" stroke-dasharray="${(c * f).toFixed(1)} ${c.toFixed(1)}" transform="rotate(-90 ${size / 2} ${size / 2})"/>` : ''}</svg>
    ${inner}${state === 'gaps' ? '<i class="ring__dot"></i>' : ''}${state === 'ready' ? `<i class="ring__check">${icon('check')}</i>` : ''}</span>`;
}

/** State label and note: with steps, "Current topic · I need …" / "Coming up" / step summary when done. */
function stateText(s: Section, st: SectionState, step?: Step, guided = false): { label: string; note: string; warn: boolean } {
  if (guided && step) {
    if (step.state === 'active') return { label: t('st_active'), note: step.needs.length ? t('rail_needs', { x: step.needs.map((n) => pick(n)).join(', ') }) : '', warn: false };
    if (step.state === 'pending') return { label: s.key === 'sheet' ? t('st_locked') : t('st_pending'), note: '', warn: false };
    return { label: t(STATE_KEY[st]), note: step.summary ? pick(step.summary) : s.note ? pick(s.note) : '', warn: st === 'gaps' };
  }
  return { label: t(STATE_KEY[st]), note: s.note ? pick(s.note) : '', warn: st === 'gaps' };
}

function stateLine(x: { label: string; note: string; warn: boolean }): string {
  return `<span class="sec__state">${esc(x.label)}</span>${x.note ? `<span class="sec__note${x.warn ? ' is-warn' : ''}">${esc(x.note)}</span>` : ''}`;
}

const guided = (a: ClientAnalysis) => Array.isArray(a.steps) && a.steps.length > 0;

function view(a: ClientAnalysis, k: SectionKey, steps: Step[]) {
  const s = getSection(a, k);
  const d = displayState(a, k, steps);
  return { s, st: d.state, step: d.step, text: stateText(s, d.state, d.step, guided(a)) };
}

type Extra = { declared_gaps?: Bi[] };
type ItemX = { ai_observation?: boolean };

/** Body of a section (expanded card or bottom sheet). */
export function sectionBody(a: ClientAnalysis, k: SectionKey): string {
  const s = getSection(a, k);
  const step = stepsOf(a).find((x) => x.key === k);
  const stepSummary = guided(a) && step?.state === 'done' && step.summary && pick(step.summary) !== pick(s.content?.summary) ? `<p class="sec__summary sec__summary--step">${icon('check')}${esc(pick(step.summary))}</p>` : '';
  if (k === 'field') {
    const rows = a.evidence?.readings ?? [];
    return `${stepSummary}${s.content ? `<p class="sec__summary">${esc(pick(s.content.summary))}</p>` : ''}
      ${factsHtml(a)}
      ${rows.length ? `<div class="sec__rows">${rows.map((r) => dataRow(r)).join('')}</div>` : `<p class="sec__empty">${esc(t('db_no_evidence'))}</p>`}
      ${gapsList(a.evidence?.gaps.map((g) => g.text) ?? [])}
      <button type="button" class="btn btn--glass btn--sm" data-goto="dashboard">${icon('dashboard')}${esc(t('ws_open_dash'))}</button>`;
  }
  if (k === 'sheet') {
    const locked = view(a, 'sheet', stepsOf(a)).st === 'locked';
    return `<p class="sec__summary">${esc(t('ws_sheet_body'))}</p>${locked ? `<p class="sec__empty">${esc(t('st_locked'))}</p>` : exportButtons(a)}`;
  }
  const c = s.content;
  if (!c) return `${stepSummary}<p class="sec__empty">${esc(s.note ? pick(s.note) : t('ws_no_content'))}</p>`;
  const items = c.items.map((it) => `<li class="sec__item${(it as ItemX).ai_observation ? ' is-ai' : ''}">
      <h4>${esc(pick(it.title))}${(it as ItemX).ai_observation ? `<span class="ai-tag mono">${esc(t('ai_obs'))}</span>` : ''}</h4><p>${esc(pick(it.body))}</p>
      ${it.citations?.length ? `<p class="sec__cites mono">${it.citations.map((ci) => `<a href="${esc(ci.url)}" target="_blank" rel="noopener">${esc(ci.title)}</a>`).join(' · ')}</p>` : ''}</li>`).join('');
  return `${stepSummary}<p class="sec__summary">${esc(pick(c.summary))}</p>${items ? `<ul class="sec__items">${items}</ul>` : ''}
    ${gapsList((c as unknown as Extra).declared_gaps ?? [])}
    ${k === 'rotation' ? `<button type="button" class="btn btn--glass btn--sm" data-goto="rotation">${icon('rotation')}${esc(t('ws_open_rotation'))}</button>` : ''}`;
}

function gapsList(g: Bi[]): string {
  if (!g.length) return '';
  return `<div class="sec__gaps"><h4 class="mono">${esc(t('ws_gaps_title'))}</h4><ul>${g.map((x) => `<li>${icon('info')}${esc(pick(x))}</li>`).join('')}</ul></div>`;
}

/** Desktop column. `just` = keys whose step just turned done (short check animation). */
/** Organic card: below Pest care, outside the six guided topics; state ready / in progress / not started. */
function organicCard(a: ClientAnalysis, isOpen: boolean): string {
  const st = organicState(a);
  const label = t(st === 'ready' ? 'st_ready' : st === 'gaps' ? 'st_gaps' : st === 'progress' ? 'st_progress' : 'og_waiting');
  return `<li class="sec sec--${st} sec--org${isOpen ? ' is-open' : ''}" data-seckey="organic">
    <button type="button" class="sec__head" aria-expanded="${isOpen}" aria-controls="sec-organic" data-sec="organic">
      ${ring(st, 40, 'ladybug')}
      <span class="sec__titles"><span class="sec__title">${esc(t('ap_tab_org'))}</span><span class="sec__state">${esc(label)}</span></span>
      ${icon('down', 'sec__chev')}
    </button>
    <div class="sec__body" id="sec-organic" ${isOpen ? '' : 'hidden'}>${isOpen ? organicBody(a) : ''}</div>
  </li>`;
}

export function sectionsColumn(a: ClientAnalysis, open: SectionKey | 'organic' | null, just: SectionKey[] = []): string {
  const steps = stepsOf(a);
  const views = ORDER.map((k) => view(a, k, steps));
  const done = views.filter((v) => v.st === 'ready' || v.st === 'gaps').length;
  const gaps = views.filter((v) => v.st === 'gaps').length;
  const preview = (k: SectionKey) => {
    if (k !== 'field' || !a.evidence) return '';
    const pickR = keyReadings(a.evidence);
    return pickR.length ? `<div class="sec__peek">${pickR.map((r) => dataRowCompact(r, undefined, shortLabel(r), true)).join('')}</div>` : '';
  };
  return `<header class="secs__head"><h2 class="serif">${esc(t('ws_sections'))}</h2>
      <span class="secs__count mono">${esc(gaps ? t('ws_progress_g', { done: done - gaps, total: views.length, g: gaps }) : t('ws_progress', { done, total: views.length }))}</span></header>
    <ul class="secs__list" data-secs-list>${views.map(({ s, st, step, text }) => {
      const isOpen = open === s.key;
      const stepCls = step ? ` is-step-${step.state}` : '';
      return `<li class="sec sec--${st}${stepCls}${isOpen ? ' is-open' : ''}${just.includes(s.key) ? ' is-just' : ''}" data-seckey="${s.key}">
        <button type="button" class="sec__head" aria-expanded="${isOpen}" aria-controls="sec-${s.key}" data-sec="${s.key}"${step?.state === 'active' ? ' aria-current="step"' : ''}>
          ${ring(st, 40, SEC_ICON[s.key])}
          <span class="sec__titles"><span class="sec__title">${esc(t(TITLE[s.key]))}${s.key === 'field' ? ` <span class="sec__sub mono">· ${esc(t('sec_field_sub'))}</span>` : ''}</span>${stateLine(text)}</span>
          ${icon('down', 'sec__chev')}
        </button>
        ${!isOpen ? preview(s.key) : ''}
        <div class="sec__body" id="sec-${s.key}" ${isOpen ? '' : 'hidden'}>${isOpen ? sectionBody(a, s.key) : ''}</div>
      </li>${s.key === 'pests' ? organicCard(a, open === 'organic') : ''}`;
    }).join('')}</ul><p class="secs__more" data-more aria-hidden="true">${icon('down')}${esc(t('ws_more_below'))}</p>`;
}

/** The section worth a glance on phone: the current topic, else one with gaps, else in progress, else the rotation. */
export function peekSection(a: ClientAnalysis): Section {
  const steps = stepsOf(a);
  if (guided(a)) { const act = steps.find((s) => s.state === 'active'); if (act) return getSection(a, act.key); }
  const all = ORDER.filter((k) => k !== 'sheet').map((k) => getSection(a, k));
  return all.find((s) => s.state === 'gaps') ?? all.find((s) => s.state === 'progress') ?? getSection(a, 'rotation');
}

export function peekHtml(a: ClientAnalysis, k: SectionKey): string {
  const { s, st, text } = view(a, k, stepsOf(a));
  const line = text.note || (s.content ? pick(s.content.summary) : text.label);
  return `${ring(st, 34, SEC_ICON[k])}<span class="peek__t"><b>${esc(t(TITLE[k]))} · ${esc(text.label)}</b><small>${esc(line)}</small></span>${icon('chevron', 'peek__chev')}`;
}

/** Phone row of round buttons. */
export function sectionsRow(a: ClientAnalysis, just: SectionKey[] = []): string {
  const steps = stepsOf(a);
  return ORDER.map((k) => {
    const { st, step, text } = view(a, k, steps);
    return `<li><button type="button" class="secbtn${just.includes(k) ? ' is-just' : ''}${step?.state === 'active' && guided(a) ? ' is-active' : ''}" data-sheet="${k}" aria-label="${esc(t(TITLE[k]))}: ${esc(text.label)}${text.note ? ' · ' + esc(text.note) : ''}">
      ${ring(st, 48, SEC_ICON[k])}<span>${esc(t(SHORT[k]))}</span></button></li>`;
  }).join('');
}

export function sheetHtml(a: ClientAnalysis, k: SectionKey): string {
  const { st, text } = view(a, k, stepsOf(a));
  return `<header class="sheet__head">${ring(st, 40, SEC_ICON[k])}<div><h2 class="serif" id="sheet-t">${esc(t(TITLE[k]))}</h2><p>${stateLine(text)}</p></div>
    <button type="button" class="icon-btn sheet__close" data-close aria-label="${esc(t('close'))}">${icon('close')}</button></header>
    <div class="sheet__body">${sectionBody(a, k)}</div>`;
}
