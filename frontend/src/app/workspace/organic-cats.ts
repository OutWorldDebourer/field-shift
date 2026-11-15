// Organic view by categories (iteration 4, phase C): a three-line "this week" summary, then category cards, each
// with an icon, one plain sentence and a count, and a disclosure with the technical core (scientific names, how it
// acts, citation, data source): natural allies grouped by the pest they control, beneficial fungi and bacteria,
// biostimulants and micronutrients (from the crop research of phase D), practices, when to act (NASA data) and what
// harms the allies. Pre-diagnosis wording; no product names or doses.
import type { ClientAnalysis } from '../../shared/api';
import type { Bi, BioAgent, OrganicContent } from '../../shared/contract';
import { esc } from '../../shared/dom';
import { pick, t } from '../../shared/i18n';
import { icon, type IconName } from '../../shared/icons';
import { askBtn } from '../../shared/viewctx';
import type { StrKey } from '../../shared/strings';

const arr = <T>(x: T[] | undefined | null): T[] => (Array.isArray(x) ? x.filter(Boolean) : []);
/** Common names once each ("Aphid parasitoid wasp ×3"): several species often share one common name. */
const keyOf = (n: string) => n.toLowerCase().replace(/\s*\(.*\)$/, '').replace(/(es|s)$/, '').trim();
function uniqueNames(list: BioAgent[]): { name: string; n: number }[] {
  const m = new Map<string, { name: string; n: number }>();
  list.forEach((b) => { const n = pick(b.name).replace(/\s*\(.*\)$/, ''); const k = keyOf(b.name?.en ?? n); const e = m.get(k); if (e) e.n += 1; else m.set(k, { name: n, n: 1 }); });
  return [...m.values()];
}
const names = (list: BioAgent[], max = 6): string =>
  uniqueNames(list).slice(0, max).map((x) => (x.n > 1 ? t('og_species', { x: x.name, n: x.n }) : x.name)).join(' · ');
const micro = (b: BioAgent) => /fung|bacter|virus|nematod/i.test(String(b.kind ?? ''));

/** Biostimulant and micronutrient rows written by the backend from the crop research (phase D). */
export type BioStim = { kind: 'micronutrient' | 'biostimulant' | string; name: Bi; why: Bi; signal?: Bi | null; citation?: { title: string; url: string } | null; reviewed?: boolean };
type Org = Partial<OrganicContent> & { biostim?: BioStim[]; biostim_gap?: Bi };

