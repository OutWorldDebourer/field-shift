// The data row, master component from the hero phone: icon, label, value, verdict, monospace source · date,
// scale badge ("field-level" / "regional context") and a sparkline. Every number on screen goes through here.
import type { Reading } from './contract';
import { esc } from './dom';
import { icon, readingIcon } from './icons';
import { fmt, fmtDate, pick, t } from './i18n';

/** Sparkline path for a series, drawn in a w×h box. */
export function sparkline(values: (number | null)[], w = 64, h = 22, cls = 'drow__spark'): string {
  const pts = values.map((v, i) => [i, v] as const).filter((p): p is readonly [number, number] => p[1] != null);
  if (pts.length < 2) return '';
  const xs = values.length - 1 || 1;
  const vs = pts.map((p) => p[1]);
  const lo = Math.min(...vs), hi = Math.max(...vs);
  const span = hi - lo || 1;
  const P = pts.map(([i, v]) => [2 + (i / xs) * (w - 4), h - 3 - ((v - lo) / span) * (h - 6)] as const);
  const d = P.map((p, i) => `${i ? 'L' : 'M'}${p[0].toFixed(1)} ${p[1].toFixed(1)}`).join('');
  const last = P[P.length - 1];
  return `<svg class="spark ${cls}" viewBox="0 0 ${w} ${h}" aria-hidden="true"><path d="${d}"/><circle cx="${last[0].toFixed(1)}" cy="${last[1].toFixed(1)}" r="2.2"/></svg>`;
}

/** Value text: numbers with sign where it is an anomaly, words when the value is a verdict only. */
export function valueText(r: Reading): { text: string; isWord: boolean } {
  if (r.value == null) return { text: pick(r.verdict), isWord: true };
  const signed = /anomaly|balance|pest_gen|projection|grace/.test(r.id);
  const digits = Math.abs(r.value) < 1 && r.unit === '' ? 2 : Math.abs(r.value) >= 10 ? 0 : 1;
  const n = fmt(r.value, digits);
  const sign = signed && r.value > 0 ? '+' : '';
  const unit = r.unit ? (r.unit === '%' || r.unit === '°' ? r.unit : ` ${r.unit}`) : '';
  return { text: `${sign}${n.replace('-', '−')}${unit}`, isWord: false };
}

/** Short resolution for tight spots: "~0.5° × 0.625° grid cell (about 50 km)" → "~50 km". */
export function shortRes(res: string): string {
  const m = res.match(/about\s*([\d.,]+\s*k?m)/i);
  if (m) return `~${m[1]}`;
  return res.length > 14 ? res.slice(0, 13) + '…' : res;
}

/** Resolution in km parsed from the text, or null. */
export function resKm(res: string): number | null {
  const a = res.match(/about\s*([\d.]+)\s*(km|m)/i) ?? res.match(/([\d.]+)\s*(km|m)/i);
  if (!a) return null;
  const v = parseFloat(a[1]);
  return a[2].toLowerCase() === 'km' ? v : v / 1000;
}

export function badge(b: Reading['badge']): string {
  const field = b === 'field-level';
  return `<span class="badge ${field ? 'badge--field' : 'badge--regional'}" title="${esc(t(field ? 'field_level_tip' : 'regional_tip'))}">${esc(t(field ? 'field_level' : 'regional'))}</span>`;
}

export function sourceLine(r: Reading): string {
  return `${esc(r.source)} · ${esc(fmtDate(r.date))} · ${esc(r.resolution)}`;
}

/** Full data row with verdict, source, date, resolution, badge and sparkline. */
export function dataRow(r: Reading): string {
  const v = valueText(r);
  const series = r.series?.map((p) => p.v) ?? [];
  return `<div class="drow" data-reading="${esc(r.id)}">
    ${icon(readingIcon(r.id), 'drow__icon')}
    <div class="drow__label">${esc(pick(r.label))}</div>
    <div class="drow__value${r.key ? ' is-key' : ''}${v.isWord ? ' is-word' : ''}">${esc(v.text)}</div>
    <div class="drow__verdict">${v.isWord ? '' : esc(pick(r.verdict))}</div>
    ${sparkline(series)}
    <div class="drow__meta"><span class="drow__src">${sourceLine(r)}</span>${badge(r.badge)}</div>
  </div>`;
}

/** Compact row as in the hero phone: icon, label, value, source · scale, sparkline. */
export function dataRowCompact(r: Reading, srcText?: string): string {
  const v = valueText(r);
  const series = r.series?.map((p) => p.v) ?? [];
  const src = srcText ?? `${r.source} · ${shortRes(r.resolution)}`;
  const regional = r.badge === 'regional context' && srcText === undefined;
  return `<div class="drow drow--compact" data-reading="${esc(r.id)}">
    ${icon(readingIcon(r.id), 'drow__icon')}
    <div class="drow__label">${esc(pick(r.label))}</div>
    <div class="drow__value${r.key ? ' is-key' : ''}${v.isWord ? ' is-word' : ''}">${esc(v.text)}</div>
    <div class="drow__src" title="${esc(t(regional ? 'regional_tip' : 'field_level_tip'))}">${esc(src)}${regional ? ' · ' + esc(t('regional')) : ''}</div>
    ${sparkline(series, 52, 22)}
  </div>`;
}
