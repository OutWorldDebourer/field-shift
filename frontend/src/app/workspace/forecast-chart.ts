// "This season vs 40-year normal" with what comes next: past 12 months (amber = this year, pale band = normal),
// then the NASA GEOS-FP forecast at daily resolution (white dotted), the season outlook per month (two thin white
// lines: "if it stays this warm" dashed, "back to normal" solid, over the normal band) and the 2031–2050 projection
// as a bracket (median and model range) at the far right. Pure markup.
import type { EvidencePackage, Forecast, Outlook } from '../../shared/contract';
import { hitsHtml, type Hit } from '../../shared/charthover';
import { esc } from '../../shared/dom';
import { fmt, monthShort, t } from '../../shared/i18n';

type Climate = EvidencePackage['climate_monthly'];
export type Proj = NonNullable<EvidencePackage['projection']> & { d_temp_range?: [number, number]; baseline_period?: string; label?: string };

export interface SeasonPlus { rows: Climate; forecast?: Forecast | null; outlook?: Outlook | null; projection?: Proj | null; provisional?: boolean; w: number; h: number }

const path = (pts: ([number, number] | null)[]) => {
  let d = '', pen = false;
  pts.forEach((p) => { if (!p) { pen = false; return; } d += `${pen ? 'L' : 'M'}${p[0].toFixed(1)} ${p[1].toFixed(1)}`; pen = true; });
  return d;
};

