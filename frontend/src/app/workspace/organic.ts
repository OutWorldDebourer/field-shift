// Organic view: the "organic" section (OrganicContent) rendered by category (organic-cats.ts, iteration 4), its empty
// state, and the small pieces reused by the section column and rotation view.
import { organicCatsHtml } from './organic-cats';
import './organic-cats.css';
import type { ClientAnalysis } from '../../shared/api';
import type { OrganicContent, Section, SectionState } from '../../shared/contract';
import { esc } from '../../shared/dom';
import { pick, t } from '../../shared/i18n';
import { icon } from '../../shared/icons';
import { getSection } from './progress';
import './organic.css';

type OrgSection = Omit<Section, 'key' | 'content'> & { key: 'organic'; content?: Partial<OrganicContent> };
const arr = <T>(x: T[] | undefined | null): T[] => (Array.isArray(x) ? x.filter(Boolean) : []);

/** The organic section, or an empty placeholder when the backend has not sent it yet. */
export function organicSection(a: ClientAnalysis): OrgSection {
  const s = a.sections.find((x) => (x.key as string) === 'organic') as unknown as OrgSection | undefined;
  return s ?? { key: 'organic', state: 'empty' };
}

/** Has content worth a page: at least a summary, an agent or a practice. */
export function organicContent(a: ClientAnalysis): Partial<OrganicContent> | null {
  const c = organicSection(a).content;
  if (!c) return null;
  return pick(c.summary) || arr(c.agents).length || arr(c.practices).length ? c : null;
}

/** State shown on the section card: ready once there is content, in progress once the Pest care topic has started. */
export function organicState(a: ClientAnalysis): SectionState {
  const s = organicSection(a);
  if (organicContent(a)) return s.state === 'gaps' ? 'gaps' : 'ready';
  if (s.state === 'progress' || getSection(a, 'pests').state !== 'empty') return 'progress';
  return 'empty';
}

export function organicHtml(a: ClientAnalysis): string {
  const c = organicContent(a);
  if (!c) {
    return `<div class="og og--empty"><section class="og-empty glass" aria-labelledby="og-e">
      <span class="og-empty__ico" aria-hidden="true">${icon('ladybug')}</span>
      <p class="rot__label mono">${esc(t('og_label'))}</p>
      <h2 class="serif" id="og-e">${esc(t('og_empty_t'))}</h2><p>${esc(t('og_empty_b'))}</p>
      <div class="og-empty__a"><button type="button" class="btn btn--solid btn--sm" data-goto="chat">${icon('chat')}${esc(t('og_go_chat'))}</button>
        ${a.readOnly ? '' : `<button type="button" class="btn btn--glass btn--sm" data-intake aria-haspopup="dialog">${icon('clipboard')}${esc(t('in_btn'))}</button>`}</div>
      <p class="notice og-notice">${icon('info')}<span>${esc(t('prediag'))}</span></p></section></div>`;
  }
  // Iteration 4: categories with a "this week" summary (organic-cats.ts).
  return organicCatsHtml(a, c);
}

/** Body of the Organic card in the section column / phone sheet. */
export function organicBody(a: ClientAnalysis): string {
  const c = organicContent(a);
  const line = c ? pick(c.summary) : t('og_empty_t');
  const top = arr(c?.agents).filter((b) => b.for_reported_pest).slice(0, 3).map((b) => pick(b.name)).filter(Boolean);
  return `<p class="sec__summary">${esc(line)}</p>
    ${top.length ? `<p class="sec__empty"><span class="og-match og-match--inline">${icon('check')}${esc(t('og_match'))}</span> ${esc(top.join(' · '))}</p>` : ''}
    <button type="button" class="btn btn--glass btn--sm" data-goto="organic">${icon('ladybug')}${esc(t('og_open'))}</button>`;
}

/** One-line summary for the rotation view's summary rows. */
export function organicLine(a: ClientAnalysis): string {
  const c = organicContent(a);
  if (!c) return t('og_empty_t');
  const names = arr(c.agents).slice(0, 3).map((b) => pick(b.name)).filter(Boolean);
  return names.length ? names.join(' · ') : pick(c.summary);
}
