// Data layers on the dashboard map. Each one paints the measurement itself, with its colour scale:
// - Vigor: NDVI per 10 m pixel (Sentinel-2), red = bare soil to green = dense crop, around the field;
// - Soil moisture: the 9 km SMAP cell coloured by its value, dry (brown) to wet (blue);
// - Temperature: the ~50 km NASA POWER cell coloured by the anomaly against the 40-year normal;
// - Slope: slope classes per 30 m NASADEM cell over ~5 km, flat to very steep.
// The coarse cells say clearly that they describe the whole area, not the plot.
import type * as Leaflet from 'leaflet';
import type { ClientAnalysis } from '../../shared/api';
import type { Reading } from '../../shared/contract';
import { esc } from '../../shared/dom';
import { fmt, fmtDateShort, t } from '../../shared/i18n';
import { loadbarHtml, startLoadbar } from '../../shared/loadbar';

export interface LayerMeta { kind: string; bbox: [number, number, number, number]; date?: string | null; source: string; resolution: string; credit: string; method: string; image_url: string }

const cache = new Map<string, Promise<LayerMeta | null>>();
export function getLayer(kind: 'ndvi' | 'slope', lat: number, lon: number, sizeM: number): Promise<LayerMeta | null> {
  const k = `${kind}:${lat.toFixed(5)}:${lon.toFixed(5)}:${sizeM}`;
  if (!cache.has(k)) {
    const ctl = new AbortController();
    const to = setTimeout(() => ctl.abort(), 90000);
    cache.set(k, fetch(`/api/imagery/layer.json?kind=${kind}&lat=${lat.toFixed(5)}&lon=${lon.toFixed(5)}&size_m=${sizeM}`, { signal: ctl.signal })
      .then((r) => (r.ok ? r.json() as Promise<LayerMeta> : null)).catch(() => null)
      .then((m) => { clearTimeout(to); if (!m) cache.delete(k); return m; }));
  }
  return cache.get(k)!;
}

// ---------- colour ramps ----------
type Stop = [number, [number, number, number]];
function ramp(stops: Stop[], v: number): string {
  const x = Math.max(stops[0][0], Math.min(stops[stops.length - 1][0], v));
  for (let i = 1; i < stops.length; i++) {
    const [a, ca] = stops[i - 1], [b, cb] = stops[i];
    if (x <= b) { const f = (x - a) / (b - a || 1); return `rgb(${ca.map((c, j) => Math.round(c + (cb[j] - c) * f)).join(',')})`; }
  }
  return `rgb(${stops[stops.length - 1][1].join(',')})`;
}
const css = (stops: Stop[]) => `linear-gradient(90deg, ${stops.map(([v, c]) => `rgb(${c.join(',')}) ${(((v - stops[0][0]) / (stops[stops.length - 1][0] - stops[0][0])) * 100).toFixed(0)}%`).join(', ')})`;

export const SOIL: Stop[] = [[0, [150, 98, 52]], [0.15, [214, 176, 112]], [0.3, [110, 178, 214]], [0.45, [36, 92, 170]]];
export const TEMP: Stop[] = [[-4, [70, 130, 210]], [0, [236, 236, 228]], [3, [244, 170, 80]], [6, [206, 52, 46]]];
const NDVI: Stop[] = [[0, [165, 0, 38]], [0.2, [244, 109, 67]], [0.4, [254, 224, 139]], [0.6, [166, 217, 106]], [0.8, [26, 152, 80]]];
const SLOPE = [
  { k: 'sl_flat', c: 'rgba(126,200,140,0.55)' }, { k: 'sl_gentle', c: 'rgba(236,216,110,0.8)' }, { k: 'sl_mod', c: 'rgba(244,166,72,0.85)' },
  { k: 'sl_strong', c: 'rgba(226,92,58,0.9)' }, { k: 'sl_steep', c: 'rgba(160,36,64,0.9)' },
] as const;

/** Colour scale with the field's value marked on it. */
function scaleBar(stops: Stop[], v: number | null, lo: string, hi: string, fmtV: (x: number) => string): string {
  const a = stops[0][0], b = stops[stops.length - 1][0];
  const pos = v == null ? null : Math.max(0, Math.min(100, ((v - a) / (b - a)) * 100));
  return `<div class="lscale"><span class="lscale__bar" style="background:${css(stops)}">${pos == null ? '' : `<i class="lscale__mark" style="left:${pos.toFixed(1)}%"><b class="mono">${esc(fmtV(v!))}</b></i>`}</span>
    <span class="lscale__ends mono"><span>${esc(lo)}</span><span>${esc(hi)}</span></span></div>`;
}

