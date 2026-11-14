// The section column (Your field, Soil, Pest care, Nutrition, Rotation, Technical sheet) with progress rings and
// states empty / in progress / ready / with gaps / locked; on phone a row of round buttons opening a bottom sheet.
import type { ClientAnalysis } from '../../shared/api';
import type { Bi, Section, SectionKey, SectionState } from '../../shared/contract';
import { dataRow, dataRowCompact } from '../../shared/datarow';
import { esc } from '../../shared/dom';
import { pick, t } from '../../shared/i18n';
import { icon, type IconName } from '../../shared/icons';
import type { StrKey } from '../../shared/strings';
import { exportButtons } from './export';
import { keyReadings, shortLabel } from '../../shared/evidence';

export const ORDER: SectionKey[] = ['field', 'soil', 'pests', 'nutrition', 'rotation', 'sheet'];
const META: Record<SectionKey, { title: StrKey; short: StrKey; icon: IconName }> = {
  field: { title: 'sec_field', short: 'sec_field_short', icon: 'field' },
  soil: { title: 'sec_soil', short: 'sec_soil', icon: 'soil' },
  pests: { title: 'sec_pests', short: 'sec_pests_short', icon: 'bug' },
  nutrition: { title: 'sec_nutrition', short: 'sec_nutrition', icon: 'nutrition' },
  rotation: { title: 'sec_rotation', short: 'sec_rotation', icon: 'rotation' },
  sheet: { title: 'sec_sheet', short: 'sec_sheet_short', icon: 'sheet' },
};
const FRACTION: Record<SectionState, number> = { empty: 0, progress: 0.5, gaps: 1, ready: 1, locked: 0 };
const STATE_KEY: Record<SectionState, StrKey> = { empty: 'st_empty', progress: 'st_progress', ready: 'st_ready', gaps: 'st_gaps', locked: 'st_locked' };

export const sectionTitle = (k: SectionKey) => t(META[k].title);

export function getSection(a: ClientAnalysis, k: SectionKey): Section {
  return a.sections.find((s) => s.key === k) ?? { key: k, state: k === 'sheet' ? 'locked' : 'empty' };
}

export function ring(state: SectionState, size = 40, ic?: IconName): string {
  const r = size / 2 - 3, c = 2 * Math.PI * r, f = FRACTION[state];
  const inner = state === 'locked' ? icon('lock', 'ring__ico') : ic ? icon(ic, 'ring__ico') : '';
  return `<span class="ring ring--${state}" style="width:${size}px;height:${size}px" aria-hidden="true">
    <svg viewBox="0 0 ${size} ${size}"><circle class="ring__bg" cx="${size / 2}" cy="${size / 2}" r="${r}"/>
    ${f > 0 ? `<circle class="ring__fg" cx="${size / 2}" cy="${size / 2}" r="${r}" stroke-dasharray="${(c * f).toFixed(1)} ${c.toFixed(1)}" transform="rotate(-90 ${size / 2} ${size / 2})"/>` : ''}</svg>
    ${inner}${state === 'gaps' ? '<i class="ring__dot"></i>' : ''}${state === 'ready' ? `<i class="ring__check">${icon('check')}</i>` : ''}</span>`;
}

function stateLine(s: Section): string {
  return `<span class="sec__state">${esc(t(STATE_KEY[s.state]))}</span>${s.note ? `<span class="sec__note${s.state === 'gaps' ? ' is-warn' : ''}">${esc(pick(s.note))}</span>` : ''}`;
}

type Extra = { declared_gaps?: Bi[] };
type ItemX = { ai_observation?: boolean };

/** Body of a section (expanded card or bottom sheet). */
export function sectionBody(a: ClientAnalysis, k: SectionKey): string {
  const s = getSection(a, k);
  if (k === 'field') {
    const rows = a.evidence?.readings ?? [];
    return `${s.content ? `<p class="sec__summary">${esc(pick(s.content.summary))}</p>` : ''}
      ${rows.length ? `<div class="sec__rows">${rows.map(dataRow).join('')}</div>` : `<p class="sec__empty">${esc(t('db_no_evidence'))}</p>`}
      ${gapsList(a.evidence?.gaps.map((g) => g.text) ?? [])}
      <button type="button" class="btn btn--glass btn--sm" data-goto="dashboard">${icon('dashboard')}${esc(t('ws_open_dash'))}</button>`;
  }
  if (k === 'sheet') {
    return `<p class="sec__summary">${esc(t('ws_sheet_body'))}</p>${s.state === 'locked' ? `<p class="sec__empty">${esc(t('st_locked'))}</p>` : exportButtons(a)}`;
  }
  const c = s.content;
  if (!c) return `<p class="sec__empty">${esc(s.note ? pick(s.note) : t('ws_no_content'))}</p>`;
  const items = c.items.map((it) => `<li class="sec__item${(it as ItemX).ai_observation ? ' is-ai' : ''}">
      <h4>${esc(pick(it.title))}${(it as ItemX).ai_observation ? `<span class="ai-tag mono">${esc(t('ai_obs'))}</span>` : ''}</h4><p>${esc(pick(it.body))}</p>
      ${it.citations?.length ? `<p class="sec__cites mono">${it.citations.map((ci) => `<a href="${esc(ci.url)}" target="_blank" rel="noopener">${esc(ci.title)}</a>`).join(' · ')}</p>` : ''}</li>`).join('');
  return `<p class="sec__summary">${esc(pick(c.summary))}</p>${items ? `<ul class="sec__items">${items}</ul>` : ''}
    ${gapsList((c as unknown as Extra).declared_gaps ?? [])}
    ${k === 'rotation' ? `<button type="button" class="btn btn--glass btn--sm" data-goto="rotation">${icon('rotation')}${esc(t('ws_open_rotation'))}</button>` : ''}`;
}

