// "Recommended topics" under the dashboard alerts, derived only from numbers on screen (forecast, outlook,
// water balance, pest generations). A tap writes the question into the chat box and switches to Chat; never sends.
import type { EvidencePackage, Forecast, Outlook } from '../../shared/contract';
import { esc } from '../../shared/dom';
import { findReading, signed } from '../../shared/evidence';
import { fmt, t } from '../../shared/i18n';
import { icon } from '../../shared/icons';
import { pestTitle } from '../../shared/pests';

export interface Topic { id: string; label: string; prompt: string }

const HOT_C = 27, RAIN_MM = 5;

export function topicsFor(ev: EvidencePackage, f: Forecast | null | undefined, o: Outlook | null | undefined): Topic[] {
  const out: Topic[] = [];
  if (f?.days.length) {
    const tmax = Math.max(...f.days.map((d) => d.tmax));
    if (tmax >= HOT_C) out.push({ id: 'hot', label: t('tp_hot'), prompt: t('tp_hot_q', { t: fmt(tmax, 0) }) });
    const rain = f.days.reduce((s, d) => s + (d.rain_mm || 0), 0);
    if (rain >= RAIN_MM) out.push({ id: 'rain', label: t('tp_rain'), prompt: t('tp_rain_q', { mm: fmt(rain, 0) }) });
  }
  const wb = findReading(ev, 'water_balance');
  if (wb?.value != null && wb.value < -20) out.push({ id: 'water', label: t('tp_water'), prompt: t('tp_water_q', { mm: signed(wb.value, 'mm', 0) }) });
  const pests = ev.pests
    .filter((p) => p.generations_now != null && p.generations_normal != null && p.generations_now > p.generations_normal)
    .sort((a, b) => (b.generations_now! - b.generations_normal!) - (a.generations_now! - a.generations_normal!));
  for (const p of pests.slice(0, 2)) {
    out.push({ id: `pest-${p.scientific}`, label: t('tp_pest', { p: pestTitle(p) }), prompt: t('tp_pest_q', { p: pestTitle(p), now: fmt(p.generations_now!, 1), norm: fmt(p.generations_normal!, 1) }) });
  }
  if (o && o.anomaly_c >= 1 && out.length < 4) out.push({ id: 'warm', label: t('tp_warm'), prompt: t('tp_warm_q', { a: signed(o.anomaly_c, '°C') }) });
  return out.slice(0, 4);
}

export function topicsHtml(list: Topic[]): string {
  if (!list.length) return '';
  return `<div class="topics"><h4 class="topics__h">${esc(t('tp_title'))}</h4><p class="topics__hint">${esc(t('tp_hint'))}</p>
    <ul class="topics__list">${list.map((x) => `<li><button type="button" class="topic" data-topic="${esc(x.prompt)}">${icon('chat')}<span>${esc(x.label)}</span></button></li>`).join('')}</ul></div>`;
}
