// Progress rail at the top of the chat: six topic chips (done = check, active = highlighted, pending = dim),
// a "Next topic: … — I need: …" line, the chime toggle, and the closing card once every topic is done.
import type { ClientAnalysis } from '../../shared/api';
import type { SectionKey, Step } from '../../shared/contract';
import { esc } from '../../shared/dom';
import { pick, t } from '../../shared/i18n';
import { icon, type IconName } from '../../shared/icons';
import { chimeOn } from './chime';
import { allDone, SHORT, TITLE } from './progress';

export const SEC_ICON: Record<SectionKey, IconName> = { field: 'field', soil: 'soil', pests: 'bug', nutrition: 'nutrition', rotation: 'rotation', sheet: 'sheet' };

export function railHtml(steps: Step[], just: SectionKey[] = []): string {
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
  return `<div class="rail__row"><ol class="rail__chips" aria-label="${esc(t('rail_label'))}">${chips}</ol>
      <button type="button" class="rail__snd" data-chime aria-pressed="${on}" aria-label="${esc(t(on ? 'chime_on' : 'chime_off'))}" title="${esc(t(on ? 'chime_on' : 'chime_off'))}">${icon(on ? 'sound' : 'mute')}</button></div>
    <p class="rail__next" aria-live="polite">${line}</p>`;
}

/** The closing card shown in the chat when every topic is done. */
export function doneCardHtml(a: ClientAnalysis, steps: Step[]): string {
  if (!allDone(steps) || a.readOnly) return '';
  return `<li class="msg chat__done"><div class="done glass">
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
