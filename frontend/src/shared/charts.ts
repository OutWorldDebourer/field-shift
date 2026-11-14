// Hand-written SVG charts: "This season vs 40-year normal" (pale band + amber line), NDVI series,
// 12-month crop calendar strip and 1–5 dot scores. Pure functions returning markup.
import type { EvidencePackage } from './contract';
import { esc } from './dom';
import { fmt, fmtDate, lang, monthShort, t } from './i18n';

type Climate = EvidencePackage['climate_monthly'];

export function seasonChart(rows: Climate, opts: { w?: number; h?: number; id?: string } = {}): string {
  const W = opts.w ?? 640, H = opts.h ?? 240, L = 40, R = 14, T = 16, B = 30;
  const valid = rows.filter((r) => r.normal_lo != null);
  if (valid.length < 2) return `<p class="chart-empty">${esc(t('db_no_evidence'))}</p>`;
  const vals = rows.flatMap((r) => [r.normal_lo, r.normal_hi, r.t_mean ?? r.normal_mean]);
  const lo = Math.floor(Math.min(...vals) - 0.5), hi = Math.ceil(Math.max(...vals) + 0.5);
  const x = (i: number) => L + (i / (rows.length - 1)) * (W - L - R);
  const y = (v: number) => T + (1 - (v - lo) / (hi - lo)) * (H - T - B);
  const band = rows.map((r, i) => `${i ? 'L' : 'M'}${x(i).toFixed(1)} ${y(r.normal_hi).toFixed(1)}`).join('') +
    rows.slice().reverse().map((r, j) => `L${x(rows.length - 1 - j).toFixed(1)} ${y(r.normal_lo).toFixed(1)}`).join('') + 'Z';
  const mean = rows.map((r, i) => `${i ? 'L' : 'M'}${x(i).toFixed(1)} ${y(r.normal_mean).toFixed(1)}`).join('');
  const pts = rows.map((r, i) => (r.t_mean == null ? null : [x(i), y(r.t_mean)] as const));
  let line = '', pen = false;
  pts.forEach((p) => { if (!p) { pen = false; return; } line += `${pen ? 'L' : 'M'}${p[0].toFixed(1)} ${p[1].toFixed(1)}`; pen = true; });
  const last = [...pts].reverse().find(Boolean);
  const lastRow = [...rows].reverse().find((r) => r.t_mean != null);
  const ticks: string[] = [];
  for (let v = lo; v <= hi; v += Math.max(1, Math.round((hi - lo) / 4))) ticks.push(`<g class="tick"><line x1="${L}" x2="${W - R}" y1="${y(v)}" y2="${y(v)}"/><text x="${L - 8}" y="${y(v) + 4}" text-anchor="end">${v}°</text></g>`);
  const months = rows.map((r, i) => {
    const m = +r.month.slice(5, 7) - 1;
    return i % (rows.length > 8 ? 2 : 1) === 0 || i === rows.length - 1 ? `<text x="${x(i)}" y="${H - 8}" text-anchor="middle">${esc(monthShort(m))}</text>` : '';
  }).join('');
  const label = lastRow ? fmt(lastRow.t_mean!, 1) : '';
  return `<svg class="chart chart--season" viewBox="0 0 ${W} ${H}" role="img" aria-label="${esc(t('db_chart_desc'))}">
    <g class="grid">${ticks.join('')}</g>
    <path class="band" d="${band}"/><path class="mean" d="${mean}"/>
    <path class="line" d="${line}"/>
    ${last ? `<circle class="dot" cx="${last[0]}" cy="${last[1]}" r="4.5"/><text class="last" x="${last[0] - 8}" y="${last[1] - 12}" text-anchor="end">${esc(label)} °C</text>` : ''}
    <g class="axis">${months}</g>
  </svg>`;
}

