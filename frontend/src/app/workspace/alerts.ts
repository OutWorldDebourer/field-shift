// Alerts for the dashboard: the backend's `evidence.alerts` when present, otherwise derived from the readings.
import type { Bi, EvidencePackage } from '../../shared/contract';

export type Alert = NonNullable<EvidencePackage['alerts']>[number];

export function alertsFor(ev: EvidencePackage): Alert[] {
  if (ev.alerts?.length) return ev.alerts;
  const out: Alert[] = [];
  const by = (id: string) => ev.readings.find((r) => r.id === id);
  const push = (id: string, level: Alert['level'], text: Bi, r: { source: string; date: string }) => out.push({ id, level, text, source: r.source, date: r.date });
  const pest = by('pest_gen');
  if (pest?.value != null && pest.value > 0.5) push('pest', 'warning', { en: 'More pest generations than normal this season', es: 'Más generaciones de plagas que lo normal esta temporada' }, pest);
  const wb = by('water_balance');
  if (wb?.value != null && wb.value < -20) push('deficit', 'watch', { en: `Irrigation deficit: ${Math.round(-wb.value)} mm must come from irrigation`, es: `Déficit de riego: ${Math.round(-wb.value)} mm deben venir del riego` }, wb);
  const temp = by('temp_anomaly');
  if (temp?.value != null && temp.value > 0.8) push('heat', 'info', { en: 'Warmer than the 40-year normal', es: 'Más cálido que la normal de 40 años' }, temp);
  return out;
}
