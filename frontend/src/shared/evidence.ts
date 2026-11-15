// Helpers that read the evidence package without assuming which readings exist (new readings just appear).
import type { Bi, EvidencePackage, Reading } from './contract';
import { fmt, pick, t } from './i18n';

export const findReading = (ev: EvidencePackage | null | undefined, prefix: string): Reading | undefined =>
  ev?.readings.find((r) => r.id.startsWith(prefix));

/** Normal period quoted in the temperature reading ("1985–2024"), if any. */
export function normalPeriod(ev: EvidencePackage | null | undefined): string | null {
  const r = findReading(ev, 'temp_anomaly');
  const m = `${r?.method ?? ''} ${r?.dataset ?? ''}`.match(/(\d{4})\s*[–-]\s*(\d{4})/);
  return m ? `${m[1]}–${m[2]}` : null;
}

/** The three hero rows: temperature, soil moisture when present (else water balance), crop vigor. */
export function keyReadings(ev: EvidencePackage | null | undefined): Reading[] {
  if (!ev) return [];
  const rows = [findReading(ev, 'temp_anomaly'), findReading(ev, 'soil_moisture') ?? findReading(ev, 'water_balance'), findReading(ev, 'ndvi')];
  return rows.filter((r): r is Reading => !!r);
}

/** Short label for tight rows (hero phone, section peek); the full label stays in the title attribute. */
export function shortLabel(r: Reading): string | undefined {
  if (r.id.startsWith('temp')) return t('r_temp_vs');
  if (r.id.startsWith('soil_moisture')) return t('r_soil');
  if (r.id === 'rain_30d' || r.id.startsWith('rain')) return t('r_rain');
  // Named after the crop's leading pest by the backend ("Pea aphid generations this season"); older packages: generic.
  if (r.id.startsWith('pest')) return r.label.en.startsWith('Green peach aphid') ? t('r_pest') : pick(r.label).replace(/ (this season|esta campaña)$/, '');
  if (r.id === 'slope') return t('r_slope');
  if (r.id === 'grace') return t('r_storage');
  if (r.id.startsWith('water_balance')) return t('r_water');
  if (r.id.startsWith('ndvi')) return t('r_vigor');
  return undefined;
}

/** A one-line pre-diagnosis built only from the package's own signals (used when the backend sends none). */
export function derivedHeadline(ev: EvidencePackage | null | undefined): Bi | null {
  if (!ev) return null;
  const parts: string[][] = [];
  const wb = findReading(ev, 'water_balance');
  if (wb?.value != null && wb.value < -30) parts.push([t('hl_water')]);
  if (ev.pests.some((p) => p.generations_now != null && p.generations_normal != null && p.generations_now > p.generations_normal)) parts.push([t('hl_pests')]);
  const tr = findReading(ev, 'temp_anomaly');
  if (!parts.length && tr?.value != null && tr.value > 1) parts.push([t('hl_heat')]);
  if (!parts.length) return null;
  const text = t('hl_lead', { x: parts.map((p) => p[0]).join(t('hl_and')) });
  return { en: text, es: text };
}

/** Signed number text from a reading, e.g. "+5.0 °C". */
export function signed(v: number, unit: string, digits = 1): string {
  return `${v > 0 ? '+' : v < 0 ? '−' : ''}${fmt(Math.abs(v), digits)}${unit ? ' ' + unit : ''}`;
}
