// Field selection (App 1): My GPS | Example site, place search, drop a point or draw an outline with live area,
// on a licence-clean satellite map (Leaflet + EOX Sentinel-2 cloudless 2016). "Use this field" → POST /api/analyses.
import type * as Leaflet from 'leaflet';
import { createAnalysis, examples, type ClientAnalysis, type Example } from '../../shared/api';
import type { FieldGeom } from '../../shared/contract';
import { $, esc, toast } from '../../shared/dom';
import { ringAreaHa, ringCenter, fmtCoord } from '../../shared/geo';
import { applyI18n, fmt, lang, onLang, pick, t } from '../../shared/i18n';
import { icon } from '../../shared/icons';
import { addSnapshot, eoxLayer, loadLeaflet } from '../map/leaflet';
import { debounce, searchPlaces, type Place } from './search';
import './field.css';

type Mode = 'example' | 'gps';
type Tool = 'none' | 'point' | 'draw';

const KEY = 'fieldshift.lastField';

export function mountFieldSelection(host: HTMLElement, opts: { site?: string | null; go?: boolean; onDone: (a: ClientAnalysis) => void }): () => void {
  let mode: Mode = opts.site ? 'example' : 'gps';
  let tool: Tool = 'none';
  let exampleId: string | null = opts.site ?? null;
  let exampleList: Example[] = [];
  let point: { lat: number; lon: number } | null = null;
  let ring: [number, number][] = [];
  let draft: [number, number][] = [];
  let label = '';
  let map: Leaflet.Map | null = null;
  let L: typeof Leaflet | null = null;
  let layer: Leaflet.LayerGroup | null = null;

  host.innerHTML = `<div class="fs">
    <div class="fs__map" id="fs-map" role="region" aria-label="${esc(t('fs_map_label'))}" data-i18n-attr="aria-label:fs_map_label"><p class="fs__loading mono">${esc(t('fs_map_loading'))}</p></div>
    <div class="reticle fs__reticle" id="fs-reticle" hidden><i></i></div>
    <div class="fs__area-tag ground-label" id="fs-area-tag" hidden></div>
    <aside class="fs__panel glass" aria-labelledby="fs-t">
      <h1 id="fs-t" class="serif fs__title" data-i18n="fs_title">${esc(t('fs_title'))}</h1>
      <p class="fs__lede" data-i18n="fs_lede">${esc(t('fs_lede'))}</p>
      <div class="seg" role="radiogroup" aria-label="${esc(t('fs_mode'))}" data-i18n-attr="aria-label:fs_mode">
        <button type="button" role="radio" data-mode="gps">${icon('gps')}<span data-i18n="fs_gps">${esc(t('fs_gps'))}</span></button>
        <button type="button" role="radio" data-mode="example">${icon('pin')}<span data-i18n="fs_example">${esc(t('fs_example'))}</span></button>
      </div>
      <div class="fs__pane" data-pane="example">
        <ul class="fs__examples" id="fs-examples" aria-label="${esc(t('fs_examples'))}"></ul>
      </div>
      <div class="fs__pane" data-pane="gps">
        <button type="button" class="btn btn--glass btn--sm" id="fs-locate">${icon('gps')}<span data-i18n="fs_gps_go">${esc(t('fs_gps_go'))}</span></button>
        <p class="fs__status" id="fs-gps-status" aria-live="polite"></p>
      </div>
      <div class="fs__search" role="search">
        <label for="fs-q" class="fs__label" data-i18n="fs_search">${esc(t('fs_search'))}</label>
        <div class="fs__qwrap">${icon('search')}<input id="fs-q" type="search" autocomplete="off" role="combobox" aria-expanded="false" aria-controls="fs-results" aria-autocomplete="list"
          placeholder="${esc(t('fs_search_ph'))}" data-i18n-attr="placeholder:fs_search_ph"></div>
        <ul class="fs__results" id="fs-results" role="listbox" hidden></ul>
        <p class="fs__osm mono" data-i18n="fs_osm">${esc(t('fs_osm'))}</p>
      </div>
      <div class="fs__tools" role="toolbar" aria-label="${esc(t('fs_draw'))}">
        <button type="button" class="tool" data-tool="point" aria-pressed="false">${icon('point')}<span data-i18n="fs_point">${esc(t('fs_point'))}</span></button>
        <button type="button" class="tool" data-tool="draw" aria-pressed="false">${icon('polygon')}<span data-i18n="fs_draw">${esc(t('fs_draw'))}</span></button>
        <button type="button" class="tool tool--quiet" id="fs-clear">${icon('trash')}<span data-i18n="fs_clear">${esc(t('fs_clear'))}</span></button>
      </div>
      <button type="button" class="btn btn--glass btn--sm fs__finish" id="fs-finish" hidden>${icon('polygon')}<span data-i18n="fs_finish">${esc(t('fs_finish'))}</span></button>
      <p class="fs__hint" id="fs-hint" aria-live="polite"></p>
      <div class="fs__readout" id="fs-readout" aria-live="polite"></div>
      <button type="button" class="btn btn--solid fs__use" id="fs-use" disabled>${esc(t('fs_use'))}${icon('arrow')}</button>
      <div class="fs__error" id="fs-error" role="alert" hidden></div>
    </aside>
  </div>`;
  applyI18n(host);

  const examplesEl = $('#fs-examples', host), hint = $('#fs-hint', host), readout = $('#fs-readout', host);
  const useBtn = $<HTMLButtonElement>('#fs-use', host), finishBtn = $<HTMLButtonElement>('#fs-finish', host);
  const reticle = $('#fs-reticle', host), areaTag = $('#fs-area-tag', host);

  // ---------- render helpers ----------
  function renderMode() {
    host.querySelectorAll<HTMLButtonElement>('[data-mode]').forEach((b) => b.setAttribute('aria-checked', String(b.dataset.mode === mode)));
    host.querySelectorAll<HTMLElement>('[data-pane]').forEach((p) => { p.hidden = p.dataset.pane !== mode; });
  }
  function renderExamples() {
    examplesEl.innerHTML = exampleList.map((e) => `<li><button type="button" class="ex" data-ex="${esc(e.id)}" aria-pressed="${e.id === exampleId}">
        <span class="ex__name">${esc(pick(e.name))}</span><span class="ex__meta mono">${esc(t('fs_huaral_meta', { ha: fmt(ringAreaHa(e.field.ring) || e.field.area_ha, 1) }))}</span></button></li>`).join('')
      + `<li class="ex__soon mono">${esc(t('fs_coming'))}</li>`;
    examplesEl.querySelectorAll<HTMLButtonElement>('[data-ex]').forEach((b) => b.addEventListener('click', () => chooseExample(b.dataset.ex!)));
  }
  function renderTools() {
    host.querySelectorAll<HTMLButtonElement>('[data-tool]').forEach((b) => b.setAttribute('aria-pressed', String(b.dataset.tool === tool)));
    finishBtn.hidden = tool !== 'draw';
    finishBtn.disabled = draft.length < 3;
    hint.textContent = tool === 'point' ? t('fs_hint_point') : tool === 'draw' ? (draft.length && draft.length < 3 ? t('fs_hint_need3') : t('fs_hint_draw')) : '';
    map?.getContainer().classList.toggle('is-picking', tool !== 'none');
  }
  function currentRing(): [number, number][] { return tool === 'draw' && draft.length >= 3 ? draft : ring; }
  function renderReadout() {
    const r = currentRing();
    const area = ringAreaHa(r);
    const c = r.length >= 3 ? ringCenter(r) : point;
    if (!c) { readout.innerHTML = `<span class="fs__none">${esc(t('fs_none'))}</span>`; useBtn.disabled = true; areaTag.hidden = true; return; }
    readout.innerHTML = r.length >= 3
      ? `<span class="mono fs__k">${esc(t('fs_area'))}</span><b class="fs__area num">${fmt(area, area < 10 ? 1 : 0)} ha</b><span class="mono fs__c">${fmtCoord(c.lat, c.lon)}</span>`
      : `<span class="mono fs__k">${esc(t('fs_point_only'))}</span><span class="mono fs__c">${fmtCoord(c.lat, c.lon)}</span>`;
    useBtn.disabled = !(ring.length >= 3 || (point && tool !== 'draw'));
    if (r.length >= 3) { areaTag.hidden = false; areaTag.innerHTML = `<b>${fmt(area, area < 10 ? 1 : 0)} ha</b>`; } else areaTag.hidden = true;
    placeReticle();
  }
  function placeReticle() {
    if (!map || !L) return;
    const r = currentRing();
    let b: Leaflet.LatLngBounds | null = null;
    if (r.length >= 3) b = L.latLngBounds(r.map(([lo, la]) => [la, lo] as [number, number]));
    else if (point) b = L.latLng(point.lat, point.lon).toBounds(120);
    if (!b) { reticle.hidden = true; areaTag.hidden = true; return; }
    const mapRect = map.getContainer().getBoundingClientRect(), hostRect = host.getBoundingClientRect();
    const nw = map.latLngToContainerPoint(b.getNorthWest()), se = map.latLngToContainerPoint(b.getSouthEast());
    const pad = 14;
    const x = mapRect.left - hostRect.left + nw.x - pad, y = mapRect.top - hostRect.top + nw.y - pad;
    const w = se.x - nw.x + pad * 2, h = se.y - nw.y + pad * 2;
    reticle.hidden = false;
    Object.assign(reticle.style, { left: x + 'px', top: y + 'px', width: Math.max(44, w) + 'px', height: Math.max(40, h) + 'px' });
    areaTag.style.transform = `translate(${x + Math.max(44, w) + 10}px, ${y - 8}px)`;
  }
  function lockOn() { reticle.classList.remove('is-locking'); void reticle.offsetWidth; reticle.classList.add('is-locking'); }

  function drawLayers() {
    if (!map || !L || !layer) return;
    layer.clearLayers();
    const Lx = L;
    const ll = (p: [number, number]) => [p[1], p[0]] as [number, number];
    if (ring.length >= 3 && tool !== 'draw') Lx.polygon(ring.map(ll), { color: '#fff', weight: 2, fillOpacity: 0.1, fillColor: '#fff' }).addTo(layer);
    if (tool === 'draw' && draft.length) {
      Lx.polyline(draft.map(ll), { color: '#fff', weight: 2, dashArray: draft.length >= 3 ? undefined : '5 5' }).addTo(layer);
      if (draft.length >= 3) Lx.polygon(draft.map(ll), { color: '#fff', weight: 0, fillOpacity: 0.12 }).addTo(layer);
      draft.forEach((p, i) => Lx.circleMarker(ll(p), { radius: i === 0 ? 7 : 5, color: '#fff', weight: 2, fillColor: i === 0 ? '#FFBE55' : '#0b1a28', fillOpacity: 1 })
        .on('click', (e) => { if (i === 0 && draft.length >= 3) { Lx.DomEvent.stop(e); finishDraw(); } }).addTo(layer!));
    }
    if (point && ring.length < 3 && tool !== 'draw') Lx.circleMarker([point.lat, point.lon], { radius: 8, color: '#fff', weight: 2.5, fillColor: '#FFBE55', fillOpacity: 1 }).addTo(layer);
    renderReadout();
  }

  // ---------- actions ----------
  function chooseExample(id: string) {
    const ex = exampleList.find((e) => e.id === id);
    if (!ex) return;
    mode = 'example'; exampleId = id; tool = 'none'; draft = [];
    ring = ex.field.ring.slice(); point = { lat: ex.field.center.lat, lon: ex.field.center.lon }; label = ex.field.label || pick(ex.name);
    renderMode(); renderExamples(); renderTools(); drawLayers();
    if (map && L && ring.length >= 3) map.fitBounds(L.latLngBounds(ring.map(([lo, la]) => [la, lo] as [number, number])).pad(1.4), { animate: false });
    lockOn();
  }
  function setTool(tl: Tool) {
    tool = tool === tl ? 'none' : tl;
    if (tool === 'draw') { draft = []; map?.doubleClickZoom.disable(); } else map?.doubleClickZoom.enable();
    if (tool !== 'none') exampleId = null;
    renderExamples(); renderTools(); drawLayers();
  }
  function finishDraw() {
    if (draft.length < 3) { hint.textContent = t('fs_hint_need3'); return; }
    ring = draft.slice(); point = ringCenter(ring); draft = []; tool = 'none'; label = label || t('fs_my_field');
    map?.doubleClickZoom.enable();
    renderTools(); drawLayers(); lockOn();
  }
  function clearAll() { ring = []; draft = []; point = null; exampleId = null; tool = 'none'; renderExamples(); renderTools(); drawLayers(); }

  function locate() {
    const st = $('#fs-gps-status', host);
    if (!('geolocation' in navigator)) { st.textContent = t('fs_gps_denied'); return; }
    st.textContent = t('fs_gps_locating');
    navigator.geolocation.getCurrentPosition((pos) => {
      const { latitude: lat, longitude: lon, accuracy } = pos.coords;
      point = { lat, lon }; ring = []; exampleId = null; label = t('fs_my_field');
      st.textContent = t('fs_gps_ok', { m: Math.round(accuracy) });
      map?.setView([lat, lon], 17);
      renderExamples(); drawLayers(); lockOn();
    }, () => { st.textContent = t('fs_gps_denied'); }, { enableHighAccuracy: true, timeout: 15000, maximumAge: 60000 });
  }

  async function use() {
    const err = $('#fs-error', host);
    err.hidden = true;
    const r = ring.length >= 3 ? ring : [];
    const c = r.length >= 3 ? ringCenter(r) : point;
    if (!c) return;
    const field: FieldGeom = { center: { lat: +c.lat.toFixed(6), lon: +c.lon.toFixed(6) }, ring: r.map(([lo, la]) => [+lo.toFixed(6), +la.toFixed(6)]) };
    try { localStorage.setItem(KEY, JSON.stringify({ field, label })); } catch { /* ignore */ }
    useBtn.disabled = true;
    useBtn.innerHTML = `<span class="status-line"><i class="tick"></i>${esc(t('fs_creating'))}</span>`;
    try {
      const a = await createAnalysis({ field, location_mode: exampleId ? 'example' : mode === 'gps' && !r.length ? 'gps' : 'map', example_site: exampleId, lang: lang(), label: label || undefined });
      opts.onDone(a);
    } catch {
      err.hidden = false;
      err.innerHTML = `<p>${esc(t('ap_offline_custom'))}</p><div class="fs__err-actions"><button type="button" class="btn btn--glass btn--sm" data-retry>${esc(t('ap_retry'))}</button>
        <button type="button" class="btn btn--solid btn--sm" data-demo>${esc(t('ap_open_example'))}</button></div>`;
      err.querySelector('[data-retry]')!.addEventListener('click', use);
      err.querySelector('[data-demo]')!.addEventListener('click', () => { chooseExample('huaral'); void use(); });
    } finally {
      useBtn.innerHTML = `${esc(t('fs_use'))}${icon('arrow')}`;
      renderReadout();
    }
  }

  // ---------- search ----------
  const q = $<HTMLInputElement>('#fs-q', host), results = $('#fs-results', host);
  let places: Place[] = [];
  let active = -1;
  const showResults = (open: boolean) => { results.hidden = !open; q.setAttribute('aria-expanded', String(open)); };
  const renderResults = () => {
    results.innerHTML = places.length ? places.map((p, i) => `<li role="option" id="fs-r${i}" aria-selected="${i === active}" data-i="${i}">${esc(p.name)}</li>`).join('') : `<li class="fs__none">${esc(t('fs_search_none'))}</li>`;
    q.setAttribute('aria-activedescendant', active >= 0 ? `fs-r${active}` : '');
    showResults(true);
  };
  const goPlace = (p: Place) => {
    showResults(false); q.value = p.name.split(',')[0];
    if (map && L) { if (p.bbox) map.fitBounds([[p.bbox[1], p.bbox[0]], [p.bbox[3], p.bbox[2]]], { maxZoom: 16 }); else map.setView([p.lat, p.lon], 15); }
    label = p.name.split(',').slice(0, 2).join(',');
    if (!ring.length && tool === 'none') { hint.textContent = t('fs_hint_draw'); }
  };
  const run = debounce(async (v: string) => {
    if (v.trim().length < 3) { showResults(false); return; }
    try { places = await searchPlaces(v); active = -1; renderResults(); }
    catch (e) { if ((e as Error).name !== 'AbortError') { results.innerHTML = `<li class="fs__none">${esc(t('fs_search_err'))}</li>`; showResults(true); } }
  }, 600);
  q.addEventListener('input', () => run(q.value));
  q.addEventListener('keydown', (e) => {
    if (results.hidden || !places.length) return;
    if (e.key === 'ArrowDown') { active = (active + 1) % places.length; renderResults(); e.preventDefault(); }
    else if (e.key === 'ArrowUp') { active = (active - 1 + places.length) % places.length; renderResults(); e.preventDefault(); }
    else if (e.key === 'Enter' && active >= 0) { goPlace(places[active]); e.preventDefault(); }
    else if (e.key === 'Escape') showResults(false);
  });
  results.addEventListener('click', (e) => { const li = (e.target as HTMLElement).closest<HTMLElement>('[data-i]'); if (li) goPlace(places[+li.dataset.i!]); });

  // ---------- wiring ----------
  host.querySelectorAll<HTMLButtonElement>('[data-mode]').forEach((b) => b.addEventListener('click', () => {
    mode = b.dataset.mode as Mode; renderMode();
    if (mode === 'example' && !exampleId && exampleList[0]) chooseExample(exampleList[0].id);
    if (mode === 'gps') { exampleId = null; renderExamples(); locate(); }
  }));
  host.querySelectorAll<HTMLButtonElement>('[data-tool]').forEach((b) => b.addEventListener('click', () => setTool(b.dataset.tool as Tool)));
  $('#fs-clear', host).addEventListener('click', clearAll);
  $('#fs-locate', host).addEventListener('click', locate);
  finishBtn.addEventListener('click', finishDraw);
  useBtn.addEventListener('click', use);
  const offLang = onLang(() => { renderExamples(); renderTools(); renderReadout(); useBtn.innerHTML = `${esc(t('fs_use'))}${icon('arrow')}`; });

  renderMode(); renderTools(); renderReadout();

  // ---------- map (lazy) ----------
  void (async () => {
    L = await loadLeaflet();
    const el = $('#fs-map', host);
    el.innerHTML = '';
    map = L.map(el, { zoomControl: false, attributionControl: true, worldCopyJump: true }).setView([-11.47, -77.21], 13);
    L.control.zoom({ position: 'topright' }).addTo(map);
    map.attributionControl.setPrefix('<a href="https://leafletjs.com" target="_blank" rel="noopener">Leaflet</a> (BSD-2)');
    eoxLayer(L).addTo(map);
    L.control.scale({ metric: true, imperial: false, position: 'bottomleft' }).addTo(map);
    layer = L.layerGroup().addTo(map);
    map.on('click', (e: Leaflet.LeafletMouseEvent) => {
      const { lat, lng } = e.latlng;
      if (tool === 'point') { point = { lat, lon: lng }; ring = []; tool = 'none'; label = label || t('fs_my_field'); renderTools(); drawLayers(); lockOn(); }
      else if (tool === 'draw') { draft.push([lng, lat]); renderTools(); drawLayers(); }
    });
    map.on('dblclick', () => { if (tool === 'draw') finishDraw(); });
    map.on('move zoom resize', placeReticle);
    const ex = await examples();
    exampleList = ex.list;
    renderExamples();
    if (exampleId) {
      chooseExample(exampleId);
      const h = exampleList.find((e) => e.id === exampleId);
      if (h) void addSnapshot(L, map, h.field.center.lat, h.field.center.lon, 1500);
      if (opts.go) void use();
    } else {
      try {
        const saved = JSON.parse(localStorage.getItem(KEY) || 'null') as { field: FieldGeom; label: string } | null;
        if (saved?.field) { ring = saved.field.ring; point = saved.field.center; label = saved.label; drawLayers(); map.setView([point.lat, point.lon], 16); }
      } catch { /* ignore */ }
    }
  })().catch(() => { $('#fs-map', host).innerHTML = `<p class="fs__loading mono">${esc(t('fs_search_err'))}</p>`; toast(t('fs_search_err')); });

  return () => { offLang(); map?.remove(); };
}
