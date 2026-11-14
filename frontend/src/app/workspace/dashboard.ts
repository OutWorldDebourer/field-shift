// Dashboard tab (App 4): field map with outline and an honest-scale layer switch, metric tiles from
// evidence.readings, "This season vs 40-year normal", NDVI series, 12-month crop calendar, alerts, projection.
// `dashboardHtml` is pure markup (also used by the index preview); `mountDashboardMap` adds the live Leaflet map.
import type * as Leaflet from 'leaflet';
import type { ClientAnalysis } from '../../shared/api';
import type { EvidencePackage, Reading } from '../../shared/contract';
import { learnCropNames, ndviChart, timelineStrip } from '../../shared/charts';
import { dataRow, sourceLine, valueText, badge, shortRes, resKm, provTag } from '../../shared/datarow';
import { shortLabel } from '../../shared/evidence';
import { esc } from '../../shared/dom';
import { fmt, fmtDateShort, pick, t } from '../../shared/i18n';
import { icon } from '../../shared/icons';
import type { StrKey } from '../../shared/strings';
import { glossBtn, glossHeaderBtn } from '../../shared/glosspanel';
import { pestTitle } from '../../shared/pests';
import { loc } from '../../shared/localize';
import { alertsFor } from './alerts';
import { chartHtml, controlsHtml, forecastNow, legendHtml, loadForecast, outlookNow, sourcesHtml, tableHtml, wireForecast } from './forecast';
import { topicsFor, topicsHtml } from './topics';
import './dashboard.css';

export type LayerKey = 'vigor' | 'soil' | 'temp' | 'slope';
export const LAYERS: { key: LayerKey; label: StrKey; reading: string }[] = [
  { key: 'vigor', label: 'ly_vigor', reading: 'ndvi' },
  { key: 'soil', label: 'ly_soil', reading: 'soil_moisture' },
  { key: 'temp', label: 'ly_temp', reading: 'temp_anomaly' },
  { key: 'slope', label: 'ly_slope', reading: 'slope' },
];

const TILE_ORDER = ['temp_anomaly', 'soil_moisture', 'rain_30d', 'ndvi', 'water_balance', 'pest_gen', 'slope', 'grace'];

export function tilesFor(ev: EvidencePackage, max = 6): Reading[] {
  const rank = (r: Reading) => { const i = TILE_ORDER.indexOf(r.id); return i < 0 ? 99 : i; };
  return ev.readings.filter((r) => r.id !== 'projection').sort((a, b) => rank(a) - rank(b)).slice(0, max);
}

let mapBase = '';
export function setMapBase(s: string) { mapBase = s; document.querySelectorAll('[data-mapbase]').forEach((e) => { e.textContent = s; }); }

export function layerLegend(ev: EvidencePackage | null, key: LayerKey): string {
  const r = ev?.readings.find((x) => x.id === LAYERS.find((l) => l.key === key)!.reading);
  if (!r) return `<p class="dmap__note">${esc(t(ev ? 'ly_na' : 'db_no_evidence'))}</p>`;
  const v = valueText(r);
  const note = r.badge === 'field-level' ? t('db_layer_field', { res: shortRes(r.resolution) }) : t('db_layer_cell', { res: shortRes(r.resolution) });
  return `<p class="dmap__val"><b>${esc(pick(r.label))}</b> <span class="${r.key ? 'amber' : ''}">${esc(v.text)}</span>${provTag(r)}</p>
    <p class="dmap__src mono">${sourceLine(r)}</p>
    <p class="dmap__note">${badge(r.badge)} ${esc(note)}</p>${ev ? `<p class="dmap__scale mono">${esc(scaleNote(ev))}</p>` : ''}<p class="dmap__base mono" data-mapbase>${esc(mapBase)}</p>`;
}

function scaleNote(ev: EvidencePackage): string {
  const f = ev.readings.find((r) => r.badge === 'field-level')?.resolution ?? '10 m';
  const r = ev.readings.find((x) => x.id === 'soil_moisture')?.resolution ?? ev.readings.find((x) => x.badge === 'regional context')?.resolution ?? '9 km';
  return t('db_scale_note', { f: shortRes(f), r: shortRes(r) });
}

