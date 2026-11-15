// Organic view (iteration 3): biological and ecological pest management from the "organic" section
// (OrganicContent): natural enemies and beneficial microorganisms grouped by kind (each with its citation and a
// soft-green badge when it matches a pest the farmer reported), practices, when to act (NASA degree-days and
// forecast), what harms the allies, and gaps. Also the small pieces reused by the section column and rotation view.
import { askBtn } from '../../shared/viewctx';
import type { ClientAnalysis } from '../../shared/api';
import type { Bi, BioAgent, OrganicContent, Section, SectionState } from '../../shared/contract';
import { esc } from '../../shared/dom';
import { pick, t } from '../../shared/i18n';
import { icon } from '../../shared/icons';
import type { StrKey } from '../../shared/strings';
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

const KIND: { k: StrKey; test: (kind: string) => boolean }[] = [
  { k: 'og_k_insect', test: (x) => /insect|mite|predator|parasitoid/.test(x) },
  { k: 'og_k_fungus', test: (x) => /fung/.test(x) },
  { k: 'og_k_bacterium', test: (x) => /bacter/.test(x) },
];
const kindOf = (b: BioAgent): StrKey => KIND.find((g) => g.test(String(b.kind ?? '').toLowerCase()))?.k ?? 'og_k_other';

const both = (b: Bi | undefined) => {
  if (!b) return '';
  const main = pick(b), other = b.en === main ? b.es : b.en;
  return other && other !== main ? `${esc(main)} <span class="og-agent__alt">/ ${esc(other)}</span>` : esc(main);
};

function agentCard(b: BioAgent): string {
  const cite = b.citation?.url ? `<a class="og-cite mono" title="${esc(b.citation.title)}" href="${esc(b.citation.url)}" target="_blank" rel="noopener">${esc(b.citation.title || t('rt_sources'))}</a>` : '';
  return `<li class="og-agent">
    ${b.for_reported_pest ? `<span class="og-match">${icon('check')}${esc(t('og_match'))}</span>` : ''}
    <h4 class="og-agent__name">${both(b.name)}</h4>
    ${b.scientific ? `<p class="og-agent__sci"><i>${esc(b.scientific)}</i></p>` : ''}
    ${pick(b.targets) ? `<p class="og-agent__t"><b>${esc(t('og_controls'))}:</b> ${esc(pick(b.targets))}</p>` : ''}
    ${b.note ? `<p class="og-agent__note">${esc(b.note)}</p>` : ''}
    ${cite}${askBtn(t('ask_agent', { x: `${pick(b.name)}${b.scientific ? ` (${b.scientific})` : ''}` }))}</li>`;
}

function agentsHtml(agents: BioAgent[]): string {
  if (!agents.length) return '';
  const order: StrKey[] = ['og_k_insect', 'og_k_fungus', 'og_k_bacterium', 'og_k_other'];
  const groups = order.map((k) => ({ k, list: agents.filter((b) => kindOf(b) === k) })).filter((g) => g.list.length);
  return `<section class="og-block" aria-labelledby="og-ag"><h3 class="serif og-block__t" id="og-ag">${esc(t('og_agents'))}</h3>
    ${groups.map((g) => `<div class="og-group"><h4 class="og-group__t mono">${esc(t(g.k))} <span>${g.list.length}</span></h4>
      <ul class="og-agents">${g.list.sort((x, y) => Number(!!y.for_reported_pest) - Number(!!x.for_reported_pest)).map(agentCard).join('')}</ul></div>`).join('')}</section>`;
}

const isInterview = (basis?: string) => !!basis && /agronom|interview|entrevista/i.test(basis);

function practicesHtml(p: OrganicContent['practices']): string {
  if (!p.length) return '';
  return `<section class="og-block" aria-labelledby="og-pr"><h3 class="serif og-block__t" id="og-pr">${esc(t('og_practices'))}</h3>
    <ul class="og-practices">${p.map((x) => `<li class="og-practice glass">
      <h4 class="serif">${esc(pick(x.title))}</h4>${isInterview(x.basis) ? `<span class="og-tag mono">${esc(t('og_interview'))}</span>` : ''}
      <p>${esc(pick(x.body))}</p>
      ${arr(x.citations).length ? `<p class="og-cites mono">${arr(x.citations).map((c) => `<a href="${esc(c.url)}" target="_blank" rel="noopener">${esc(c.title)}</a>`).join(' · ')}</p>` : ''}</li>`).join('')}</ul></section>`;
}

const listBlock = (id: string, title: string, items: Bi[], ic: 'info' | 'alert' | 'orbit', tag = '') => items.length
  ? `<section class="og-side__b" aria-labelledby="${id}"><h3 class="og-side__t mono" id="${id}">${esc(title)}${tag}</h3>
      <ul class="og-list">${items.map((x) => `<li>${icon(ic)}<span>${esc(pick(x))}</span></li>`).join('')}</ul></section>`
  : '';

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
  const nasaTag = `<span class="og-nasa" title="${esc(t('og_nasa_tip'))}">${esc(t('og_nasa'))}</span>`;
  return `<div class="og">
    <aside class="og-side glass" aria-labelledby="og-h">
      <p class="rot__label mono">${esc(t('og_label'))}</p>
      <h2 class="serif og-h" id="og-h">${esc(pick(c.title) || t('og_title'))}</h2>
      ${pick(c.summary) ? `<p class="og-sum">${esc(pick(c.summary))}</p>` : ''}
      <p class="notice og-notice">${icon('info')}<span>${esc(t('prediag'))}</span></p>
      ${listBlock('og-ti', t('og_timing'), arr(c.timing), 'orbit', nasaTag)}
      ${listBlock('og-av', t('og_avoid'), arr(c.avoid), 'alert')}
      ${listBlock('og-ga', t('ws_gaps_title'), arr(c.gaps), 'info')}
    </aside>
    <div class="og-main">${agentsHtml(arr(c.agents))}${practicesHtml(arr(c.practices))}</div>
  </div>`;
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
