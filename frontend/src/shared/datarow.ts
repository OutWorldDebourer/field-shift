// The data row, master component from the hero phone: icon, label, value, verdict, monospace source · date,
// scale badge ("field-level" / "regional context") and a sparkline. Every number on screen goes through here.
import type { Reading } from './contract';
import { esc } from './dom';
import { icon, readingIcon } from './icons';
import { fmt, fmtDate, fmtDateShort, pick, t } from './i18n';
import { glossBtn } from './glosspanel';
import { termFor } from './glossary';
import { loc } from './localize';

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
export function valueText(r: Reading, compact = false): { text: string; isWord: boolean } {
  if (r.value == null) return { text: pick(r.verdict), isWord: true };
  const signed = /anomaly|balance|projection|grace/.test(r.id);
  const digits = Math.abs(r.value) < 1 ? 2 : Math.abs(r.value) >= 100 ? 0 : 1;
  const n = fmt(r.value, digits);
  const sign = signed && r.value > 0 ? '+' : '';
  const u = loc(compact ? r.unit.replace(/\s+vs\s.*$/, '').replace(/^NDVI$/i, '') : r.unit);
  const unit = u ? (u === '%' || u === '°' ? u : ` ${u}`) : '';
  return { text: `${sign}${n.replace('-', '−')}${unit}`, isWord: false };
}

/** Short resolution for tight spots: "~0.5° × 0.625° grid cell (about 50 km)" → "~50 km". */
export function shortRes(res: string): string {
  const about = res.match(/(?:about|\(~)\s*([\d.,]+\s*k?m)\b/i);
  const lead = res.match(/^~?\s*([\d.,]+\s*k?m)\b/i);
  const out = about ? `~${about[1]}` : lead ? `${res.trim().startsWith('~') ? '~' : ''}${lead[1]}` : res.length > 14 ? res.slice(0, 13) + '…' : res;
  return out.replace(/ /g, ' ');
}

/** Resolution in km parsed from the text, or null. */
export function resKm(res: string): number | null {
  const a = res.match(/about\s*([\d.]+)\s*(km|m)/i) ?? res.match(/([\d.]+)\s*(km|m)/i);
  if (!a) return null;
  const v = parseFloat(a[1]);
  return a[2].toLowerCase() === 'km' ? v : v / 1000;
}

/** Visible "provisional" tag for readings flagged by the backend. */
export function provTag(r: Pick<Reading, 'provisional' | 'note'>): string {
  if (!r.provisional) return '';
  return `<span class="prov" title="${esc(r.note ? pick(r.note) : t('prov_tip'))}">${esc(t('prov'))}</span>`;
}

export function badge(b: Reading['badge']): string {
  const field = b === 'field-level';
  return `<span class="badge ${field ? 'badge--field' : 'badge--regional'}" title="${esc(t(field ? 'field_level_tip' : 'regional_tip'))}">${esc(t(field ? 'field_level' : 'regional'))}</span>`;
}

export function sourceLine(r: Reading): string {
  return `${esc(loc(r.source))} · ${esc(fmtDate(r.date))} · ${esc(loc(r.resolution))}`;
}

/** Short source line for tiles and compact rows; the full line goes in a tooltip. */
export function sourceShort(r: Reading): string {
  const b = basePeriod(r);
  return `${esc(loc(r.source))} · <span class="nw">${esc(fmtDateShort(r.date))}</span>${b ? ` · <span class="nw">vs ${b[1]}–${b[2]}</span>` : ''} · <span class="nw">${esc(shortRes(r.resolution))}</span>`;
}

const sourceFull = (r: Reading) => `${r.source} · ${r.dataset} · ${fmtDate(r.date)} · ${r.resolution} · ${r.license}${r.method ? ' · ' + r.method : ''}`;
const basePeriod = (r: Reading) => (r.id.startsWith('temp') ? (r.method ?? '').match(/(\d{4})\s*[–-]\s*(\d{4})/) : null);

/** Verdict without a repeated figure: "much warmer than normal (+5.0 °C)" → "much warmer than normal". */
export function cleanVerdict(r: Reading): string {
  const v = pick(r.verdict);
  return v.replace(/\s*\((?:[+−-]?\d[^)]*)\)\s*$/, '');
}

/** Full data row with verdict, source, date, resolution, badge and sparkline. */
export function dataRow(r: Reading, label?: string): string {
  const v = valueText(r);
  const series = r.series?.map((p) => p.v) ?? [];
  return `<div class="drow" data-reading="${esc(r.id)}">
    ${icon(readingIcon(r.id), 'drow__icon')}
    <div class="drow__label"${label ? ` title="${esc(pick(r.label))}"` : ''}>${esc(label ?? pick(r.label))}${glossBtn(termFor(r.id), pick(r.label))}</div>
    <div class="drow__value${r.key ? ' is-key' : ''}${v.isWord ? ' is-word' : ''}">${esc(v.text)}${provTag(r)}</div>
    <div class="drow__verdict">${v.isWord ? '' : esc(cleanVerdict(r))}</div>
    ${sparkline(series)}
    <div class="drow__meta"><span class="drow__src" title="${esc(sourceFull(r))}">${sourceShort(r)}</span>${badge(r.badge)}</div>
  </div>`;
}

/** Compact row as in the hero phone: icon, label, value, source · scale, sparkline. */
export function dataRowCompact(r: Reading, srcText?: string, label?: string, verdict = false): string {
  const v = valueText(r, true);
  const series = r.series?.map((p) => p.v) ?? [];
  const src = srcText ?? '';
  return `<div class="drow drow--compact" data-reading="${esc(r.id)}">
    ${icon(readingIcon(r.id), 'drow__icon')}
    <div class="drow__label" title="${esc(pick(r.label))}">${esc(label ?? pick(r.label))}${glossBtn(termFor(r.id), pick(r.label))}</div>
    <div class="drow__value${r.key ? ' is-key' : ''}${v.isWord ? ' is-word' : ''}">${esc(v.text)}</div>
    ${verdict ? `<div class="drow__verdict drow__verdict--c">${esc(v.isWord ? '' : cleanVerdict(r))}</div>` : ''}
    <div class="drow__src" title="${esc(sourceFull(r))}">${srcText ? esc(src) : sourceShort(r)} ${badge(r.badge)}${provTag(r)}</div>
    ${sparkline(series, 52, 22)}
  </div>`;
}
