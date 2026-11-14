// Rotation view (App 5): the pre-diagnosis card (headline, confidence, soil/pest/nutrition summaries) and three
// strategy columns with a 12-month plan strip and 1–5 dot scores. Perennial crops (or content without strategies)
// get practice cards instead: inter-row cover, intercropping, residue, biological control — driven by the data.
import type { ClientAnalysis } from '../../shared/api';
import type { SectionContent } from '../../shared/contract';
import { calendarStrip, dots } from '../../shared/charts';
import { esc } from '../../shared/dom';
import { pick, t } from '../../shared/i18n';
import { icon, type IconName } from '../../shared/icons';
import type { StrKey } from '../../shared/strings';
import { exportButtons } from './export';
import { getSection, ring } from './sections';

type Strategy = NonNullable<SectionContent['strategies']>[number];
const SCORES: { k: keyof Strategy['scores']; label: StrKey }[] = [
  { k: 'soil', label: 'sc_soil' }, { k: 'water', label: 'sc_water' }, { k: 'pest_break', label: 'sc_pest_break' }, { k: 'income', label: 'sc_income' }, { k: 'risk', label: 'sc_risk' },
];

function strategyCol(s: Strategy, startMonth: number, idx: number): string {
  const tag = s.current ? `<span class="strat__tag">${esc(t('rt_current'))}</span>` : s.best_for ? `<span class="strat__tag strat__tag--best">${esc(pick(s.best_for))}</span>` : '';
  return `<article class="strat glass${s.best_for ? ' is-best' : ''}" aria-labelledby="strat-${idx}">
    <header class="strat__head">${tag}<h3 class="serif" id="strat-${idx}">${s.current ? esc(t('rt_current')) + ' · ' : ''}${esc(pick(s.name))}</h3></header>
    <p class="strat__tl mono">${esc(t('rt_timeline'))}</p>
    ${calendarStrip(s.timeline, startMonth, true, true)}
    <dl class="strat__scores">${SCORES.map((sc) => `<div><dt>${esc(t(sc.label))}${sc.k === 'risk' ? `<small>${esc(t('sc_risk_note'))}</small>` : ''}</dt><dd>${dots(s.scores[sc.k], t(sc.label))}</dd></div>`).join('')}</dl>
    <details class="strat__more"><summary>${esc(t('rt_more'))}</summary>
      <dl><dt>${esc(t('rt_why'))}</dt><dd>${esc(pick(s.why))}</dd><dt>${esc(t('rt_evidence'))}</dt><dd>${esc(pick(s.evidence))}</dd>
      <dt>${esc(t('rt_gaps'))}</dt><dd>${esc(pick(s.gaps))}</dd><dt>${esc(t('rt_risks'))}</dt><dd>${esc(pick(s.risks))}</dd></dl>
    </details>
  </article>`;
}

export function rotationHtml(a: ClientAnalysis): string {
  const rot = getSection(a, 'rotation');
  const c = rot.content;
  const perennial = a.crop_current?.crop_type === 'perennial' || (!!c && !c.strategies?.length && c.items.length > 0);
  const start = a.evidence ? new Date(a.evidence.generated_at).getUTCMonth() + 1 : new Date().getMonth() + 1;
  const pd = a.prediagnosis;
  const conf = pd?.confidence ?? null;
  const confN = conf === 'high' ? 3 : conf === 'medium' ? 2 : conf === 'low' ? 1 : 0;
  const mini = (k: 'soil' | 'pests' | 'nutrition', ic: IconName, title: StrKey) => {
    const s = getSection(a, k);
    return `<li class="rot__mini">${ring(s.state, 34, ic)}<div><h4>${esc(t(title))}</h4><p>${esc(s.content ? pick(s.content.summary) : s.note ? pick(s.note) : t('ws_no_content'))}</p></div></li>`;
  };
  const right = !c
    ? `<p class="rot__empty glass">${esc(t('rt_empty'))}</p>`
    : perennial
      ? `<p class="rot__lede">${esc(pick(c.summary) || t('rt_perennial'))}</p><p class="rot__lede rot__lede--q">${esc(t('rt_perennial'))}</p>
         <ul class="practices">${c.items.map((it) => `<li class="glass"><h3 class="serif">${esc(pick(it.title))}</h3><p>${esc(pick(it.body))}</p></li>`).join('')}</ul>`
      : `<div class="strats">${(c.strategies ?? []).map((s, i) => strategyCol(s, start % 12, i)).join('')}</div>`;
  return `<div class="rot">
    <section class="rot__dx glass" aria-labelledby="rot-h">
      <p class="rot__label mono">${esc(t('rt_label'))}</p>
      <h2 class="serif rot__h" id="rot-h">${esc(pd ? pick(pd.headline) : c ? pick(c.summary) : t('rt_empty'))}</h2>
      ${conf ? `<div class="conf"><span class="mono">${esc(t('rt_confidence'))}</span><span class="conf__bar" aria-hidden="true">${[1, 2, 3].map((i) => `<i class="${i <= confN ? 'on' : ''}"></i>`).join('')}</span><b>${esc(t(conf === 'high' ? 'conf_high' : conf === 'medium' ? 'conf_medium' : 'conf_low'))}</b></div>` : ''}
      <p class="rot__confirm">${icon('info')}${esc(t('rt_confirm'))}</p>
      <ul class="rot__minis">${mini('soil', 'soil', 'sec_soil')}${mini('pests', 'bug', 'sec_pests')}${mini('nutrition', 'nutrition', 'sec_nutrition')}</ul>
    </section>
    <section class="rot__cmp" aria-labelledby="rot-c">
      <header class="rot__cmphead"><h2 class="serif" id="rot-c">${esc(perennial && c ? t('sec_rotation') : t('rt_compare'))}</h2>
        ${!perennial && c ? `<p class="mono rot__legend"><i></i>${esc(t('rt_plan_legend'))}</p>` : ''}</header>
      ${right}
    </section>
    <footer class="rot__foot">${exportButtons(a, true)}<p class="mono">${esc(t('ex_cites'))}</p></footer>
  </div>`;
}
