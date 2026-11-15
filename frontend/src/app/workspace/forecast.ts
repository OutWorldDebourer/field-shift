// Forecast controls and data for the "This season vs 40-year normal" card: horizon 3/5/7/10 days (NASA GEOS-FP,
// GET /api/forecast), season outlook 0–6 months (GET /api/power/outlook, "Estimate, not a forecast"), and the
// 2031–2050 projection toggle (evidence.projection). Choices persist per device; responses are cached per field.
import { getForecast, getOutlook, type ClientAnalysis } from '../../shared/api';
import type { EvidencePackage, Forecast, Outlook } from '../../shared/contract';
import { esc } from '../../shared/dom';
import { loadbarHtml, startLoadbar, type Loadbar } from '../../shared/loadbar';
import { fmt, t } from '../../shared/i18n';
import { normalPeriod } from '../../shared/evidence';
import { installChartHover } from '../../shared/charthover';
import { d2, seasonPlusChart, type Proj } from './forecast-chart';

export type EvidenceX = EvidencePackage & { forecast?: Forecast | null; outlook?: Outlook | null };
export interface FcState { days: number; months: number; proj: boolean }

const KEY = 'fieldshift.forecast';
const DAYS = [3, 5, 7, 10];
export const fc: FcState = (() => {
  const d: FcState = { days: 10, months: 3, proj: true };
  try { const s = JSON.parse(localStorage.getItem(KEY) ?? 'null') as Partial<FcState> | null; if (s) Object.assign(d, s); } catch { /* blocked */ }
  if (!DAYS.includes(d.days)) d.days = 10;
  d.months = Math.max(0, Math.min(6, Math.round(d.months)));
  return d;
})();
const save = () => { try { localStorage.setItem(KEY, JSON.stringify(fc)); } catch { /* blocked */ } };

// ---------- data ----------
const fcCache = new Map<string, Forecast | null>();
const olCache = new Map<string, Outlook | null>();
const cell = (a: ClientAnalysis) => `${a.field.center.lat.toFixed(3)},${a.field.center.lon.toFixed(3)}`;

/** Forecast for the chosen horizon if already known (embedded in the evidence or fetched before). */
export function forecastNow(a: ClientAnalysis): Forecast | null | undefined {
  const ev = a.evidence as EvidenceX | null;
  const emb = ev?.forecast;
  if (emb?.days?.length && (emb.days.length >= fc.days || a.offline)) return { ...emb, days: emb.days.slice(0, fc.days) };
  const c = fcCache.get(`${cell(a)}:${fc.days}`) ?? fcCache.get(`${cell(a)}:10`);
  if (c) return { ...c, days: c.days.slice(0, fc.days) };
  return fcCache.has(`${cell(a)}:${fc.days}`) ? null : undefined;
}
export function outlookNow(a: ClientAnalysis): Outlook | null | undefined {
  if (!fc.months) return null;
  const ev = a.evidence as EvidenceX | null;
  const emb = ev?.outlook;
  if (emb?.months?.length && (emb.months.length >= fc.months || a.offline)) return { ...emb, months: emb.months.slice(0, fc.months) };
  const c = olCache.get(`${cell(a)}:${fc.months}`) ?? olCache.get(`${cell(a)}:6`);
  if (c) return { ...c, months: c.months.slice(0, fc.months) };
  return olCache.has(`${cell(a)}:${fc.months}`) ? null : undefined;
}

/** Forget failed (null) reads for this field so the next load asks the server again. */
function forgetFailed(a: ClientAnalysis): void {
  const c = cell(a);
  for (const [k, v] of fcCache) if (v === null && k.startsWith(c + ':')) fcCache.delete(k);
  for (const [k, v] of olCache) if (v === null && k.startsWith(c + ':')) olCache.delete(k);
}

/** Fetch whatever the current choices need; resolves when both are known (or failed). */
export async function loadForecast(a: ClientAnalysis): Promise<void> {
  if (a.offline) return;
  const { lat, lon } = a.field.center;
  const jobs: Promise<unknown>[] = [];
  if (forecastNow(a) === undefined) { const k = `${cell(a)}:${fc.days}`; jobs.push(getForecast(lat, lon, fc.days).then((r) => fcCache.set(k, r))); }
  if (outlookNow(a) === undefined) { const k = `${cell(a)}:${fc.months}`; jobs.push(getOutlook(lat, lon, fc.months).then((r) => olCache.set(k, r))); }
  await Promise.all(jobs);
}