export function seasonPlusChart(o: SeasonPlus): string {
  const { rows } = o;
  if (rows.filter((r) => r.normal_lo != null).length < 2) return `<p class="chart-empty">${esc(t('db_no_evidence'))}</p>`;
  const fc = o.forecast?.days?.length ? o.forecast.days : null;
  const ol = o.outlook?.months?.length ? o.outlook.months : null;
  const pj = o.projection && o.projection.d_temp_c != null ? o.projection : null;
  const H = o.h, L = 40, R = 12, T = 26, B = 30, GAP = 14;
  const wFc = fc ? 120 : 0, wOl = ol ? Math.max(70, ol.length * 44) : 0, wPj = pj ? 64 : 0;
  const extra = (fc ? wFc + GAP : 0) + (ol ? wOl + GAP : 0) + (pj ? wPj + GAP : 0);
  const W = Math.max(o.w, 420 + extra);
  const pastEnd = W - R - extra;
  const base = rows.reduce((s, r) => s + r.normal_mean, 0) / rows.length;
  const projVals = pj ? [base + pj.d_temp_c!, ...(pj.d_temp_range ?? []).map((v) => base + v)] : [];
  const vals = [
    ...rows.flatMap((r) => [r.normal_lo, r.normal_hi, r.t_mean ?? r.normal_mean]),
    ...(fc ?? []).map((d) => d.tmean),
    ...(ol ?? []).flatMap((m) => [m.normal_lo, m.normal_hi, m.persist_mean]),
    ...projVals,
  ].filter((v) => Number.isFinite(v));
  const lo = Math.floor(Math.min(...vals) - 0.5), hi = Math.ceil(Math.max(...vals) + 0.5);
  const y = (v: number) => T + (1 - (v - lo) / (hi - lo || 1)) * (H - T - B);
  const x = (i: number) => L + (i / (rows.length - 1)) * (pastEnd - L);

  // past 12 months
  const band = rows.map((r, i) => `${i ? 'L' : 'M'}${x(i).toFixed(1)} ${y(r.normal_hi).toFixed(1)}`).join('') +
    rows.slice().reverse().map((r, j) => `L${x(rows.length - 1 - j).toFixed(1)} ${y(r.normal_lo).toFixed(1)}`).join('') + 'Z';
  const mean = path(rows.map((r, i) => [x(i), y(r.normal_mean)]));
  const pts = rows.map((r, i) => (r.t_mean == null ? null : [x(i), y(r.t_mean)] as [number, number]));
  const last = [...pts].reverse().find(Boolean) ?? null;
  const lastRow = [...rows].reverse().find((r) => r.t_mean != null);
  const ticks: string[] = [];
  for (let v = lo; v <= hi; v += Math.max(1, Math.round((hi - lo) / 4))) ticks.push(`<g class="tick"><line x1="${L}" x2="${W - R}" y1="${y(v)}" y2="${y(v)}"/><text x="${L - 8}" y="${y(v) + 4}" text-anchor="end">${v}°</text></g>`);
  const months = rows.map((r, i) => {
    const m = +r.month.slice(5, 7) - 1;
    return i % (rows.length > 8 ? 2 : 1) === 0 || i === rows.length - 1 ? `<text x="${x(i)}" y="${H - 8}" text-anchor="middle">${esc(monthShort(m))}</text>` : '';
  }).join('');

  // hover/keyboard read-out: one point per date with its values (charthover.ts)
  const hits: Hit[] = [];
  const mLabel = (ym: string) => `${monthShort(+ym.slice(5, 7) - 1)} ${ym.slice(0, 4)}`;
  const sgn = (v: number) => `${v > 0 ? '+' : ''}${fmt(v, 1)}`;
  rows.forEach((r, i) => {
    const v: string[] = [];
    if (r.t_mean != null) v.push(t('hv_this', { v: fmt(r.t_mean, 1) }) + (r === lastRow && o.provisional ? ` · ${t('prov')}` : ''));
    v.push(t('hv_normal', { v: fmt(r.normal_mean, 1), lo: fmt(r.normal_lo, 1), hi: fmt(r.normal_hi, 1) }));
    hits.push({ x: x(i), y: y(r.t_mean ?? r.normal_mean), k: mLabel(r.month), v });
  });

  let cur = pastEnd, zones = '', marks = '';
  const zoneHead = (x0: number, w: number, label: string) => `<line class="zone__sep" x1="${x0 - GAP / 2}" x2="${x0 - GAP / 2}" y1="${T - 14}" y2="${H - B}"/><text class="zone__t" x="${x0 + w / 2}" y="${T - 12}" text-anchor="middle">${esc(label)}</text>`;

  // forecast: daily tmean, white dotted, in its own zone just after the last month
  if (fc) {
    const x0 = cur + GAP, n = fc.length;
    const fx = (i: number) => x0 + (n === 1 ? wFc / 2 : (i / (n - 1)) * wFc);
    zones += zoneHead(x0, wFc, t('fc_days_long', { n }));
    const link = last ? `<path class="fc__link" d="M${last[0].toFixed(1)} ${last[1].toFixed(1)}L${fx(0).toFixed(1)} ${y(fc[0].tmean).toFixed(1)}"/>` : '';
    marks += `${link}<path class="fc__line" d="${path(fc.map((d, i) => [fx(i), y(d.tmean)]))}"/>` +
      fc.map((d, i) => `<circle class="fc__dot" cx="${fx(i).toFixed(1)}" cy="${y(d.tmean).toFixed(1)}" r="2.4"/>`).join('');
    fc.forEach((d, i) => hits.push({ x: fx(i), y: y(d.tmean), k: `${d2(d.date)} ${d.date.slice(0, 4)} · ${t('fc_label')}`,
      v: [t('hv_fc', { m: fmt(d.tmean, 1), x: fmt(d.tmax, 1), n: fmt(d.tmin, 1) }), t('hv_rain', { r: fmt(d.rain_mm, 1) })] }));
    cur = x0 + wFc;
  }
  // outlook: per month, normal band continues; two thin white lines
  if (ol) {
    const x0 = cur + GAP, n = ol.length;
    const ox = (i: number) => x0 + (n === 1 ? wOl / 2 : (i / (n - 1)) * wOl);
    zones += zoneHead(x0, wOl, t('fc_zone_ol'));
    const ob = ol.map((m, i) => `${i ? 'L' : 'M'}${ox(i).toFixed(1)} ${y(m.normal_hi).toFixed(1)}`).join('') +
      ol.slice().reverse().map((m, j) => `L${ox(n - 1 - j).toFixed(1)} ${y(m.normal_lo).toFixed(1)}`).join('') + 'Z';
    marks += `${n > 1 ? `<path class="band band--ol" d="${ob}"/>` : `<rect class="band band--ol" x="${ox(0) - 8}" y="${y(ol[0].normal_hi)}" width="16" height="${Math.max(2, y(ol[0].normal_lo) - y(ol[0].normal_hi))}"/>`}
      <path class="ol__normal" d="${path(ol.map((m, i) => [ox(i), y(m.normal_mean)]))}"/>
      <path class="ol__persist" d="${path(ol.map((m, i) => [ox(i), y(m.persist_mean)]))}"/>
      ${ol.map((m, i) => `<circle class="ol__dot" cx="${ox(i).toFixed(1)}" cy="${y(m.persist_mean).toFixed(1)}" r="2.6"/>
        <text x="${ox(i).toFixed(1)}" y="${H - 8}" text-anchor="middle">${esc(monthShort(+m.month.slice(5, 7) - 1))}</text>`).join('')}`;
    ol.forEach((m, i) => hits.push({ x: ox(i), y: y(m.persist_mean), k: `${mLabel(m.month)} · ${t('hv_est')}`,
      v: [`${t('fc_persist')}: ${fmt(m.persist_mean, 1)} °C`, `${t('fc_backnormal')}: ${fmt(m.normal_mean, 1)} °C`] }));
    cur = x0 + wOl;
  }
  // projection: bracket for the model range, tick for the median
  if (pj) {
    const x0 = cur + GAP, cx = x0 + wPj / 2;
    zones += zoneHead(x0, wPj, t('fc_zone_pj'));
    const med = y(base + pj.d_temp_c!);
    const r = pj.d_temp_range?.length === 2 ? [y(base + pj.d_temp_range[1]), y(base + pj.d_temp_range[0])] : [med - 6, med + 6];
    marks += `<path class="pj__br" d="M${cx - 9} ${r[0]}H${cx}V${r[1]}H${cx - 9}"/><line class="pj__med" x1="${cx - 14}" x2="${cx + 8}" y1="${med}" y2="${med}"/>
      <text class="pj__v" x="${cx + 12}" y="${med + 4}">${pj.d_temp_c! > 0 ? '+' : ''}${fmt(pj.d_temp_c!, 1)}°</text>
      <line class="pj__base" x1="${x0}" x2="${x0 + wPj}" y1="${y(base)}" y2="${y(base)}"/>`;
    const rg = pj.d_temp_range?.length === 2 ? pj.d_temp_range : [pj.d_temp_c!, pj.d_temp_c!];
    hits.push({ x: cx, y: med, k: t('hv_proj_k'), v: [t('hv_proj', { d: sgn(pj.d_temp_c!), a: sgn(rg[0]), b: sgn(rg[1]) }), t('db_proj_label')] });
  }

  const label = lastRow ? fmt(lastRow.t_mean!, 1) : '';
  return `<svg class="chart chart--season chart--plus" style="--w:${Math.round(W)}" viewBox="0 0 ${W} ${H}" role="img" tabindex="0" data-hover aria-label="${esc(t('fc_chart_desc') + ' ' + t('hv_kbd'))}">
    <g class="grid">${ticks.join('')}</g>
    <path class="band" d="${band}"/><path class="mean" d="${mean}"/>
    <g class="zones">${zones}</g>
    <path class="line" d="${path(pts)}"/>
    ${last ? `<circle class="dot" cx="${last[0]}" cy="${last[1]}" r="4.5"/><text class="last" x="${last[0] - 8}" y="${last[1] - 12}" text-anchor="end">${esc(label)} °C${o.provisional ? ' · ' + esc(t('prov')) : ''}</text>` : ''}
    <g class="plus">${marks}</g>
    <g class="axis">${months}</g>
    ${hitsHtml(hits)}
  </svg>`;
}

/** "14 Nov" from an ISO date. */
export function d2(iso: string): string {
  const d = new Date(iso.slice(0, 10) + 'T12:00:00Z');
  if (isNaN(d.getTime())) return iso;
  return `${d.getUTCDate()} ${monthShort(d.getUTCMonth())}`;
}