/** Pest groups a farmer recognises; the knowledge base's "controls" text is matched in both languages. */
const GROUPS: { re: RegExp; k: StrKey }[] = [
  { re: /aphid|pulg/i, k: 'pg_aphids' }, { re: /leafminer|leaf miner|liriomyza|minador/i, k: 'pg_leafminers' },
  { re: /whitefl|bemisia|mosca blanca/i, k: 'pg_whiteflies' }, { re: /mite|ácaro|acaro|arañita|tetranych/i, k: 'pg_mites' },
  { re: /thrips|trips/i, k: 'pg_thrips' }, { re: /mealybug|cochinilla|scale|escama/i, k: 'pg_mealybugs' },
  { re: /fruit fl|ceratitis|anastrepha|mosca de la fruta/i, k: 'pg_fruitflies' },
  { re: /armyworm|caterpillar|worm|moth|looper|borer|lepidopter|gusano|oruga|polilla|spodoptera|helicoverpa/i, k: 'pg_caterpillars' },
  { re: /weevil|gorgojo|beetle larva|grub/i, k: 'pg_weevils' },
];
/** The pest group an ally controls; generalists are grouped together. */
function pestOf(b: BioAgent): string {
  const x = `${b.targets?.en ?? ''} ${b.targets?.es ?? ''}`;
  if (/generalist|generalista|soft-bodied|cuerpo blando/i.test(x)) return t('og_generalist');
  const g = GROUPS.find((y) => y.re.test(x));
  return g ? t(g.k) : (pick(b.targets).split(/[(;:,]/)[0].trim() || t('og_generalist'));
}

const cite = (c?: { title: string; url: string } | null) => (c?.url ? `<a class="og-cite mono" href="${esc(c.url)}" target="_blank" rel="noopener">${esc(c.title || t('rt_sources'))}</a>` : '');

function agentRow(b: BioAgent): string {
  return `<li><b>${esc(pick(b.name))}</b>${b.scientific ? ` <i>${esc(b.scientific)}</i>` : ''}${b.for_reported_pest ? ` <span class="og-match og-match--inline">${icon('check')}${esc(t('og_match'))}</span>` : ''}
    <span class="og-row__how">${esc(t('og_controls'))}: ${esc(pick(b.targets))}${b.note ? ` · ${esc(b.note)}` : ''}</span>${cite(b.citation)}</li>`;
}

function card(o: { id: string; ic: IconName; title: string; n: number; line: string; body: string; tag?: string; ask?: string; cls?: string }): string {
  return `<article class="og-cat glass${o.cls ? ` ${o.cls}` : ''}" aria-labelledby="${o.id}">
    <header class="og-cat__h"><span class="og-cat__ico" aria-hidden="true">${icon(o.ic)}</span><h3 class="serif" id="${o.id}">${esc(o.title)}</h3>
      ${o.n ? `<span class="og-cat__n" aria-label="${esc(t('og_count', { n: o.n }))}">${o.n}</span>` : `<span class="og-cat__gap">${esc(t('og_gap'))}</span>`}</header>
    <p class="og-cat__line">${o.line}</p>${o.tag ?? ''}
    ${o.body ? `<details class="og-cat__more"><summary>${esc(t('og_tech'))}</summary>${o.body}</details>` : ''}
    ${o.ask ? askBtn(o.ask) : ''}
  </article>`;
}

/** The crop's leading pest pressure: one plain instruction, then the figures as an instrument line. */
function readout(a: ClientAnalysis): { plain: string; src: string } {
  const ev = a.evidence;
  const p = ev?.pests.filter((x) => x.role === 'current' && x.generations_now != null && x.generations_normal != null)
    .sort((x, y) => (y.generations_now! - y.generations_normal!) - (x.generations_now! - x.generations_normal!))[0];
  if (!p) return { plain: t('og_nasa_tip'), src: '' };
  const name = typeof p.name === 'string' ? p.name : pick(p.name);
  const up = p.generations_now! > p.generations_normal!;
  const season = ev?.readings.find((x) => x.id === 'pest_gen')?.date ?? '';
  return { plain: t(up ? 'og_ti_up' : 'og_ti_ok', { p: name }), src: t('og_readout', { now: p.generations_now!.toFixed(1), norm: p.generations_normal!.toFixed(1), d: season.replace(/-/g, '‑') }) };
}

export function organicCatsHtml(a: ClientAnalysis, c: Org): string {
  const agents = arr(c.agents);
  const allies = agents.filter((b) => !micro(b));
  const microbes = agents.filter(micro);
  // Natural allies grouped by the pest they control; reported pests first.
  const groups = new Map<string, BioAgent[]>();
  for (const b of allies) { const k = pestOf(b); groups.set(k, [...(groups.get(k) ?? []), b]); }
  const byPest = [...groups.entries()].sort((x, y) => Number(y[1].some((b) => b.for_reported_pest)) - Number(x[1].some((b) => b.for_reported_pest)) || y[1].length - x[1].length);
  const timing = arr(c.timing), avoid = arr(c.avoid), practices = arr(c.practices), bio = arr(c.biostim);

  const nasa = `<span class="og-nasa" title="${esc(t('og_nasa_tip'))}">${esc(t('og_nasa_badge'))}</span>`;
  const cards = [
    ...byPest.slice(0, 4).map(([pest, list], i) => card({
      id: `og-p${i}`, ic: 'ladybug', title: t('og_for_pest_t', { p: pest }), n: list.length,
      line: esc(t('og_ally_line', { n: list.length, p: pest })), body: `<p class="og-row__names">${esc(names(list))}</p><ul class="og-rows">${list.map(agentRow).join('')}</ul>`,
      ask: t('ask_allies', { p: pest }), cls: list.some((b) => b.for_reported_pest) ? 'is-match' : '',
    })),
    card({ id: 'og-mic', ic: 'soil', title: t('og_c_micro'), n: microbes.length,
      line: microbes.length ? esc(t('og_c_micro_l', { x: microbes.slice(0, 3).map((b) => pick(b.name)).join(', ') })) : esc(t('og_none')),
      body: microbes.length ? `<ul class="og-rows">${microbes.map(agentRow).join('')}</ul>` : '', ask: microbes.length ? t('og_c_micro') : undefined }),
    card({ id: 'og-bio', ic: 'nutrition', title: t('og_c_bio'), n: bio.length,
      line: bio.length ? esc(t('og_c_bio_l', { x: bio.slice(0, 3).map((b) => pick(b.name)).join(', ') })) : esc(pick(c.biostim_gap) || t('og_c_bio_gap')),
      body: bio.length ? `<ul class="og-rows">${bio.map((b) => `<li><b>${esc(pick(b.name))}</b> <span class="og-tag mono">${esc(b.kind === 'micronutrient' ? t('og_micro') : t('og_biostim'))}</span>${b.reviewed === false ? ` <span class="og-tag mono">${esc(t('og_ai_verify'))}</span>` : ''}
        <span class="og-row__how">${esc(pick(b.why))}</span>${b.signal ? `<span class="og-row__sig">${icon('orbit')}${esc(pick(b.signal))}</span>` : ''}${cite(b.citation)}</li>`).join('')}</ul><p class="og-row__note">${esc(t('og_no_doses'))}</p>` : '',
      ask: bio.length ? t('ask_bio') : undefined }),
    card({ id: 'og-pr', ic: 'leaf', title: t('og_practices'), n: practices.length,
      line: esc(practices.slice(0, 2).map((p) => pick(p.title)).join(' · ')),
      body: `<ul class="og-rows">${practices.map((p) => `<li><b>${esc(pick(p.title))}</b><span class="og-row__how">${esc(pick(p.body))}</span>${arr(p.citations).map(cite).join(' ')}</li>`).join('')}</ul>` }),
    card({ id: 'og-ti', ic: 'orbit', title: t('og_timing'), n: timing.length, tag: nasa,
      line: timing.length ? `${esc(readout(a).plain)}<span class="og-readout mono">${esc(readout(a).src)}</span>` : esc(t('og_none')),
      body: timing.length ? `<ul class="og-rows">${timing.map((x) => `<li>${esc(pick(x))}</li>`).join('')}</ul><p class="og-row__note mono">${esc(t('og_nasa_tip'))} · ${esc(t('regional'))}</p>` : '' }),
    card({ id: 'og-av', ic: 'alert', title: t('og_avoid'), n: avoid.length,
      line: esc(avoid[0] ? pick(avoid[0]) : t('og_none')),
      body: avoid.length > 1 ? `<ul class="og-rows">${avoid.map((x) => `<li>${esc(pick(x))}</li>`).join('')}</ul>` : '' }),
  ].join('');

  // This week, in three lines: when to act, the allies for the main pest, one practice.
  const week: string[] = [];
  if (timing[0]) week.push(pick(timing[0]));
  if (byPest[0]) week.push(t('og_week_allies', { p: byPest[0][0], x: uniqueNames(byPest[0][1]).slice(0, 2).map((x) => x.name.toLowerCase()).join(t('og_and')) }));
  if (practices[0]) week.push(pick(practices[0].title).replace(/\.?$/, '.'));
  return `<div class="og og--cats">
    <section class="og-week glass" aria-labelledby="og-h">
      <h2 class="serif og-h" id="og-h">${esc(t('og_week'))}</h2>
      <ol class="og-week__l">${week.slice(0, 3).map((x) => `<li>${esc(x)}</li>`).join('')}</ol>
      <p class="notice og-notice">${icon('info')}<span>${esc(t('prediag'))}</span></p>
      ${arr(c.gaps).length ? `<p class="og-gaps">${icon('info')}<span>${esc(arr(c.gaps).map((g) => pick(g)).join(' '))}</span></p>` : ''}
    </section>
    <div class="og-cats">${cards}</div>
  </div>`;
}
