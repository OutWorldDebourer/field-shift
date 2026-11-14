// Coordinates typed or pasted into the place search: "-8.43, -78.76", "8.43°S 78.76°W", "8°25'48\"S 78°45'36\"W",
// "S 8.43 O 78.76" (O = oeste). Returns null for anything that is not clearly a coordinate pair, so place names
// still go to the search.

const NUM = String.raw`\d+(?:[.,]\d+)?`;
const PART = new RegExp(String.raw`([NSEWO])?\s*(-?${NUM})\s*°?\s*(?:(${NUM})\s*['′]\s*)?(?:(${NUM})\s*(?:"|″|'')\s*)?([NSEWO])?`, 'gi');

const num = (s: string | undefined) => (s ? Number(s.replace(',', '.')) : 0);

export function parseCoords(input: string): { lat: number; lon: number } | null {
  const s = input.trim();
  if (!s || /[a-df-mp-rt-vx-z]{2,}/i.test(s)) return null; // words (other than N/S/E/W/O) mean a place name
  // Plain decimal pair: "-8.43, -78.76" or "-8.43 -78.76" (a comma can be a decimal mark only with a space separator).
  const plain = /^\s*(-?\d+(?:\.\d+)?)\s*[,;\s]\s*(-?\d+(?:\.\d+)?)\s*$/.exec(s) ?? /^\s*(-?\d+(?:,\d+)?)\s*[;\s]\s*(-?\d+(?:,\d+)?)\s*$/.exec(s);
  if (plain) return valid(num(plain[1]), num(plain[2]));
  const parts: { v: number; hemi: string }[] = [];
  for (const m of s.matchAll(PART)) {
    if (!m[2]) continue;
    const deg = num(m[2]);
    const v = Math.sign(deg || 1) * (Math.abs(deg) + num(m[3]) / 60 + num(m[4]) / 3600);
    parts.push({ v, hemi: (m[1] || m[5] || '').toUpperCase() });
    if (parts.length === 2) break;
  }
  if (parts.length !== 2) return null;
  let lat: number | null = null, lon: number | null = null;
  for (const p of parts) {
    if (p.hemi === 'N' || p.hemi === 'S') lat = p.hemi === 'S' ? -Math.abs(p.v) : Math.abs(p.v);
    else if (p.hemi === 'E' || p.hemi === 'W' || p.hemi === 'O') lon = p.hemi === 'E' ? Math.abs(p.v) : -Math.abs(p.v);
  }
  if (lat == null && lon == null) { lat = parts[0].v; lon = parts[1].v; }
  else if (lat == null) lat = parts.find((p) => !p.hemi)?.v ?? null;
  else if (lon == null) lon = parts.find((p) => !p.hemi)?.v ?? null;
  return lat == null || lon == null ? null : valid(lat, lon);
}

const valid = (lat: number, lon: number) => (Math.abs(lat) <= 90 && Math.abs(lon) <= 180 && Number.isFinite(lat) && Number.isFinite(lon) ? { lat, lon } : null);