function currentTimeline(a: ClientAnalysis): string[] | null {
  const rot = a.sections.find((s) => s.key === 'rotation');
  const cur = rot?.content?.strategies?.find((s) => s.current) ?? rot?.content?.strategies?.[0];
  return cur?.timeline?.length ? cur.timeline : null;
}

function currentPlan(a: ClientAnalysis) {
  const rot = a.sections.find((s) => s.key === 'rotation');
  return rot?.content?.strategies?.find((s) => s.current) ?? null;
}

export function dashboardHtml(a: ClientAnalysis, opts: { preview?: boolean; layer?: LayerKey } = {}): string {
  const ev = a.evidence;
  learnCropNames([a.crop_current, ...(a.crop_history ?? [])]);
  const layer = opts.layer ?? 'vigor';
  const map = `<section class="dash__map glass" aria-label="${esc(t('db_map_label'))}">
      <div class="dmap" data-dmap></div><div class="reticle dmap__ret" data-dret hidden><i></i></div>
      <div class="dmap__layers" role="radiogroup" aria-label="${esc(t('db_layers'))}">
        ${LAYERS.map((l) => { const na = !!ev && !ev.readings.some((r) => r.id.startsWith(l.reading)); return `<button type="button" role="radio" aria-checked="${l.key === layer}" data-layer="${l.key}"${na ? ` aria-disabled="true" title="${esc(t('ly_na'))}"` : ''}${opts.preview ? ' tabindex="-1"' : ''}>${esc(t(l.label))}</button>`; }).join('')}
      </div>
      <div class="dmap__legend" data-legend aria-live="polite">${layerLegend(ev, layer)}</div>
    </section>`;
  if (!ev || !ev.readings.length) {
    const gaps = ev?.gaps ?? [];
    return `<div class="dash dash--empty">${map}<div class="dash__side"><section class="glass dash__card dash__wait">
      <h3 class="serif">${esc(t('db_no_evidence'))}</h3>
      ${gaps.length ? `<h4 class="dash__h">${esc(t('ws_gaps_title'))}</h4><ul class="gaps">${gaps.map((g) => `<li>${icon('info')}${esc(pick(g.text))}</li>`).join('')}</ul>` : ''}
      ${opts.preview || a.readOnly ? '' : `<button type="button" class="btn btn--solid btn--sm" data-build-evidence>${icon('orbit')}${esc(t('db_build_evidence'))}</button>`}
      <p class="status-line" data-build-status hidden><i class="tick"></i>${esc(t('db_building'))}</p>
    </section></div></div>`;
  }
  const tiles = tilesFor(ev, 6).map((r) => `<li class="tile-m glass">${dataRow(r, shortLabel(r) ?? pick(r.label))}</li>`).join('');
  const alerts = alertsFor(ev);
  const tl = currentTimeline(a);
  const startMonth = new Date(ev.generated_at).getUTCMonth() + 1;
  const proj = ev.projection;
  const pests = ev.pests.filter((p) => p.generations_now != null);
  return `<div class="dash${opts.preview ? ' dash--preview' : ''}">
    ${map}
    <div class="dash__side">
      <div class="dash__tools">${glossHeaderBtn('')}</div>
      <ul class="dash__tiles" aria-label="${esc(t('db_readings'))}">${tiles}</ul>
      ${seasonCard(a, !!opts.preview)}
      <div class="dash__row dash__row--3">
        ${tl ? `<section class="glass dash__card dash__cal"><h3 class="dash__h">${esc(t(tl.length === 12 ? 'db_calendar' : 'db_sequence'))}</h3>${timelineStrip(tl, startMonth % 12, true, !!opts.preview)}${currentPlan(a) ? `<p class="dash__plan">${esc(pick(currentPlan(a)!.name))}</p><p class="dash__plan-why">${esc(pick(currentPlan(a)!.why))}</p>` : ''}${opts.preview || a.readOnly ? '' : `<button type="button" class="btn btn--glass btn--sm" data-goto="rotation">${icon('rotation')}${esc(t('ws_open_rotation'))}</button>`}</section>` : ''}
        <section class="glass dash__card dash__alerts"><h3 class="dash__h">${esc(t('db_alerts'))}</h3>
          ${alerts.length ? `<ul class="alerts">${alerts.map((al) => `<li class="al al--${al.level}">${icon(al.level === 'info' ? 'info' : 'alert')}<span>${esc(pick(al.text))}<small class="mono">${al.detail ? esc(al.detail) + ' · ' : ''}${esc(al.source)} · ${esc(fmtDateShort(al.date))}</small></span></li>`).join('')}</ul>` : `<p class="chart-empty">${esc(t('db_no_alerts'))}</p>`}
          ${a.readOnly ? '' : `<div data-topics>${topicsHtml(topicsFor(ev, forecastNow(a), outlookNow(a)))}</div>`}
        </section>
        ${`<section class="glass dash__card dash__proj"><h3 class="serif dash__h3">${esc(t('db_projection'))}${glossBtn('nexgddp', t('db_projection'))}</h3><p class="proj__lbl mono">${esc(t('db_proj_label'))}</p>
          ${proj && proj.d_temp_c != null ? `<p class="proj"><span class="proj__v">${proj.d_temp_c > 0 ? '+' : ''}${fmt(proj.d_temp_c, 2)} °C</span> <span>${esc(loc(proj.horizon))} · ${esc(proj.scenario)}</span></p>
            ${projRange(proj) ? `<p class="proj__r">${esc(projRange(proj)!)}</p>` : ''}${projBase(proj) ? `<p class="proj__r">${esc(t('db_proj_base', { p: loc(projBase(proj)!) }))}</p>` : ''}
            ${proj.d_rain_pct != null ? `<p class="proj__r">${proj.d_rain_pct > 0 ? '+' : '−'}${fmt(Math.abs(proj.d_rain_pct), 1)}% ${esc(t('db_rain'))}</p>` : ''}
            <details class="proj__models"><summary class="mono">NEX-GDDP-CMIP6 · ${esc(t('db_proj_models', { n: proj.models.length }))} · ~25 km</summary><p class="mono">${esc(proj.models.join(', '))}</p></details>` : `<p class="chart-empty">${esc(t('db_projection_none'))}</p>`}
        </section>`}
      </div>
      ${`<div class="dash__row">
        <section class="glass dash__card"><h3 class="dash__h">${esc(t('db_ndvi_title'))}${glossBtn('ndvi', t('db_ndvi_title'))}</h3>${ndviLine(ev)}${ndviChart(ev.ndvi_series, { w: narrow() ? 360 : 420, h: 150 })}
          <p class="dash__src mono">${esc([...new Set(ev.ndvi_series.map((n) => n.sensor))].join(' · '))} · 10–30 m · ${esc(t('field_level'))}</p></section>
        <section class="glass dash__card"><h3 class="dash__h">${esc(t('db_pests'))}${glossBtn('pest_gen', t('db_pests'))}</h3>
          ${pests.length ? `<ul class="pests">${pests.map((p) => `<li><span><b>${esc(pestTitle(p))}</b> <i>${esc(p.scientific)}</i></span>
            <span class="mono">${fmt(p.generations_now!, 1)} ${esc(t('db_now'))} · ${p.generations_normal != null ? fmt(p.generations_normal, 1) + ' ' + esc(t('db_norm')) : '—'}</span></li>`).join('')}</ul>
            <p class="dash__src mono" title="${esc(pests.map((p) => p.citation).join(' | '))}">NASA POWER · ${esc(t('degree_days'))} · ${esc(t('regional'))}</p>` : `<p class="chart-empty">${esc(t('db_no_evidence'))}</p>`}
        </section></div>`}
    </div>
  </div>`;
}