/** Fetch what the current choices need with a loading bar in the card's [data-fcload] slot, then repaint. */
export async function withForecastLoad(root: HTMLElement, a: ClientAnalysis, rerender: () => void): Promise<void> {
  const slot = root.querySelector<HTMLElement>('[data-fcload]');
  const needF = !a.offline && forecastNow(a) === undefined, needO = !a.offline && fc.months > 0 && outlookNow(a) === undefined;
  let bar: Loadbar | null = null;
  if (slot && (needF || needO)) {
    slot.hidden = false;
    slot.innerHTML = loadbarHtml(t(needF ? 'lb_forecast' : 'lb_outlook'));
    bar = startLoadbar(slot.firstElementChild as HTMLElement, needF ? 30000 : 5000, [...(needF ? ['NASA GEOS-FP · NCCS'] : []), ...(needO ? ['NASA POWER'] : [])]);
  }
  try { await loadForecast(a); bar?.done(); } catch { bar?.fail(); }
  if (slot && bar) setTimeout(() => { if (slot.isConnected) { slot.hidden = true; slot.innerHTML = ''; } }, 500);
  rerender();
}

// ---------- markup ----------
export function controlsHtml(a: ClientAnalysis): string {
  const hasProj = a.evidence?.projection?.d_temp_c != null;
  return `<div class="fc" role="group" aria-label="${esc(t('fc_controls'))}" data-fc>
    <div class="fc__grp"><span class="fc__l" id="fc-d">${esc(t('fc_horizon'))}</span>
      <div class="seg" role="radiogroup" aria-labelledby="fc-d">${DAYS.map((d) => `<button type="button" role="radio" aria-checked="${fc.days === d}" data-fcdays="${d}" aria-label="${esc(t('fc_days_long', { n: d }))}">${d}</button>`).join('')}</div></div>
    <div class="fc__grp"><label class="fc__l" for="fc-m">${esc(t('fc_outlook'))}</label>
      <span class="fc__range"><input type="range" id="fc-m" min="0" max="6" step="1" value="${fc.months}" data-fcmonths aria-valuetext="${esc(fc.months ? t('fc_months_long', { n: fc.months }) : t('fc_off'))}"><output class="mono" for="fc-m">${fc.months || esc(t('fc_off'))}</output></span></div>
    ${hasProj ? `<label class="fc__grp fc__sw"><input type="checkbox" role="switch" data-fcproj ${fc.proj ? 'checked' : ''}><span class="fc__knob" aria-hidden="true"></span><span class="fc__l">${esc(t('fc_projection_long'))}</span></label>` : ''}
  </div>`;
}

export function legendHtml(a: ClientAnalysis, f: Forecast | null | undefined, o: Outlook | null | undefined): string {
  const ev = a.evidence!;
  const items = [
    `<span><i class="k-line"></i>${esc(t('db_this_year'))}</span>`,
    `<span><i class="k-band"></i>${esc(t('db_normal'))}${normalPeriod(ev) ? ' ' + esc(normalPeriod(ev)!) : ''}</span>`,
  ];
  if (f) items.push(`<span><i class="k-dot"></i>${esc(t('fc_label'))}</span>`);
  if (o) items.push(`<span><i class="k-dash"></i>${esc(t('fc_persist'))}</span>`, `<span><i class="k-thin"></i>${esc(t('fc_backnormal'))}</span>`);
  if (fc.proj && ev.projection?.d_temp_c != null) items.push(`<span><i class="k-br"></i>${esc(t('db_proj_label'))}</span>`);
  return items.join('');
}

export function chartHtml(a: ClientAnalysis, wide: number, h: number): string {
  const ev = a.evidence!;
  const f = forecastNow(a), o = outlookNow(a);
  installChartHover();
  return seasonPlusChart({
    rows: ev.climate_monthly, forecast: f ?? null, outlook: o ?? null,
    projection: fc.proj ? (ev.projection as Proj | null) : null,
    provisional: !!ev.readings.find((r) => r.id.startsWith('temp'))?.provisional, w: wide, h,
  });
}

