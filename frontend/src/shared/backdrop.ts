// The farmer's field from above: a real satellite plate (backend Sentinel-2 snapshot, else EOX Sentinel-2
// cloudless 2016 tiles) with the field outline as a white hairline and the lock-on reticle. Used behind the app
// and in the index's ground sections. The plate is square (size_m) and is positioned so the field lands on `focus`.
import { snapshotMeta, snapshotUrl } from './api';
import { bboxAround, bboxProject } from './geo';
import { t } from './i18n';

export interface BackdropOpts {
  lat: number; lon: number; ring: [number, number][];
  sizeM?: number;
  /** Where the field centre should sit, as a fraction of the host box. */
  focus?: () => { x: number; y: number };
  /** Extra zoom on top of "cover" (1 = just covers). */
  zoom?: number;
  outline?: 'measured' | 'plan' | 'none';
  reticle?: boolean;
  /** Use only the EOX tiles (no backend call), e.g. on the index. */
  tilesOnly?: boolean;
  onCredit?: (credit: string, date: string | null) => void;
}

export interface Backdrop { el: HTMLElement; layout(): void; setOutline(kind: 'measured' | 'plan' | 'none'): void; fieldBox(): DOMRect | null; destroy(): void }

const EOX = (z: number, x: number, y: number) => `https://tiles.maps.eox.at/wmts/1.0.0/s2cloudless_3857/default/g/${z}/${y}/${x}.jpg`;
const tileX = (lon: number, z: number) => ((lon + 180) / 360) * 2 ** z;
const tileY = (lat: number, z: number) => { const r = (lat * Math.PI) / 180; return ((1 - Math.log(Math.tan(r) + 1 / Math.cos(r)) / Math.PI) / 2) * 2 ** z; };
const tile2lon = (x: number, z: number) => (x / 2 ** z) * 360 - 180;
const tile2lat = (y: number, z: number) => { const n = Math.PI - (2 * Math.PI * y) / 2 ** z; return (180 / Math.PI) * Math.atan(0.5 * (Math.exp(n) - Math.exp(-n))); };

export function mountBackdrop(host: HTMLElement, o: BackdropOpts): Backdrop {
  const sizeM = o.sizeM ?? 1500;
  const bbox = bboxAround(o.lat, o.lon, sizeM);
  const proj = bboxProject(bbox, 1000, 1000);
  const wrap = document.createElement('div');
  wrap.className = 'backdrop';
  wrap.innerHTML = `<div class="backdrop__plate"><div class="backdrop__ground"></div>
    <svg class="backdrop__marks" viewBox="0 0 1000 1000" preserveAspectRatio="none" aria-hidden="true"><path class="backdrop__ring"/></svg>
    <div class="reticle backdrop__reticle"><i></i></div></div>`;
  host.prepend(wrap);
  const plate = wrap.firstElementChild as HTMLElement;
  const ground = plate.querySelector('.backdrop__ground') as HTMLElement;
  const ring = plate.querySelector('.backdrop__ring') as SVGPathElement;
  const ret = plate.querySelector('.backdrop__reticle') as HTMLElement;

  // Outline + reticle in plate-normalised coordinates.
  const pts = o.ring.length ? o.ring.map(([lon, lat]) => proj(lon, lat)) : [proj(o.lon, o.lat)];
  ring.setAttribute('d', o.ring.length >= 3 ? 'M' + pts.map((p) => `${p.x.toFixed(1)} ${p.y.toFixed(1)}`).join('L') + 'Z' : '');
  const xs = pts.map((p) => p.x), ys = pts.map((p) => p.y);
  const pad = o.ring.length >= 3 ? 14 : 22;
  const box = { x0: Math.min(...xs) - pad, y0: Math.min(...ys) - pad, x1: Math.max(...xs) + pad, y1: Math.max(...ys) + pad };
  Object.assign(ret.style, { left: box.x0 / 10 + '%', top: box.y0 / 10 + '%', width: (box.x1 - box.x0) / 10 + '%', height: (box.y1 - box.y0) / 10 + '%' });
  ret.hidden = o.reticle === false;
  const setOutline = (k: 'measured' | 'plan' | 'none') => { wrap.dataset.outline = k; };
  setOutline(o.outline ?? 'measured');

  const fallbackTiles = () => {
    const z = 15;
    const [x0, y0, x1, y1] = bbox;
    const tx0 = Math.floor(tileX(x0, z)), tx1 = Math.floor(tileX(x1, z));
    const ty0 = Math.floor(tileY(y1, z)), ty1 = Math.floor(tileY(y0, z));
    let html = '';
    for (let x = tx0; x <= tx1; x++) for (let y = ty0; y <= ty1; y++) {
      const a = proj(tile2lon(x, z), tile2lat(y, z)), b = proj(tile2lon(x + 1, z), tile2lat(y + 1, z));
      html += `<img alt="" decoding="async" src="${EOX(z, x, y)}" style="left:${a.x / 10}%;top:${a.y / 10}%;width:${(b.x - a.x) / 10 + 0.05}%;height:${(b.y - a.y) / 10 + 0.05}%">`;
    }
    ground.innerHTML = html;
    ground.classList.add('is-tiles');
    o.onCredit?.(t('eox_short'), null);
  };

  if (o.tilesOnly) fallbackTiles();
  else {
    const img = new Image();
    img.alt = '';
    img.decoding = 'async';
    img.onload = () => {
      ground.replaceChildren(img);
      void snapshotMeta(o.lat, o.lon, sizeM).then((m) => o.onCredit?.(m?.credit ? `${m.source ?? 'Sentinel-2 L2A'} · ${m.credit}` : t('snap_credit_fallback'), m?.date ?? null));
    };
    img.onerror = fallbackTiles;
    img.src = snapshotUrl(o.lat, o.lon, sizeM);
  }

  const layout = () => {
    const W = host.clientWidth, H = host.clientHeight;
    if (!W || !H) return;
    const S = Math.max(W, H) * (o.zoom ?? 1.1);
    const f = o.focus?.() ?? { x: 0.5, y: 0.5 };
    const c = proj(o.lon, o.lat);
    let left = f.x * W - (c.x / 1000) * S;
    let top = f.y * H - (c.y / 1000) * S;
    left = Math.min(0, Math.max(W - S, left));
    top = Math.min(0, Math.max(H - S, top));
    Object.assign(plate.style, { width: S + 'px', height: S + 'px', transform: `translate3d(${left}px, ${top}px, 0)` });
  };
  const ro = new ResizeObserver(layout);
  ro.observe(host);
  layout();
  return {
    el: wrap, layout, setOutline,
    fieldBox: () => ret.getBoundingClientRect(),
    destroy: () => { ro.disconnect(); wrap.remove(); },
  };
}