const narrow = () => typeof innerWidth === 'number' && innerWidth < 700;

/** The season card with the forecast controls; inner parts are repainted in place when a control changes. */
function seasonCard(a: ClientAnalysis, preview = false): string {
  return `<section class="glass dash__card dash__chart" data-season-card>
    <header class="dash__ch"><h3 class="serif">${esc(t('db_chart_title'))}${glossBtn('outlook', t('db_chart_title'))}</h3></header>
    ${a.offline && !preview ? '' : `<div data-fcbox>${controlsHtml(a)}</div>`}
    ${seasonParts(a)}
  </section>`;
}
function seasonParts(a: ClientAnalysis): string {
  return `<div data-season-parts><p class="legend legend--plus">${legendHtml(a, forecastNow(a), outlookNow(a))}</p>
    <div class="dash__chartwrap">${chartHtml(a, narrow() ? 380 : 680, narrow() ? 230 : 220)}</div>
    ${sourcesHtml(a)}${tableHtml(a)}</div>`;
}

/** Forecast controls, data loading and the recommended topics for the live dashboard. */
export function mountDashboardForecast(root: HTMLElement, getA: () => ClientAnalysis): void {
  const card = root.querySelector<HTMLElement>('[data-season-card]');
  if (!card) return;
  const paint = () => {
    const a = getA();
    if (!a.evidence || !root.isConnected) return;
    const parts = card.querySelector('[data-season-parts]');
    if (parts) parts.outerHTML = seasonParts(a);
    const box = card.querySelector('[data-fcbox]');
    const focused = document.activeElement && box?.contains(document.activeElement) ? (document.activeElement as HTMLElement).dataset : null;
    if (box) box.innerHTML = controlsHtml(a);
    if (focused) {
      const sel = focused.fcdays ? `[data-fcdays="${focused.fcdays}"]` : 'fcmonths' in focused ? '[data-fcmonths]' : 'fcproj' in focused ? '[data-fcproj]' : '';
      if (sel) box?.querySelector<HTMLElement>(sel)?.focus();
    }
    const tp = root.querySelector('[data-topics]');
    if (tp) tp.innerHTML = topicsHtml(topicsFor(a.evidence, forecastNow(a), outlookNow(a)));
  };
  wireForecast(card, getA, paint);
  void loadForecast(getA()).then(paint);
}
function ndviLine(ev: EvidencePackage): string {
  const r = ev.readings.find((x) => x.id === 'ndvi');
  return r ? `<p class="dash__line">${esc(pick(r.verdict))} · ${esc(valueText(r).text)} · ${esc(fmtDateShort(r.date))}</p>` : '';
}