export interface LayerPainter { paint(k: 'vigor' | 'soil' | 'temp' | 'slope', r: Reading | undefined): void; clear(): void }

export function mountLayers(L: typeof Leaflet, map: Leaflet.Map, a: ClientAnalysis, slot: () => HTMLElement | null, outline: Leaflet.Layer,
  cellBounds: (km: number) => Leaflet.LatLngBounds, fieldBounds: Leaflet.LatLngBounds, resKm: (r: string) => number | null): LayerPainter {
  let overlay: Leaflet.Layer | null = null;
  let token = 0;
  const f = a.field;
  const front = () => (outline as Leaflet.Path).bringToFront?.();
  const clear = () => { overlay?.remove(); overlay = null; };
  const setSlot = (html: string) => { const s = slot(); if (s) s.innerHTML = html; };

  async function image(kind: 'ndvi' | 'slope', sizeM: number, my: number, fit: boolean, scale: (m: LayerMeta) => string) {
    setSlot(loadbarHtml(t(kind === 'ndvi' ? 'lb_ndvi' : 'lb_slope')));
    const el = slot()?.firstElementChild as HTMLElement | null;
    const bar = el ? startLoadbar(el, kind === 'ndvi' ? 15000 : 8000, [kind === 'ndvi' ? 'Sentinel-2 L2A' : 'NASADEM']) : null;
    const m = await getLayer(kind, f.center.lat, f.center.lon, sizeM);
    // The dashboard may have re-rendered (and removed this map) while the layer was loading.
    if (my !== token || !map.getPane('overlayPane')) return;
    if (!m) { bar?.fail(); setSlot(`<p class="dmap__note">${esc(t('ly_img_na'))}</p>`); return; }
    bar?.done();
    const [w, s, e, n] = m.bbox;
    overlay = L.imageOverlay(m.image_url, [[s, w], [n, e]], { opacity: kind === 'ndvi' ? 0.82 : 0.9, interactive: false, className: `lyr lyr--${kind}` }).addTo(map);
    front();
    if (fit) map.fitBounds([[s, w], [n, e]], { padding: [20, 20] });
    setSlot(scale(m));
  }

  return {
    clear,
    paint(k, r) {
      clear();
      const my = ++token;
      setSlot('');
      if (a.offline) return;
      if (k === 'vigor') {
        map.fitBounds(fieldBounds);
        void image('ndvi', 1500, my, false, (m) => `${scaleBar(NDVI, r?.value ?? null, t('ly_ndvi_lo'), t('ly_ndvi_hi'), (x) => fmt(x, 2))}
          <p class="dmap__src mono">${esc(t('ly_ndvi_src', { d: m.date ? fmtDateShort(m.date) : '—' }))}</p>`);
      } else if (k === 'slope') {
        void image('slope', 5000, my, true, () => `<ul class="lclass">${SLOPE.map((c) => `<li><i style="background:${c.c}"></i>${esc(t(c.k))}</li>`).join('')}</ul>
          <p class="dmap__src mono">${esc(t('ly_slope_src'))}</p>`);
      } else if (r && r.value != null) {
        const stops = k === 'soil' ? SOIL : TEMP;
        const km = resKm(r.resolution) ?? (k === 'soil' ? 9 : 50);
        const b = cellBounds(km);
        overlay = L.rectangle(b, { color: '#fff', weight: 1.5, dashArray: '6 6', fillColor: ramp(stops, r.value), fillOpacity: 0.5, interactive: false }).addTo(map);
        front();
        map.fitBounds(b.pad(0.15));
        setSlot(k === 'soil'
          ? `${scaleBar(SOIL, r.value, t('ly_soil_lo'), t('ly_soil_hi'), (x) => `${fmt(x, 2)} m³/m³`)}<p class="dmap__note">${esc(t('ly_cell_area', { km: fmt(km, 0) }))}</p>`
          : `${scaleBar(TEMP, r.value, '−4 °C', '+6 °C', (x) => `${x > 0 ? '+' : ''}${fmt(x, 1)} °C`)}<p class="dmap__note">${esc(t('ly_cell_area', { km: fmt(km, 0) }))}</p>`);
      }
    },
  };
}