function gapsList(g: Bi[]): string {
  if (!g.length) return '';
  return `<div class="sec__gaps"><h4 class="mono">${esc(t('ws_gaps_title'))}</h4><ul>${g.map((x) => `<li>${icon('info')}${esc(pick(x))}</li>`).join('')}</ul></div>`;
}

/** Desktop column. */
export function sectionsColumn(a: ClientAnalysis, open: SectionKey | null): string {
  const secs = ORDER.map((k) => getSection(a, k));
  const done = secs.filter((s) => s.state === 'ready' || s.state === 'gaps').length;
  const preview = (k: SectionKey) => {
    if (k !== 'field' || !a.evidence) return '';
    const pickR = keyReadings(a.evidence);
    return pickR.length ? `<div class="sec__peek">${pickR.map((r) => dataRowCompact(r, undefined, shortLabel(r))).join('')}</div>` : '';
  };
  return `<header class="secs__head"><h2 class="serif">${esc(t('ws_sections'))}</h2>
      <span class="secs__count mono">${esc(t('ws_progress', { done, total: secs.length }))}</span></header>
    <ul class="secs__list">${secs.map((s) => {
      const isOpen = open === s.key;
      return `<li class="sec sec--${s.state}${isOpen ? ' is-open' : ''}">
        <button type="button" class="sec__head" aria-expanded="${isOpen}" aria-controls="sec-${s.key}" data-sec="${s.key}">
          ${ring(s.state, 40, META[s.key].icon)}
          <span class="sec__titles"><span class="sec__title">${esc(t(META[s.key].title))}${s.key === 'field' ? ` <span class="sec__sub mono">· ${esc(t('sec_field_sub'))}</span>` : ''}</span>${stateLine(s)}</span>
          ${icon('down', 'sec__chev')}
        </button>
        ${!isOpen ? preview(s.key) : ''}
        <div class="sec__body" id="sec-${s.key}" ${isOpen ? '' : 'hidden'}>${isOpen ? sectionBody(a, s.key) : ''}</div>
      </li>`;
    }).join('')}</ul>`;
}

/** The section worth a glance on phone: one with gaps, else in progress, else the rotation. */
export function peekSection(a: ClientAnalysis): Section {
  const all = ORDER.filter((k) => k !== 'sheet').map((k) => getSection(a, k));
  return all.find((s) => s.state === 'gaps') ?? all.find((s) => s.state === 'progress') ?? getSection(a, 'rotation');
}

export function peekHtml(a: ClientAnalysis, k: SectionKey): string {
  const s = getSection(a, k);
  const line = s.note ? pick(s.note) : s.content ? pick(s.content.summary) : t(STATE_KEY[s.state]);
  return `${ring(s.state, 34, META[k].icon)}<span class="peek__t"><b>${esc(t(META[k].title))}</b> · ${esc(t(STATE_KEY[s.state]))}<small>${esc(line)}</small></span>${icon('chevron', 'peek__chev')}`;
}

/** Phone row of round buttons. */
export function sectionsRow(a: ClientAnalysis): string {
  return ORDER.map((k) => {
    const s = getSection(a, k);
    return `<li><button type="button" class="secbtn" data-sheet="${k}" aria-label="${esc(t(META[k].title))}: ${esc(t(STATE_KEY[s.state]))}${s.note ? ' · ' + esc(pick(s.note)) : ''}">
      ${ring(s.state, 48, META[k].icon)}<span>${esc(t(META[k].short))}</span></button></li>`;
  }).join('');
}

export function sheetHtml(a: ClientAnalysis, k: SectionKey): string {
  const s = getSection(a, k);
  return `<header class="sheet__head">${ring(s.state, 40, META[k].icon)}<div><h2 class="serif" id="sheet-t">${esc(t(META[k].title))}</h2><p>${stateLine(s)}</p></div>
    <button type="button" class="icon-btn sheet__close" data-close aria-label="${esc(t('close'))}">${icon('close')}</button></header>
    <div class="sheet__body">${sectionBody(a, k)}</div>`;
}
