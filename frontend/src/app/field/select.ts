// Field selection (App 1): My GPS | Example site, place search, drop a point or draw an outline with live area,
// on a licence-clean satellite map (Leaflet + EOX Sentinel-2 cloudless 2016). "Use this field" → POST /api/analyses.
import type * as Leaflet from 'leaflet';
import { createAnalysis, examples, type ClientAnalysis, type Example } from '../../shared/api';
import type { FieldGeom } from '../../shared/contract';
import { $, esc, toast } from '../../shared/dom';
import { ringAreaHa, ringCenter, fmtCoord } from '../../shared/geo';
import { applyI18n, fmt, fmtDate, lang, onLang, pick, t } from '../../shared/i18n';
import { icon } from '../../shared/icons';
import { addSnapshot, eoxLayer, loadLeaflet } from '../map/leaflet';
import { debounce, searchPlaces, type Place } from './search';
import './field.css';

type Mode = 'example' | 'gps';
type Tool = 'none' | 'point' | 'draw';

const KEY = 'fieldshift.lastField';

export function mountFieldSelection(host: HTMLElement, opts: { site?: string | null; go?: boolean; onDone: (a: ClientAnalysis) => void }): () => void {
  let mode: Mode = 'example';
  let snapDate: string | null = null;
  let snapBox: [number, number, number, number] | null = null;
  /** The dated snapshot's date, only when it actually covers the field being shown. */
  const coveredDate = (): string | null => {
    const r = currentRing(); const pts = r.length >= 3 ? r : point ? [[point.lon, point.lat] as [number, number]] : [];
    if (!snapDate || !snapBox || !pts.length) return null;
    return pts.every(([lo, la]) => lo >= snapBox![0] && lo <= snapBox![2] && la >= snapBox![1] && la <= snapBox![3]) ? snapDate : null;
  };
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
    <div class="fs__alt instr" id="fs-alt" aria-hidden="true"></div>
    <button type="button" class="fs__full pill" id="fs-full" aria-pressed="false">${icon('layers')}<span>${esc(t('fs_full'))}</span></button>
    <div class="fs__credit"><span class="mono" id="fs-img"></span><button type="button" class="fs__info" id="fs-info" aria-expanded="false" aria-controls="fs-lic" aria-label="${esc(t('fs_licences'))}">${icon('info')}</button>
      <div class="fs__lic glass" id="fs-lic" hidden><p>Sentinel-2 cloudless 2016 by EOX IT Services GmbH (Contains modified Copernicus Sentinel data 2016) · CC BY 4.0</p><p>Sentinel-2 L2A · Contains modified Copernicus Sentinel data 2026, processed by Microsoft Planetary Computer</p><p>Place search: Nominatim · © OpenStreetMap contributors · ODbL</p><p>Leaflet · BSD-2-Clause</p></div></div>
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
        <p class="fs__status" id="fs-gps-status" aria-live="polite"></p>
      </div>
      <div class="fs__search" role="search">
        <label for="fs-q" class="fs__label" data-i18n="fs_search">${esc(t('fs_search'))}</label>
        <div class="fs__qwrap">${icon('search')}<input id="fs-q" type="search" autocomplete="off" role="combobox" aria-expanded="false" aria-controls="fs-results" aria-autocomplete="list"
          placeholder="${esc(t('fs_search_ph'))}" data-i18n-attr="placeholder:fs_search_ph"></div>
        <ul class="fs__results" id="fs-results" role="listbox" hidden></ul>
      </div>
      <div class="fs__tools" role="toolbar" aria-label="${esc(t('fs_draw'))}">
        <button type="button" class="tool" data-tool="point" aria-pressed="false">${icon('point')}<span data-i18n="fs_point">${esc(t('fs_point'))}</span></button>
        <button type="button" class="tool" data-tool="draw" aria-pressed="false">${icon('polygon')}<span data-i18n="fs_draw">${esc(t('fs_draw'))}</span></button>
        <button type="button" class="tool tool--quiet" id="fs-clear">${icon('trash')}<span data-i18n="fs_clear">${esc(t('fs_clear'))}</span></button>
      </div>
      <button type="button" class="btn btn--glass btn--sm fs__finish" id="fs-finish" hidden>${icon('polygon')}<span data-i18n="fs_finish">${esc(t('fs_finish'))}</span></button>
      <p class="fs__hint" id="fs-hint" aria-live="polite"></p>
      <div class="fs__readout" id="fs-readout" aria-live="polite"></div>
      <p class="fs__why" id="fs-why" data-i18n="fs_why">${esc(t('fs_why'))}</p>
      <div class="fs__actions"><button type="button" class="btn btn--solid fs__use" id="fs-use" disabled>${esc(t('fs_use'))}${icon('arrow')}</button>
</div>
      <div class="fs__error" id="fs-error" role="alert" hidden></div>
    </aside>
  </div>`;
  applyI18n(host);

  const examplesEl = $('#fs-examples', host), hint = $('#fs-hint', host), readout = $('#fs-readout', host);
  const useBtn = $<HTMLButtonElement>('#fs-use', host), finishBtn = $<HTMLButtonElement>('#fs-finish', host);
  const reticle = $('#fs-reticle', host), areaTag = $('#fs-area-tag', host);

  // ---------- render helpers ----------
  function renderMode() {
    host.querySelectorAll<HTMLButtonElement>('[data-mode]').forEach((b) => { b.setAttribute('aria-checked', String(b.dataset.mode === mode)); b.tabIndex = b.dataset.mode === mode ? 0 : -1; });
    host.querySelectorAll<HTMLElement>('[data-pane]').forEach((p) => { p.hidden = p.dataset.pane !== mode; });
  }
  function renderExamples() {
    examplesEl.innerHTML = exampleList.map((e) => `<li><button type="button" class="ex" data-ex="${esc(e.id)}" aria-pressed="${e.id === exampleId}">
        <span class="ex__name">${esc(pick(e.name))}</span><span class="ex__meta mono">${esc(t('fs_huaral_meta', { ha: fmt(ringAreaHa(e.field.ring) || e.field.area_ha, 1) }))}</span></button></li>`).join('');
    examplesEl.querySelectorAll<HTMLButtonElement>('[data-ex]').forEach((b) => b.addEventListener('click', () => chooseExample(b.dataset.ex!)));
  }
  function renderTools() {
    host.querySelectorAll<HTMLButtonElement>('[data-tool]').forEach((b) => b.setAttribute('aria-pressed', String(b.dataset.tool === tool)));
    finishBtn.hidden = tool !== 'draw';
    $('#fs-clear', host).hidden = !(ring.length || draft.length || point);
    finishBtn.disabled = draft.length < 3;
    hint.textContent = tool === 'point' ? t('fs_hint_point') : tool === 'draw' ? (draft.length && draft.length < 3 ? t('fs_hint_need3') : t('fs_hint_draw')) : '';
    map?.getContainer().classList.toggle('is-picking', tool !== 'none');
  }
  function currentRing(): [number, number][] { return tool === 'draw' && draft.length >= 3 ? draft : ring; }
  function renderReadout() {
    const r = currentRing();
    const area = ringAreaHa(r);
    const c = r.length >= 3 ? ringCenter(r) : point;
    if (!c) { readout.innerHTML = `<span class="fs__none">${esc(t('fs_none'))}</span>`; useBtn.disabled = true; $('#fs-why', host).hidden = false; areaTag.hidden = true; return; }
    readout.innerHTML = r.length >= 3
      ? `<span class="mono fs__k">${esc(t('fs_area'))}</span><b class="fs__area num">${fmt(area, area < 10 ? 1 : 0)} ha</b><span class="mono fs__c">${fmtCoord(c.lat, c.lon)}</span>`
      : `<span class="mono fs__k">${esc(t('fs_point_only'))}</span><span class="mono fs__c">${fmtCoord(c.lat, c.lon)}</span>`;
    useBtn.disabled = !(ring.length >= 3 || (point && tool !== 'draw'));
    $('#fs-why', host).hidden = !useBtn.disabled;
    if (r.length >= 3) { areaTag.hidden = false; areaTag.innerHTML = `<b>${fmt(area, area < 10 ? 1 : 0)} ha</b><br><span>${esc(coveredDate() ? t('fs_img_date', { d: fmtDate(coveredDate()!) }) : t('fs_img_base'))}</span>`; } else areaTag.hidden = true;
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
    if (map && ring.length >= 3) { const c = ringCenter(ring); map.setView([c.lat, c.lon], innerWidth < 760 ? 15 : 15.5, { animate: false }); }
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
    const fail = () => { st.innerHTML = `${esc(t('fs_gps_denied'))} <button type="button" class="fs__retry">${esc(t('ap_retry'))}</button>`; st.querySelector('button')!.addEventListener('click', locate); };
    navigator.geolocation.getCurrentPosition((pos) => {
      const { latitude: lat, longitude: lon, accuracy } = pos.coords;
      point = { lat, lon }; ring = []; exampleId = null; label = t('fs_my_field');
      st.textContent = t('fs_gps_ok', { m: Math.round(accuracy) });
      map?.setView([lat, lon], 17);
      renderExamples(); drawLayers(); lockOn();
    }, fail, { enableHighAccuracy: true, timeout: 15000, maximumAge: 60000 });
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
  // Radiogroup keyboard: arrows move and select, one tab stop.
  host.querySelector('.seg')!.addEventListener('keydown', (e) => {
    const k = (e as KeyboardEvent).key;
    if (!['ArrowLeft', 'ArrowRight', 'ArrowUp', 'ArrowDown'].includes(k)) return;
    const bs = [...host.querySelectorAll<HTMLButtonElement>('[data-mode]')];
    const i = bs.findIndex((b) => b.dataset.mode === mode);
    const n = bs[(i + (k === 'ArrowRight' || k === 'ArrowDown' ? 1 : -1) + bs.length) % bs.length];
    n.focus(); n.click(); e.preventDefault();
  });
  host.querySelectorAll<HTMLButtonElement>('[data-mode]').forEach((b) => b.addEventListener('click', () => {
    mode = b.dataset.mode as Mode; renderMode();
    if (mode === 'example' && !exampleId && exampleList[0]) chooseExample(exampleList[0].id);
    if (mode === 'gps') { exampleId = null; renderExamples(); locate(); }
  }));
  host.querySelectorAll<HTMLButtonElement>('[data-tool]').forEach((b) => b.addEventListener('click', () => setTool(b.dataset.tool as Tool)));
  $('#fs-clear', host).addEventListener('click', clearAll);
  $('#fs-full', host).addEventListener('click', (e) => {
    const b = e.currentTarget as HTMLElement;
    const on = !host.classList.contains('is-full');
    host.classList.toggle('is-full', on); b.setAttribute('aria-pressed', String(on));
    b.querySelector('span')!.textContent = t(on ? 'fs_full_off' : 'fs_full');
    setTimeout(() => map?.invalidateSize(), 60);
  });
  $('#fs-info', host).addEventListener('click', (e) => {
    const b = e.currentTarget as HTMLElement; const lic = $('#fs-lic', host);
    lic.hidden = !lic.hidden; b.setAttribute('aria-expanded', String(!lic.hidden));
  });
  finishBtn.addEventListener('click', finishDraw);
  useBtn.addEventListener('click', use);
  const offLang = onLang(() => { renderExamples(); renderTools(); renderReadout(); useBtn.innerHTML = `${esc(t('fs_use'))}${icon('arrow')}`; });

  renderMode(); renderTools(); renderReadout();

  // Phone: the action bar is fixed to the viewport, so it must live outside the blurred panel (backdrop-filter
  // creates a containing block for fixed children).
  const actions = $('.fs__actions', host), panel = $('.fs__panel', host), fsEl = $('.fs', host);
  const mq = matchMedia('(max-width: 760px)');
  const placeActions = () => { if (mq.matches) fsEl.append(actions); else panel.insertBefore(actions, $('#fs-error', host)); };
  mq.addEventListener('change', placeActions);
  placeActions();

  // ---------- map (lazy) ----------
  void (async () => {
    L = await loadLeaflet();
    const el = $('#fs-map', host);
    el.innerHTML = '';
    map = L.map(el, { zoomControl: false, attributionControl: false, worldCopyJump: true, maxZoom: 16, zoomSnap: 0.5 }).setView([-11.47, -77.21], 13);
    L.control.zoom({ position: 'topright' }).addTo(map);
    eoxLayer(L).addTo(map);
    L.control.scale({ metric: true, imperial: false, position: 'bottomleft' }).addTo(map);
    layer = L.layerGroup().addTo(map);
    map.on('click', (e: Leaflet.LeafletMouseEvent) => {
      const { lat, lng } = e.latlng;
      if (tool === 'point') { point = { lat, lon: lng }; ring = []; tool = 'none'; label = label || t('fs_my_field'); renderTools(); drawLayers(); lockOn(); }
      else if (tool === 'draw') { draft.push([lng, lat]); renderTools(); drawLayers(); }
    });
    map.on('dblclick', () => { if (tool === 'draw') finishDraw(); });
    const instr = () => {
      if (!map) return;
      const c = map.getCenter(), z = map.getZoom(), H = map.getSize().y;
      const mpp = (156543.03 * Math.cos((c.lat * Math.PI) / 180)) / 2 ** z;
      const km = (mpp * H) / (2 * Math.tan(Math.PI / 6)) / 1000;
      $('#fs-alt', host).textContent = t('alt', { km: km >= 10 ? Math.round(km) : km.toFixed(1) });
      $('#fs-img', host).textContent = coveredDate() ? `Sentinel-2 L2A ${coveredDate()} · EOX 2016 base` : t('eox_short');
    };
    map.on('move zoom resize', () => { placeReticle(); instr(); });
    instr();
    const ex = await examples();
    exampleList = ex.list;
    renderExamples();
    let saved: { field: FieldGeom; label: string } | null = null;
    try { saved = JSON.parse(localStorage.getItem(KEY) || 'null'); } catch { /* ignore */ }
    if (!exampleId && !saved?.field && exampleList[0]) exampleId = exampleList[0].id;
    const h = exampleList.find((e) => e.id === (exampleId ?? 'huaral')) ?? exampleList[0];
    if (h) void addSnapshot(L, map, h.field.center.lat, h.field.center.lon, 5000).then((m) => { snapDate = m?.date ?? null; snapBox = (m?.bbox as [number, number, number, number]) ?? null; instr(); renderReadout(); });
    if (exampleId) {
      chooseExample(exampleId);
      if (opts.go) void use();
    } else if (saved?.field) {
      mode = 'gps'; renderMode();
      ring = saved.field.ring; point = saved.field.center; label = saved.label; drawLayers(); map.setView([point.lat, point.lon], 16);
    }
  })().catch(() => { $('#fs-map', host).innerHTML = `<p class="fs__loading mono">${esc(t('fs_search_err'))}</p>`; toast(t('fs_search_err')); });

  return () => { offLang(); map?.remove(); };
}