/** Source lines under the chart: one per layer shown, each with its own label. */
export function sourcesHtml(a: ClientAnalysis): string {
  const ev = a.evidence!;
  const last = ev.climate_monthly.length ? ev.climate_monthly[ev.climate_monthly.length - 1].month : '';
  const f = forecastNow(a), o = outlookNow(a);
  const lines = [`NASA POWER · ${esc(t('t2m'))} · ${esc(last)} · ~50 km · ${esc(t('regional'))}`];
  if (f) lines.push(`<b>${esc(t('fc_label'))}</b> · ${esc(f.source)}${f.run ? ' · ' + esc(t('fc_run', { r: f.run })) : ''} · ${esc(f.resolution)}`);
  else if (f === undefined && !a.offline) lines.push(`<span class="fc__wait">${esc(t('fc_loading'))}</span>`);
  else if (f === null && !a.offline) lines.push(`${esc(t('fc_na'))} <button type="button" class="fc__retry" data-fcretry>${esc(t('ap_retry'))}</button>`);
  if (o) lines.push(`<b>${esc(t('fc_outlook_label'))}</b> · ${esc(t('fc_outlook_src', { a: `${o.anomaly_c > 0 ? '+' : ''}${fmt(o.anomaly_c, 1)} °C` }))}`);
  else if (fc.months && o === null && !a.offline) lines.push(`${esc(t('fc_outlook_na'))} <button type="button" class="fc__retry" data-fcretry>${esc(t('ap_retry'))}</button>`);
  const p = ev.projection as Proj | null;
  if (fc.proj && p?.d_temp_c != null) {
    const r = p.d_temp_range?.length === 2 ? ` · ${esc(t('db_proj_range', { a: sg(p.d_temp_range[0]), b: sg(p.d_temp_range[1]) + ' °C' }))}` : '';
    lines.push(`<b>${esc(t('db_proj_label'))}</b> · NEX-GDDP-CMIP6 · ${esc(p.scenario)} · ${sg(p.d_temp_c)} °C${r}`);
  }
  return lines.map((l) => `<p class="dash__src mono">${l}</p>`).join('');
}
const sg = (v: number) => `${v > 0 ? '+' : v < 0 ? '−' : ''}${fmt(Math.abs(v), 2)}`;

/** Compact day-by-day table (date, min/max °C, rain mm); dates run across so it stays short. */
export function tableHtml(a: ClientAnalysis): string {
  const f = forecastNow(a);
  if (!f?.days.length) return '';
  return `<div class="fct"><p class="fct__h"><b>${esc(t('fc_table'))}</b> · <span class="mono">${esc(t('fc_label'))}</span>
      <button type="button" class="fct__sheet" data-export="sheet">${esc(t('fc_in_sheet'))}</button></p>
    <div class="fct__scroll" tabindex="0" role="region" aria-label="${esc(t('fc_table'))}"><table>
      <thead><tr><th scope="row">${esc(t('fc_date'))}</th>${f.days.map((d) => `<th scope="col">${esc(d2(d.date))}</th>`).join('')}</tr></thead>
      <tbody><tr><th scope="row">${esc(t('fc_minmax'))}</th>${f.days.map((d) => `<td>${fmt(d.tmin, 0)}/${fmt(d.tmax, 0)}</td>`).join('')}</tr>
      <tr><th scope="row">${esc(t('fc_rain'))}</th>${f.days.map((d) => `<td>${fmt(d.rain_mm, d.rain_mm > 0 && d.rain_mm < 1 ? 1 : 0)}</td>`).join('')}</tr></tbody>
    </table></div></div>`;
}

/** Wire the control bar; `rerender` repaints the chart area after a change (and again when data arrives). */
export function wireForecast(root: HTMLElement, a: () => ClientAnalysis, rerender: () => void): void {
  const after = () => { save(); rerender(); void withForecastLoad(root, a(), rerender); };
  root.addEventListener('click', (e) => {
    // A failed read is remembered so the card does not keep asking; "Retry" forgets it and asks again.
    if ((e.target as HTMLElement).closest('[data-fcretry]')) { forgetFailed(a()); after(); return; }
    const b = (e.target as HTMLElement).closest<HTMLButtonElement>('[data-fcdays]');
    if (!b) return;
    fc.days = +b.dataset.fcdays!; after();
    root.querySelector<HTMLElement>(`[data-fcdays="${fc.days}"]`)?.focus();
  });
  root.addEventListener('keydown', (e) => {
    const b = (e.target as HTMLElement).closest<HTMLButtonElement>('[data-fcdays]');
    if (!b || !['ArrowRight', 'ArrowLeft'].includes(e.key)) return;
    const i = DAYS.indexOf(fc.days);
    fc.days = DAYS[Math.max(0, Math.min(DAYS.length - 1, i + (e.key === 'ArrowRight' ? 1 : -1)))];
    e.preventDefault(); after();
    root.querySelector<HTMLElement>(`[data-fcdays="${fc.days}"]`)?.focus();
  });
  root.addEventListener('change', (e) => {
    const el = e.target as HTMLInputElement;
    if (el.matches('[data-fcmonths]')) { fc.months = +el.value; after(); root.querySelector<HTMLElement>('[data-fcmonths]')?.focus(); }
    else if (el.matches('[data-fcproj]')) { fc.proj = el.checked; after(); root.querySelector<HTMLElement>('[data-fcproj]')?.focus(); }
  });
  root.addEventListener('input', (e) => {
    const el = e.target as HTMLInputElement;
    if (!el.matches('[data-fcmonths]')) return;
    const out = el.parentElement?.querySelector('output');
    if (out) out.textContent = el.value === '0' ? t('fc_off') : el.value;
  });
}