type Proj = NonNullable<EvidencePackage['projection']> & { d_temp_range?: [number, number]; baseline_period?: string };
const projRange = (p: Proj) => (p.d_temp_range?.length === 2 ? t('db_proj_range', { a: `${p.d_temp_range[0] > 0 ? '+' : ''}${fmt(p.d_temp_range[0], 2)}`, b: `${p.d_temp_range[1] > 0 ? '+' : ''}${fmt(p.d_temp_range[1], 2)} °C` }) : null);
const projBase = (p: Proj) => p.baseline_period ?? null;

/** Live Leaflet map for the dashboard with the layer switch; returns a cleanup. */
export async function mountDashboardMap(root: HTMLElement, a: ClientAnalysis): Promise<() => void> {
  const host = root.querySelector<HTMLElement>('[data-dmap]');
  if (!host) return () => {};
  const { loadLeaflet, eoxLayer, addSnapshot } = await import('../map/leaflet');
  const L = await loadLeaflet();
  const f = a.field;
  const map = L.map(host, { zoomControl: false, attributionControl: true, scrollWheelZoom: false }).setView([f.center.lat, f.center.lon], 16);
  L.control.zoom({ position: 'topright' }).addTo(map);
  map.attributionControl.setPrefix('Leaflet (BSD-2)');
  eoxLayer(L).addTo(map);
  setMapBase(t('db_map_base_old'));
  void addSnapshot(L, map, f.center.lat, f.center.lon, 5000).then((m) => {
    if (!m?.date) return;
    setMapBase(t('db_map_base', { d: m.date }));
  });
  const ringLL = f.ring.map(([lon, lat]) => [lat, lon] as [number, number]);
  const fieldStyle = { color: '#fff', weight: 2, fillOpacity: 0.08, fillColor: '#fff' };
  const outline = ringLL.length >= 3 ? L.polygon(ringLL, fieldStyle).addTo(map) : L.circleMarker([f.center.lat, f.center.lon], { radius: 8, color: '#fff', weight: 2, fillOpacity: 0.2 }).addTo(map);
  const fieldBounds = ringLL.length >= 3 ? (outline as Leaflet.Polygon).getBounds().pad(1.2) : L.latLng(f.center.lat, f.center.lon).toBounds(500);
  let overlay: Leaflet.Layer | null = null;
  const ev = a.evidence;

  const cell = (km: number) => {
    const dLat = km / 2 / 110.574, dLon = km / 2 / (111.32 * Math.cos((f.center.lat * Math.PI) / 180));
    return L.latLngBounds([f.center.lat - dLat, f.center.lon - dLon], [f.center.lat + dLat, f.center.lon + dLon]);
  };
  const setLayer = (k: LayerKey) => {
    overlay?.remove();
    overlay = null;
    root.querySelectorAll<HTMLButtonElement>('[data-layer]').forEach((b) => b.setAttribute('aria-checked', String(b.dataset.layer === k)));
    const legend = root.querySelector<HTMLElement>('[data-legend]');
    if (legend) legend.innerHTML = layerLegend(ev, k);
    const r = ev?.readings.find((x) => x.id.startsWith(LAYERS.find((l) => l.key === k)!.reading));
    if (!r) { if (outline instanceof L.Polygon) outline.setStyle(fieldStyle); map.fitBounds(fieldBounds); return; }
    if (r.badge === 'field-level') {
      const ndvi = k === 'vigor' && r.value != null ? Math.max(0, Math.min(1, r.value)) : null;
      const fill = ndvi == null ? '#ffffff' : `hsl(${Math.round(30 + ndvi * 90)}, 32%, ${Math.round(48 + ndvi * 8)}%)`;
      if (outline instanceof L.Polygon) outline.setStyle({ ...fieldStyle, weight: 1.5, fillColor: fill, fillOpacity: 0.25 });
      map.fitBounds(fieldBounds);
    } else {
      if (outline instanceof L.Polygon) outline.setStyle(fieldStyle);
      const km = resKm(r.resolution) ?? 9;
      const b = cell(km);
      overlay = L.rectangle(b, { color: '#fff', weight: 1.5, dashArray: '6 6', fillColor: '#cfe3ef', fillOpacity: 0.12, interactive: false }).addTo(map);
      map.fitBounds(b.pad(0.15));
    }
  };
  root.querySelectorAll<HTMLButtonElement>('[data-layer]').forEach((b) => b.addEventListener('click', () => setLayer(b.dataset.layer as LayerKey)));
  root.querySelector('[role=radiogroup]')?.addEventListener('keydown', (e) => {
    const ke = e as KeyboardEvent;
    if (!['ArrowRight', 'ArrowLeft', 'ArrowDown', 'ArrowUp'].includes(ke.key)) return;
    const btns = [...root.querySelectorAll<HTMLButtonElement>('[data-layer]')];
    const i = btns.findIndex((b) => b.getAttribute('aria-checked') === 'true');
    const n = (i + (ke.key === 'ArrowRight' || ke.key === 'ArrowDown' ? 1 : -1) + btns.length) % btns.length;
    btns[n].focus(); btns[n].click(); ke.preventDefault();
  });
  const ret = root.querySelector<HTMLElement>('[data-dret]');
  const placeRet = () => {
    if (!ret) return;
    const b = ringLL.length >= 3 ? (outline as Leaflet.Polygon).getBounds() : L.latLng(f.center.lat, f.center.lon).toBounds(120);
    const nw = map.latLngToContainerPoint(b.getNorthWest()), se = map.latLngToContainerPoint(b.getSouthEast());
    const pad = 10, w = Math.max(30, se.x - nw.x + pad * 2), h = Math.max(26, se.y - nw.y + pad * 2);
    ret.hidden = w > host.clientWidth * 0.9;
    Object.assign(ret.style, { left: host.offsetLeft + nw.x - pad + 'px', top: host.offsetTop + nw.y - pad + 'px', width: w + 'px', height: h + 'px' });
  };
  map.on('move zoom resize', placeRet);
  // Stop observing before the map is removed; a resize after removal would touch a map without panes.
  let alive = true;
  const ro = new ResizeObserver(() => { if (!alive || !host.isConnected) return; map.invalidateSize(); placeRet(); });
  ro.observe(host);
  requestAnimationFrame(() => { if (!alive) return; map.invalidateSize(); setLayer('vigor'); placeRet(); });
  return () => { alive = false; ro.disconnect(); map.remove(); };
}