export function ndviChart(series: EvidencePackage['ndvi_series'], opts: { w?: number; h?: number } = {}): string {
  const W = opts.w ?? 640, H = opts.h ?? 150, L = 34, R = 14, T = 12, B = 26;
  if (series.length < 2) return `<p class="chart-empty">${esc(t('db_no_evidence'))}</p>`;
  const ts = series.map((s) => Date.parse(s.date));
  const t0 = Math.min(...ts), t1 = Math.max(...ts);
  const x = (v: number) => L + ((v - t0) / (t1 - t0 || 1)) * (W - L - R);
  const y = (v: number) => T + (1 - v / 0.8) * (H - T - B);
  const d = series.map((s, i) => `${i ? 'L' : 'M'}${x(ts[i]).toFixed(1)} ${y(s.ndvi).toFixed(1)}`).join('');
  const dots = series.map((s, i) => `<circle cx="${x(ts[i]).toFixed(1)}" cy="${y(s.ndvi).toFixed(1)}" r="3"><title>${esc(fmtDate(s.date))} · NDVI ${s.ndvi.toFixed(2)} · ${esc(s.sensor)}</title></circle>`).join('');
  const grid = [0, 0.2, 0.4, 0.6, 0.8].map((v) => `<g class="tick"><line x1="${L}" x2="${W - R}" y1="${y(v)}" y2="${y(v)}"/><text x="${L - 6}" y="${y(v) + 4}" text-anchor="end">${v.toFixed(1)}</text></g>`).join('');
  const first = series[0], last = series[series.length - 1];
  return `<svg class="chart chart--ndvi" viewBox="0 0 ${W} ${H}" role="img" aria-label="NDVI ${esc(fmtDate(first.date))} – ${esc(fmtDate(last.date))}">
    <g class="grid">${grid}</g><path class="line-w" d="${d}"/><g class="dots">${dots}</g>
    <g class="axis"><text x="${L}" y="${H - 6}">${esc(fmtDate(first.date))}</text><text x="${W - R}" y="${H - 6}" text-anchor="end">${esc(fmtDate(last.date))}</text></g>
  </svg>`;
}

const CROPS: Record<string, { en: string; es: string }> = {
  pea: { en: 'Pea', es: 'Arveja' }, lettuce: { en: 'Lettuce', es: 'Lechuga' }, maize: { en: 'Maize', es: 'Maíz' },
  bean: { en: 'Beans', es: 'Frijol' }, gm: { en: 'Green manure', es: 'Abono verde' }, bare: { en: 'Bare soil', es: 'Suelo desnudo' },
  fallow: { en: 'Fallow', es: 'Descanso' }, potato: { en: 'Potato', es: 'Papa' }, oat_vetch: { en: 'Oat + vetch', es: 'Avena + vicia' },
};
export const cropName = (k: string): string => CROPS[k]?.[lang()] ?? k.replace(/_/g, ' ');

/** Group a 12-slot timeline into runs [{crop, start, len}]. */
export function runs(timeline: string[]): { crop: string; start: number; len: number }[] {
  const out: { crop: string; start: number; len: number }[] = [];
  timeline.forEach((c, i) => { const last = out[out.length - 1]; if (last && last.crop === c) last.len++; else out.push({ crop: c, start: i, len: 1 }); });
  return out;
}

/** 12-month strip. `plan` = cartographic blue (a proposal), otherwise neutral white (a record). */
export function calendarStrip(timeline: string[], startMonth: number, plan = false, compact = false): string {
  const rs = runs(timeline);
  const cells = rs.map((r) => {
    const empty = r.crop === 'bare' || r.crop === 'fallow';
    return `<li class="cal__run${empty ? ' is-bare' : ''} cal--${esc(r.crop)}" style="grid-column:${r.start + 1} / span ${r.len}" title="${esc(cropName(r.crop))}"><span>${esc(r.len >= (compact ? 3 : 2) || !compact ? cropName(r.crop) : '')}</span></li>`;
  }).join('');
  const months = timeline.map((_, i) => `<span>${esc(monthShort((startMonth + i) % 12).slice(0, compact ? 1 : 3))}</span>`).join('');
  return `<div class="cal${plan ? ' cal--plan' : ''}${compact ? ' cal--compact' : ''}"><ol class="cal__runs">${cells}</ol><div class="cal__months" aria-hidden="true">${months}</div></div>`;
}

/** Season-by-season sequence for timelines that are not 12 months: one chip per season, arrows between. */
export function sequenceStrip(timeline: string[], plan = false): string {
  return `<ol class="seq${plan ? ' seq--plan' : ''}">${timeline.map((c, i) => `<li class="seq__c${c === 'bare' || c === 'fallow' ? ' is-bare' : ''}" title="${esc(cropName(c))}">${i ? '<i aria-hidden="true">→</i>' : ''}<span>${esc(cropName(c))}</span></li>`).join('')}</ol>`;
}

/** 12-month strip when the timeline has 12 slots, otherwise the season sequence. */
export function timelineStrip(timeline: string[], startMonth: number, plan = false, compact = false): string {
  return timeline.length === 12 ? calendarStrip(timeline, startMonth, plan, compact) : sequenceStrip(timeline, plan);
}

export function dots(n: number, label: string): string {
  const v = Math.max(0, Math.min(5, Math.round(n)));
  return `<span class="dots" role="img" aria-label="${esc(label)}: ${esc(t('sc_of5', { n: v }))}">${[1, 2, 3, 4, 5].map((i) => `<i class="${i <= v ? 'on' : ''}"></i>`).join('')}</span>`;
}
