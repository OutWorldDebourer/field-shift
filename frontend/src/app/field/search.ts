// Place search with Nominatim (OpenStreetMap, ODbL): debounced, at most one request per second, aborts stale ones.
import { lang } from '../../shared/i18n';

export interface Place { name: string; lat: number; lon: number; bbox?: [number, number, number, number] }

let last = 0;
let ctl: AbortController | null = null;

export async function searchPlaces(q: string): Promise<Place[]> {
  const wait = Math.max(0, last + 1000 - Date.now());
  if (wait) await new Promise((r) => setTimeout(r, wait));
  ctl?.abort();
  ctl = new AbortController();
  last = Date.now();
  const url = `https://nominatim.openstreetmap.org/search?format=jsonv2&limit=6&addressdetails=0&accept-language=${lang()}&q=${encodeURIComponent(q)}`;
  const r = await fetch(url, { signal: ctl.signal, headers: { Accept: 'application/json' } });
  if (!r.ok) throw new Error(String(r.status));
  const rows = (await r.json()) as { display_name: string; lat: string; lon: string; boundingbox?: string[] }[];
  return rows.map((x) => ({
    name: x.display_name, lat: +x.lat, lon: +x.lon,
    bbox: x.boundingbox ? [+x.boundingbox[2], +x.boundingbox[0], +x.boundingbox[3], +x.boundingbox[1]] : undefined,
  }));
}

export function debounce<A extends unknown[]>(fn: (...a: A) => void, ms: number): (...a: A) => void {
  let h = 0;
  return (...a: A) => { clearTimeout(h); h = window.setTimeout(() => fn(...a), ms); };
}
