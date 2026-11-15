// Rotation view (App 5): the pre-diagnosis card (headline, confidence, soil/pest/nutrition summaries) and three
// strategy columns with a 12-month plan strip and 1–5 dot scores. Perennial crops (or content without strategies)
// get practice cards instead: inter-row cover, intercropping, residue, biological control — driven by the data.
import { evaluateFormHtml } from './evaluate';
import { askBtn } from '../../shared/viewctx';
import type { ClientAnalysis } from '../../shared/api';
import type { SectionContent } from '../../shared/contract';
import { dots, learnCropNames, timelineStrip } from '../../shared/charts';
import { derivedHeadline } from '../../shared/evidence';
import { provTag } from '../../shared/datarow';
import { esc } from '../../shared/dom';
import { pick, t } from '../../shared/i18n';
import { icon, type IconName } from '../../shared/icons';
import type { StrKey } from '../../shared/strings';
import { exportButtons } from './export';
import { getSection, ring } from './sections';
import { organicLine, organicState } from './organic';

type Strategy = NonNullable<SectionContent['strategies']>[number];
type Proposal = NonNullable<SectionContent['proposal']>;
const cap = (x: string) => x.charAt(0).toUpperCase() + x.slice(1);
/** Evidence as one line per item (the joined text read as a run-on block). */
const evList = (s: Strategy): string => {
  const list = ((s as unknown as { evidence_list?: { en: string; es: string }[] }).evidence_list ?? []).map((e) => pick(e)).filter(Boolean);
  return list.length ? `<ul class="strat__ev">${list.map((e) => `<li>${esc(cap(e))}</li>`).join('')}</ul>` : esc(pick(s.evidence));
};
/** Dates keep their hyphens on one line. */
const nb = (x: string) => x.replace(/-/g, '‑');
/** "Water use" of a proposal is scored against this reading: say which, with its date and scale. */
const waterLine = (a: ClientAnalysis): string => {
  const r = a.evidence?.readings.find((x) => x.id === 'water_balance');
  return r ? t('ev_water_src', { v: Math.round(r.value ?? 0), d: nb(r.date) }) : '';
};
const SCORES: { k: keyof Strategy['scores']; label: StrKey }[] = [
  { k: 'soil', label: 'sc_soil' }, { k: 'water', label: 'sc_water' }, { k: 'pest_break', label: 'sc_pest_break' }, { k: 'income', label: 'sc_income' }, { k: 'risk', label: 'sc_risk' },
];

type Cite = { title: string; url: string };
const cites = (s: Strategy): Cite[] => ((s as unknown as { citations?: Cite[] }).citations ?? []);
const uniq = (c: Cite[]) => c.filter((x, i) => c.findIndex((y) => y.url === x.url) === i);

function strategyCol(s: Strategy, startMonth: number, idx: number, proposal: Proposal | null = null, waterSrc = ''): string {
  const tag = proposal ? (proposal.crop?.crop_type === 'perennial' ? `<span class="strat__tag strat__tag--prop">${esc(t('ev_perennial'))}</span>` : '')
    : s.current ? `<span class="strat__tag">${esc(t('rt_current'))}</span>` : s.best_for ? `<span class="strat__tag strat__tag--best">${esc(pick(s.best_for))}</span>` : '';
  return `<article class="strat glass${s.best_for ? ' is-best' : ''}${proposal ? ' is-proposal' : ''}" aria-labelledby="strat-${idx}">
    <header class="strat__head">${tag}<h3 class="serif" id="strat-${idx}">${esc(pick(s.name).replace(/\s*\((current plan|plan actual)\)\s*$/i, ''))}</h3>${proposal && waterSrc ? `<p class="strat__src mono">${esc(waterSrc)}</p>` : ''}</header>
    ${proposal?.crop?.crop_type === 'perennial'
      ? `<p class="strat__tl mono">${esc(t('ev_plantation'))}</p><p class="strat__plant"><span class="strat__plant-now">${esc(cap(pick(proposal.crop.name)))}</span><span class="strat__plant-bar" aria-hidden="true"></span><span class="strat__plant-t">${esc(t('ev_plantation_t'))}</span></p>`
      : `<p class="strat__tl mono">${esc(t(s.timeline.length === 12 ? 'rt_timeline' : 'rt_seasons'))}</p>
    ${timelineStrip(s.timeline, startMonth, true, true)}`}
    <dl class="strat__scores">${SCORES.map((sc) => `<div><dt>${esc(t(sc.label))}</dt><dd>${dots(s.scores[sc.k], t(sc.label))}</dd></div>`).join('')}</dl>

    <details class="strat__more"><summary>${esc(t('rt_more'))}</summary>
      <dl><dt>${esc(t('rt_why'))}</dt><dd>${esc(cap(pick(s.why)))}</dd><dt>${esc(t('rt_evidence'))}</dt><dd>${evList(s)}</dd>
      <dt>${esc(t('rt_gaps'))}</dt><dd>${esc(pick(s.gaps))}</dd><dt>${esc(t('rt_risks'))}</dt><dd>${esc(pick(s.risks))}</dd>
      ${cites(s).length ? `<dt>${esc(t('rt_sources'))}</dt><dd class="strat__cites">${uniq(cites(s)).map((c) => `<a href="${esc(c.url)}" target="_blank" rel="noopener">${esc(c.title)}</a>`).join('')}</dd>` : ''}</dl>
    </details>
    ${askBtn(proposal ? t('ask_proposal', { x: pick(proposal.crop?.name ?? s.name) }) : t('ask_strategy', { n: idx + 1, x: pick(s.name).replace(/\s*\((current plan|plan actual)\)\s*$/i, '') }))}
  </article>`;
}

