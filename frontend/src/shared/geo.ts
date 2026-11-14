// Geometry helpers: polygon area in hectares (spherical excess on the WGS84 mean radius), centroid, bbox.

const R = 6371008.8;
const rad = (d: number) => (d * Math.PI) / 180;

/** Area of a [lon, lat] ring in hectares. */
export function ringAreaHa(ring: [number, number][]): number {
  if (ring.length < 3) return 0;
  let s = 0;
  for (let i = 0; i < ring.length; i++) {
    const [lon1, lat1] = ring[i];
    const [lon2, lat2] = ring[(i + 1) % ring.length];
    s += rad(lon2 - lon1) * (2 + Math.sin(rad(lat1)) + Math.sin(rad(lat2)));
  }
  return Math.abs((s * R * R) / 2) / 10000;
}

export function ringCenter(ring: [number, number][]): { lat: number; lon: number } {
  const n = ring.length || 1;
  return { lon: ring.reduce((a, p) => a + p[0], 0) / n, lat: ring.reduce((a, p) => a + p[1], 0) / n };
}

/** Largest side of the ring's bounding box in metres. */
export function ringSpanM(ring: [number, number][]): number {
  if (!ring.length) return 0;
  const lons = ring.map((p) => p[0]), lats = ring.map((p) => p[1]);
  const lat0 = rad((Math.min(...lats) + Math.max(...lats)) / 2);
  const w = rad(Math.max(...lons) - Math.min(...lons)) * R * Math.cos(lat0);
  const h = rad(Math.max(...lats) - Math.min(...lats)) * R;
  return Math.max(w, h);
}

export function fmtCoord(lat: number, lon: number): string {
  return `${Math.abs(lat).toFixed(2)}°${lat < 0 ? 'S' : 'N'} ${Math.abs(lon).toFixed(2)}°${lon < 0 ? 'W' : 'E'}`;
}

/** Project lon/lat into a bbox-mapped box of w×h pixels (equirectangular, fine at field scale). */
export function bboxProject(bbox: [number, number, number, number], w: number, h: number) {
  const [x0, y0, x1, y1] = bbox;
  return (lon: number, lat: number) => ({ x: ((lon - x0) / (x1 - x0)) * w, y: ((y1 - lat) / (y1 - y0)) * h });
}

/** A square bbox of `sizeM` metres around a point. */
export function bboxAround(lat: number, lon: number, sizeM: number): [number, number, number, number] {
  const dLat = sizeM / 2 / 110574;
  const dLon = sizeM / 2 / (111320 * Math.cos(rad(lat)));
  return [lon - dLon, lat - dLat, lon + dLon, lat + dLat];
}
