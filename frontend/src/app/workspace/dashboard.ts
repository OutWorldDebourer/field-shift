// Dashboard tab (App 4): field map with outline and an honest-scale layer switch, metric tiles from
// evidence.readings, "This season vs 40-year normal", NDVI series, 12-month crop calendar, alerts, projection.
// `dashboardHtml` is pure markup (also used by the index preview); `mountDashboardMap` adds the live Leaflet map.
import type * as Leaflet from 'leaflet';
import type { ClientAnalysis } from '../../shared/api';
import type { EvidencePackage, Reading } from '../../shared/contract';
import { ndviChart, seasonChart, timelineStrip } from '../../shared/charts';
import { normalPeriod } from '../../shared/evidence';
import { dataRow, sourceLine, valueText, badge, shortRes, resKm } from '../../shared/datarow';
import { esc } from '../../shared/dom';
import { fmt, fmtDateShort, pick, t } from '../../shared/i18n';
import { icon } from '../../shared/icons';
import type { StrKey } from '../../shared/strings';
import { alertsFor } from './alerts';
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

export function layerLegend(ev: EvidencePackage | null, key: LayerKey): string {
  const r = ev?.readings.find((x) => x.id === LAYERS.find((l) => l.key === key)!.reading);
  if (!r) return `<p class="dmap__note">${esc(t(ev ? 'ly_na' : 'db_no_evidence'))}</p>`;
  const v = valueText(r);
  const note = r.badge === 'field-level' ? t('db_layer_field', { res: shortRes(r.resolution) }) : t('db_layer_cell', { res: shortRes(r.resolution) });
  return `<p class="dmap__val"><b>${esc(pick(r.label))}</b> <span class="${r.key ? 'amber' : ''}">${esc(v.text)}</span></p>
    <p class="dmap__src mono">${sourceLine(r)}</p>
    <p class="dmap__note">${badge(r.badge)} ${esc(note)}</p>${ev ? `<p class="dmap__scale mono">${esc(scaleNote(ev))}</p>` : ''}`;
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
  const tiles = tilesFor(ev, 6).map((r) => `<li class="tile-m glass">${dataRow(r)}</li>`).join('');
  const alerts = alertsFor(ev);
  const last = ev.climate_monthly.length ? ev.climate_monthly[ev.climate_monthly.length - 1].month : null;
  const tl = currentTimeline(a);
  const startMonth = new Date(ev.generated_at).getUTCMonth() + 1;
  const proj = ev.projection;
  const pests = ev.pests.filter((p) => p.generations_now != null);
  return `<div class="dash${opts.preview ? ' dash--preview' : ''}">
    ${map}
    <div class="dash__side">
      <ul class="dash__tiles" aria-label="${esc(t('db_readings'))}">${tiles}</ul>
      <div class="dash__row dash__row--3">
        ${tl ? `<section class="glass dash__card dash__cal"><h3 class="dash__h">${esc(t(tl.length === 12 ? 'db_calendar' : 'db_sequence'))}</h3>${timelineStrip(tl, startMonth % 12, true, !!opts.preview)}${currentPlan(a) ? `<p class="dash__plan">${esc(pick(currentPlan(a)!.name))}</p><p class="dash__plan-why">${esc(pick(currentPlan(a)!.why))}</p>` : ''}${opts.preview || a.readOnly ? '' : `<button type="button" class="btn btn--glass btn--sm" data-goto="rotation">${icon('rotation')}${esc(t('ws_open_rotation'))}</button>`}</section>` : ''}
        <section class="glass dash__card"><h3 class="dash__h">${esc(t('db_alerts'))}</h3>
          ${alerts.length ? `<ul class="alerts">${alerts.map((al) => `<li class="al al--${al.level}">${icon(al.level === 'info' ? 'info' : 'alert')}<span>${esc(pick(al.text))}<small class="mono">${al.detail ? esc(al.detail) + ' · ' : ''}${esc(al.source)} · ${esc(fmtDateShort(al.date))}</small></span></li>`).join('')}</ul>` : `<p class="chart-empty">${esc(t('db_no_alerts'))}</p>`}
        </section>
        ${opts.preview ? '' : `<section class="glass dash__card dash__proj"><h3 class="dash__h">${esc(t('db_projection'))}</h3>
          ${proj && proj.d_temp_c != null ? `<p class="proj"><span class="proj__v">+${fmt(proj.d_temp_c, 1)} °C</span> <span>${esc(proj.horizon)} · ${esc(proj.scenario)}</span></p>
            ${proj.d_rain_pct != null ? `<p class="proj__r">${proj.d_rain_pct > 0 ? '+' : '−'}${Math.abs(proj.d_rain_pct)}% ${esc(t('db_rain'))}</p>` : ''}
            <p class="dash__src mono">NEX-GDDP-CMIP6 · ${esc(proj.models.join(', '))} · ~25 km · ${esc(t('regional'))}</p>` : `<p class="chart-empty">${esc(t('db_projection_none'))}</p>`}
        </section>`}
      </div>
      <section class="glass dash__card dash__chart">
        <header class="dash__ch"><h3 class="serif">${esc(t('db_chart_title'))}</h3>
          <p class="legend"><span><i class="k-line"></i>${esc(t('db_this_year'))}</span><span><i class="k-band"></i>${esc(t('db_normal'))}${normalPeriod(ev) ? ' ' + esc(normalPeriod(ev)!) : ''}</span></p></header>
        ${seasonChart(ev.climate_monthly, { w: 680, h: opts.preview ? 180 : 200 })}
        <p class="dash__src mono">NASA POWER · T2M · ${last ? esc(last) : ''} · ~50 km · ${esc(t('regional'))}</p>
      </section>
      ${opts.preview ? '' : `<div class="dash__row">
        <section class="glass dash__card"><h3 class="dash__h">${esc(t('db_ndvi_title'))}</h3>${ndviChart(ev.ndvi_series, { w: 420, h: 150 })}
          <p class="dash__src mono">${esc([...new Set(ev.ndvi_series.map((n) => n.sensor))].join(' · '))} · 10–30 m · ${esc(t('field_level'))}</p></section>
        <section class="glass dash__card"><h3 class="dash__h">${esc(t('db_pests'))}</h3>
          ${pests.length ? `<ul class="pests">${pests.map((p) => `<li><span><b>${esc(p.name)}</b> <i>${esc(p.scientific)}</i></span>
            <span class="mono">${fmt(p.generations_now!, 1)} ${esc(t('db_now'))} · ${p.generations_normal != null ? fmt(p.generations_normal, 1) + ' ' + esc(t('db_norm')) : '—'}</span></li>`).join('')}</ul>
            <p class="dash__src mono" title="${esc(pests.map((p) => p.citation).join(' | '))}">NASA POWER · degree-days · ${esc(t('regional'))}</p>` : `<p class="chart-empty">${esc(t('db_no_evidence'))}</p>`}
        </section></div>`}
    </div>
  </div>`;
}

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
  void addSnapshot(L, map, f.center.lat, f.center.lon, 1500);
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
    Object.assign(ret.style, { left: nw.x - pad + 'px', top: nw.y - pad + 'px', width: w + 'px', height: h + 'px' });
  };
  map.on('move zoom resize', placeRet);
  requestAnimationFrame(() => { map.invalidateSize(); setLayer('vigor'); placeRet(); });
  return () => map.remove();
}