export function rotationHtml(a: ClientAnalysis): string {
  learnCropNames([a.crop_current, ...(a.crop_history ?? [])]);
  const rot = getSection(a, 'rotation');
  const c = rot.content;
  const perennial = a.crop_current?.crop_type === 'perennial' || (!!c && !c.strategies?.length && c.items.length > 0);
  const start = a.evidence ? new Date(a.evidence.generated_at).getUTCMonth() + 1 : new Date().getMonth() + 1;
  const pd = a.prediagnosis ?? (derivedHeadline(a.evidence) ? { headline: derivedHeadline(a.evidence)!, confidence: null } : null);
  const conf = pd?.confidence ?? null;
  const confN = conf === 'high' ? 3 : conf === 'medium' ? 2 : conf === 'low' ? 1 : 0;
  const mini = (k: 'soil' | 'pests' | 'nutrition', ic: IconName, title: StrKey) => {
    const s = getSection(a, k);
    const titles = s.content?.items.slice(0, 3).map((it) => pick(it.title)).filter(Boolean) ?? [];
    const line = titles.length ? titles.join(' · ') : s.content ? pick(s.content.summary) : s.note ? pick(s.note) : t('ws_no_content');
    return `<li class="rot__mini">${ring(s.state, 34, ic)}<div><h4>${esc(t(title))}</h4><p>${esc(line)}</p></div></li>`;
  };
  const right = !c
    ? `<p class="rot__empty glass">${esc(t('rt_empty'))}</p>`
    : perennial
      ? `<p class="rot__lede">${esc(pick(c.summary) || t('rt_perennial'))}</p><p class="rot__lede rot__lede--q">${esc(t('rt_perennial'))}</p>
         <ul class="practices">${c.items.map((it) => `<li class="glass"><h3 class="serif">${esc(pick(it.title))}</h3><p>${esc(pick(it.body))}</p></li>`).join('')}</ul>`
      : `<div class="strats${c.proposal ? ' strats--4' : ''}">${(c.strategies ?? []).map((s, i) => strategyCol(s, start % 12, i)).join('')}${c.proposal ? strategyCol(c.proposal as unknown as Strategy, start % 12, 3, c.proposal, waterLine(a)) : ''}</div>`;
  // A perennial current crop shows its practices; a proposal still gets its own scored card below them.
  const extra = c && perennial && c.proposal ? `<div class="strats strats--1">${strategyCol(c.proposal as unknown as Strategy, start % 12, 3, c.proposal, waterLine(a))}</div>` : '';
  return `<div class="rot">
    <section class="rot__dx glass" aria-labelledby="rot-h">
      <p class="rot__label mono">${esc(t('rt_label'))}</p>
      <h2 class="serif rot__h" id="rot-h">${esc(pd ? pick(pd.headline) : c ? pick(c.summary) : t('rt_empty'))}</h2>
      ${a.evidence?.readings.some((r) => r.provisional) && /warm|cálid|temperat/i.test(pd ? pick(pd.headline) : '') ? `<p class="rot__prov">${provTag(a.evidence!.readings.find((r) => r.provisional)!)} <span>${esc(pick(a.evidence!.readings.find((r) => r.provisional)!.note ?? { en: t('prov_tip'), es: t('prov_tip') }))}</span></p>` : ''}
      ${conf ? `<div class="conf"><span class="mono">${esc(t('rt_confidence'))}</span><span class="conf__bar" aria-hidden="true">${[1, 2, 3].map((i) => `<i class="${i <= confN ? 'on' : ''}"></i>`).join('')}</span><b>${esc(t(conf === 'high' ? 'conf_high' : conf === 'medium' ? 'conf_medium' : 'conf_low'))}</b></div>` : ''}
      <p class="rot__confirm">${icon('info')}${esc(t('rt_confirm'))}</p>
      <ul class="rot__minis">${mini('soil', 'soil', 'sec_soil')}${mini('pests', 'bug', 'sec_pests')}${mini('nutrition', 'nutrition', 'sec_nutrition')}
        <li class="rot__mini">${ring(organicState(a), 34, 'ladybug')}<div><h4><button type="button" class="rot__minilink" data-goto="organic">${esc(t('ap_tab_org'))}</button></h4><p>${esc(organicLine(a))}</p></div></li></ul>
    </section>
    <section class="rot__cmp" aria-labelledby="rot-c">
      <header class="rot__cmphead"><h2 class="serif" id="rot-c">${esc(perennial && c ? t('sec_rotation') : c?.proposal ? t('rt_compare_prop') : t('rt_compare'))}</h2>
        ${!perennial && c ? `<p class="mono rot__legend"><span class="rot__swatch" aria-hidden="true"></span>${esc(t('rt_plan_legend2'))} · ${esc(t('rt_dots_legend'))}</p>` : ''}</header>
      ${evaluateFormHtml(a, c?.proposal ? pick(c.proposal.crop?.name ?? c.proposal.name) : '')}
      ${right}${extra}
    </section>
    <footer class="rot__foot">${exportButtons(a, true)}<p class="mono">${esc(t('ex_cites'))}</p></footer>
  </div>`;
}
