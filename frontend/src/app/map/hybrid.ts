// Hybrid view: OpenStreetMap vectors (roads, rivers, town and road names) drawn over the satellite image.
// Vector tiles from OpenFreeMap (no key, no account; data © OpenStreetMap contributors, ODbL), rendered with
// MapLibre GL inside Leaflet. Loaded lazily, only when the hybrid view is first shown.
import type * as Leaflet from 'leaflet';

const ATTR = '© <a href="https://www.openstreetmap.org/copyright" target="_blank" rel="noopener">OpenStreetMap</a> contributors · <a href="https://openfreemap.org" target="_blank" rel="noopener">OpenFreeMap</a>';

type Lang = 'en' | 'es';

/** A transparent style: only lines and labels, so the satellite image stays visible underneath. */
function style(lang: Lang): unknown {
  const name = ['coalesce', ['get', `name:${lang}`], ['get', 'name:latin'], ['get', 'name']];
  // Zoom must be the outer interpolation; the road class only scales the width inside it.
  const byClass = (big: number, mid: number, small: number) => ['match', ['get', 'class'], ['motorway', 'trunk', 'primary'], big, ['secondary', 'tertiary'], mid, small];
  const roadW = (big: number, mid: number, small: number) =>
    ['interpolate', ['exponential', 1.5], ['zoom'], 8, ['*', 0.4, byClass(big, mid, small)], 12, byClass(big, mid, small), 16, ['*', 4, byClass(big, mid, small)]];
  const halo = 'rgba(8,16,26,0.9)';
  return {
    version: 8,
    glyphs: 'https://tiles.openfreemap.org/fonts/{fontstack}/{range}.pbf',
    sources: { osm: { type: 'vector', url: 'https://tiles.openfreemap.org/planet' } },
    layers: [
      { id: 'water-lines', type: 'line', source: 'osm', 'source-layer': 'waterway', minzoom: 9,
        paint: { 'line-color': '#7fc4f0', 'line-opacity': 0.85, 'line-width': ['interpolate', ['linear'], ['zoom'], 9, 0.6, 15, 2.4] } },
      { id: 'roads-casing', type: 'line', source: 'osm', 'source-layer': 'transportation', minzoom: 8,
        filter: ['in', ['get', 'class'], ['literal', ['motorway', 'trunk', 'primary', 'secondary', 'tertiary', 'minor', 'service', 'track']]],
        layout: { 'line-cap': 'round', 'line-join': 'round' },
        paint: { 'line-color': halo, 'line-opacity': 0.55,
          'line-width': roadW(3.2, 2.4, 1.6) } },
      { id: 'roads', type: 'line', source: 'osm', 'source-layer': 'transportation', minzoom: 8,
        filter: ['in', ['get', 'class'], ['literal', ['motorway', 'trunk', 'primary', 'secondary', 'tertiary', 'minor', 'service', 'track']]],
        layout: { 'line-cap': 'round', 'line-join': 'round' },
        paint: {
          'line-color': ['match', ['get', 'class'], ['motorway', 'trunk', 'primary'], '#ffd98a', ['secondary', 'tertiary'], '#fff3d6', '#ffffff'],
          'line-opacity': ['match', ['get', 'class'], ['track', 'service'], 0.6, 0.9],
          'line-width': roadW(1.8, 1.2, 0.7),
          'line-dasharray': ['match', ['get', 'class'], 'track', ['literal', [2, 2]], ['literal', [1, 0]]],
        } },
      { id: 'road-names', type: 'symbol', source: 'osm', 'source-layer': 'transportation_name', minzoom: 13,
        layout: { 'symbol-placement': 'line', 'text-field': name, 'text-font': ['Noto Sans Regular'], 'text-size': 12, 'text-max-angle': 30 },
        paint: { 'text-color': '#ffffff', 'text-halo-color': halo, 'text-halo-width': 1.6 } },
      { id: 'water-names', type: 'symbol', source: 'osm', 'source-layer': 'waterway', minzoom: 11,
        layout: { 'symbol-placement': 'line', 'text-field': name, 'text-font': ['Noto Sans Italic'], 'text-size': 12 },
        paint: { 'text-color': '#bfe3fa', 'text-halo-color': halo, 'text-halo-width': 1.4 } },
      { id: 'places', type: 'symbol', source: 'osm', 'source-layer': 'place',
        filter: ['in', ['get', 'class'], ['literal', ['city', 'town', 'village', 'hamlet', 'suburb', 'neighbourhood']]],
        layout: {
          'text-field': name, 'text-font': ['Noto Sans Bold'],
          'text-size': ['match', ['get', 'class'], 'city', 18, 'town', 15, 'village', 13, 12],
          'text-max-width': 8, 'text-padding': 4,
        },
        paint: { 'text-color': '#ffffff', 'text-halo-color': halo, 'text-halo-width': 2 } },
    ],
  };
}

export interface Hybrid { layer: Leaflet.Layer; setLang(lang: Lang): void }

/** Builds the hybrid vector layer in the given pane (call once; add or remove the returned layer as needed). */
export async function createHybrid(lang: Lang, pane: string): Promise<Hybrid> {
  const [{ maplibreGL }, ml, worker] = await Promise.all([
    import('@maplibre/maplibre-gl-leaflet'),
    import('maplibre-gl'),
    // MapLibre 6 loads its tile worker from a separate file next to its own module; once bundled that file is not
    // there, so ship it as a static asset and point MapLibre to it.
    import('maplibre-gl/dist/maplibre-gl-worker.mjs?url'),
    import('maplibre-gl/dist/maplibre-gl.css'),
  ]);
  ml.setWorkerUrl(worker.default);
  const gl = maplibreGL({ style: style(lang) as never, pane, interactive: false, attribution: ATTR } as never) as unknown as Leaflet.Layer & { getMaplibreMap(): { setStyle(s: unknown): void } };
  return {
    layer: gl,
    setLang(l: Lang) { try { gl.getMaplibreMap().setStyle(style(l)); } catch { /* not on the map yet */ } },
  };
}
