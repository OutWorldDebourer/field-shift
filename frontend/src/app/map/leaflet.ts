// Lazy Leaflet loader and the licence-clean satellite base layer (EOX Sentinel-2 cloudless 2016, CC BY 4.0),
// plus the backend's dated Sentinel-2 snapshot as a sharp overlay around the field when it is available.
import type * as Leaflet from 'leaflet';
import { snapshotMeta, snapshotUrl, type SnapshotMeta } from '../../shared/api';
import { bboxAround } from '../../shared/geo';
import { t } from '../../shared/i18n';

let lib: Promise<typeof Leaflet> | null = null;

export function loadLeaflet(): Promise<typeof Leaflet> {
  if (!lib) {
    lib = Promise.all([import('leaflet'), import('leaflet/dist/leaflet.css')]).then(([m]) => (m as unknown as { default?: typeof Leaflet }).default ?? (m as unknown as typeof Leaflet));
  }
  return lib;
}

export const EOX_URL = 'https://tiles.maps.eox.at/wmts/1.0.0/s2cloudless_3857/default/g/{z}/{y}/{x}.jpg';

export function eoxLayer(L: typeof Leaflet): Leaflet.TileLayer {
  const layer = L.tileLayer(EOX_URL, {
    maxNativeZoom: 15, maxZoom: 17, minZoom: 2,
    // Neutral imagery-toned tile instead of a blank hole when a tile fails.
    errorTileUrl: 'data:image/svg+xml;utf8,<svg xmlns="http://www.w3.org/2000/svg" width="256" height="256"><rect width="256" height="256" fill="%2357584a"/></svg>',
    attribution: `<a href="https://s2maps.eu" target="_blank" rel="noopener">Sentinel-2 cloudless 2016</a> by EOX IT Services GmbH (Contains modified Copernicus Sentinel data 2016) · CC BY 4.0`,
  });
  // Retry a failed tile once before the neutral error tile stays.
  layer.on('tileerror', (e: Leaflet.TileErrorEvent) => {
    const img = e.tile as HTMLImageElement & { _retried?: boolean };
    if (img._retried) return;
    img._retried = true;
    setTimeout(() => { img.src = layer.getTileUrl(e.coords); }, 800);
  });
  return layer;
}

/** Adds the dated Sentinel-2 snapshot (backend) around a point; resolves with its credit or null. */
export async function addSnapshot(L: typeof Leaflet, map: Leaflet.Map, lat: number, lon: number, sizeM = 1500): Promise<SnapshotMeta | null> {
  const meta = await snapshotMeta(lat, lon, sizeM);
  if (!meta) return null;
  const b = meta.bbox ?? bboxAround(lat, lon, sizeM);
  const overlay = L.imageOverlay(snapshotUrl(lat, lon, sizeM), [[b[1], b[0]], [b[3], b[2]]], { opacity: 1, className: 'snap-overlay', attribution: `${meta.source ?? 'Sentinel-2 L2A'} ${meta.date ?? ''} · ${meta.credit ?? t('snap_credit_fallback')}` });
  overlay.addTo(map);
  return meta;
}
