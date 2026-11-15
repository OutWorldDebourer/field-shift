// Progress rail at the top of the chat (iteration 4: compact). A thin bar with six dots and the active topic's name;
// a click expands the six topic chips and the "Next topic: … — I need: …" line. Also the chime toggle and the
// closing card once every topic is done.
import type { ClientAnalysis } from '../../shared/api';
import type { SectionKey, Step } from '../../shared/contract';
import { esc } from '../../shared/dom';
import { pick, t } from '../../shared/i18n';
import { icon, type IconName } from '../../shared/icons';
import { chimeOn } from './chime';
import { allDone, SHORT, TITLE } from './progress';

export const SEC_ICON: Record<SectionKey, IconName> = { field: 'field', soil: 'soil', pests: 'bug', nutrition: 'nutrition', rotation: 'rotation', sheet: 'sheet' };

export function railHtml(steps: Step[], just: SectionKey[] = [], expanded = false): string {
  const active = steps.find((s) => s.state === 'active');
  const name = (s: Step) => pick(s.title) || t(TITLE[s.key]);
  const chips = steps.map((s) => {
    const sr = t(s.state === 'done' ? 'rail_done_sr' : s.state === 'active' ? 'rail_active_sr' : 'rail_pending_sr', { t: name(s) });
    const cls = `rail__chip is-${s.state}${just.includes(s.key) ? ' is-just' : ''}${just.length && s.state === 'active' ? ' is-arriving' : ''}`;
    return `<li><button type="button" class="${cls}" data-railsec="${s.key}" aria-label="${esc(sr)}"${s.state === 'active' ? ' aria-current="step"' : ''}>
      <span class="rail__mark" aria-hidden="true">${s.state === 'done' ? icon('check') : icon(SEC_ICON[s.key])}</span><span class="rail__t" aria-hidden="true">${esc(t(SHORT[s.key]))}</span></button></li>`;
  }).join('');
  const needs = active?.needs?.map((n) => pick(n)).filter(Boolean) ?? [];
  const line = !active
    ? `<span class="rail__all">${icon('check')}${esc(t('rail_all'))}</span>`
    : `${esc(t('rail_next', { t: '' }))}<b>${esc(name(active))}</b>${needs.length ? ` <span class="rail__needs">— ${esc(t('rail_needs', { x: needs.join(', ') }))}</span>` : ''}`;
  const on = chimeOn();
  const done = steps.filter((s) => s.state === 'done').length;
  const dots = steps.map((s) => `<i class="rail__dot is-${s.state}${just.includes(s.key) ? ' is-just' : ''}"></i>`).join('');
  const now = active ? `<span class="rail__count mono">${esc(t('rail_count', { n: done, m: steps.length }))}</span><b>${esc(name(active))}</b>`
    : `<b>${esc(t('rail_all'))}</b>`;
  return `<div class="rail__bar"><button type="button" class="rail__toggle" data-rail-toggle aria-expanded="${expanded}" aria-controls="rail-more">
        <span class="rail__dots" aria-hidden="true">${dots}</span><span class="rail__now">${now}</span>${icon('down')}</button>
      <button type="button" class="rail__snd" data-chime aria-pressed="${on}" aria-label="${esc(t(on ? 'chime_on' : 'chime_off'))}" title="${esc(t(on ? 'chime_on' : 'chime_off'))}">${icon(on ? 'sound' : 'mute')}</button></div>
    <div class="rail__more" id="rail-more"${expanded ? '' : ' hidden'}><ol class="rail__chips" aria-label="${esc(t('rail_label'))}">${chips}</ol>
      <p class="rail__next" aria-live="polite">${line}</p></div>`;
}

const DONE_KEY = (id: string) => `fs-done-closed:${id}`;

/** Remembers (per analysis, in this browser) that the farmer closed the closing card. */
export function closeDoneCard(id: string): void {
  try { localStorage.setItem(DONE_KEY(id), '1'); } catch { /* storage blocked: closed for this view only */ }
}

function doneClosed(id: string): boolean {
  try { return localStorage.getItem(DONE_KEY(id)) === '1'; } catch { return false; }
}

/** The closing card shown in the chat when every topic is done (until the farmer closes it). */
export function doneCardHtml(a: ClientAnalysis, steps: Step[]): string {
  if (!allDone(steps) || a.readOnly || doneClosed(a.id)) return '';
  return `<li class="msg chat__done"><div class="done glass">
    <button type="button" class="done__x" data-done-close aria-label="${esc(t('close'))}" title="${esc(t('close'))}">${icon('close')}</button>
    <span class="done__mark" aria-hidden="true">${icon('check')}</span>
    <h3 class="serif">${esc(t('done_t'))}</h3><p>${esc(t('done_b'))}</p>
    <div class="done__a"><button type="button" class="btn btn--solid btn--sm" data-export="sheet">${icon('sheet')}${esc(t('done_sheet'))}</button>
      <button type="button" class="btn btn--glass btn--sm" data-goto="dashboard">${icon('dashboard')}${esc(t('done_dash'))}</button></div></div></li>`;
}

/** Screen-reader / toast line for a step that just flipped to done. */
export function flipLine(steps: Step[], key: SectionKey): string {
  const s = steps.find((x) => x.key === key);
  const nx = steps.find((x) => x.state === 'active');
  const name = (x: Step) => pick(x.title) || t(TITLE[x.key]);
  return s ? (nx ? t('rail_flip', { t: name(s), n: name(nx) }) : t('rail_flip_last', { t: name(s) })) : '';
}
