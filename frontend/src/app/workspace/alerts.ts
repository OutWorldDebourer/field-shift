// Alerts for the dashboard: the backend's `evidence.alerts` when present, otherwise derived from the package's own
// numbers (pest generations vs normal, water balance, temperature anomaly). Never a number that is not in the package.
import type { Bi, EvidencePackage } from '../../shared/contract';
import { findReading, signed } from '../../shared/evidence';
import { fmt, t } from '../../shared/i18n';

export type Alert = NonNullable<EvidencePackage['alerts']>[number] & { detail?: string };

/** Text built in the current language (alerts re-render on every language switch). */
const both = (fn: () => string): Bi => { const s = fn(); return { en: s, es: s }; };

export function alertsFor(ev: EvidencePackage): Alert[] {
  if (ev.alerts?.length) return ev.alerts;
  const out: Alert[] = [];
  const temp = findReading(ev, 'temp_anomaly');
  for (const p of ev.pests) {
    if (p.generations_now == null || p.generations_normal == null || p.generations_now <= p.generations_normal) continue;
    out.push({ id: `pest-${p.name}`, level: 'warning', text: both(() => t('al_pest', { name: p.name })), detail: t('al_pest_d', { now: fmt(p.generations_now!, 1), norm: fmt(p.generations_normal!, 1) }), source: 'NASA POWER degree-days', date: findReading(ev, 'pest_gen')?.date ?? temp?.date ?? '' });
  }
  const wb = findReading(ev, 'water_balance');
  if (wb?.value != null && wb.value < -20) out.push({ id: 'deficit', level: 'watch', text: both(() => t('al_water')), detail: signed(wb.value, 'mm', 0), source: wb.source, date: wb.date });
  if (temp?.value != null && temp.value > 1) out.push({ id: 'heat', level: 'info', text: both(() => t('al_heat')), detail: signed(temp.value, '°C'), source: temp.source, date: temp.date });
  return out.slice(0, 4);
}
