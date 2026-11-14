// Place search with Nominatim (OpenStreetMap, ODbL). Usage policy: no search-as-you-type; one search per explicit
// request (Enter or the Search button), at most one request per second, results cached per query.
import { lang } from '../../shared/i18n';

export interface Place { name: string; lat: number; lon: number; bbox?: [number, number, number, number]; kind?: string }

let last = 0;
let ctl: AbortController | null = null;

const cache = new Map<string, Place[]>();

export async function searchPlaces(q: string): Promise<Place[]> {
  const key = q.trim().toLowerCase() + '|' + lang();
  const hit = cache.get(key);
  if (hit) return hit;
  const wait = Math.max(0, last + 1000 - Date.now());
  if (wait) await new Promise((r) => setTimeout(r, wait));
  ctl?.abort();
  ctl = new AbortController();
  last = Date.now();
  const url = `https://nominatim.openstreetmap.org/search?format=jsonv2&limit=6&addressdetails=0&accept-language=${lang()}&q=${encodeURIComponent(q)}`;
  const r = await fetch(url, { signal: ctl.signal, headers: { Accept: 'application/json' } });
  if (!r.ok) throw new Error(String(r.status));
  const rows = (await r.json()) as { display_name: string; lat: string; lon: string; boundingbox?: string[]; addresstype?: string; type?: string }[];
  // Nominatim often returns the same name for a town, its district and its province: keep one per name + kind.
  const seen = new Set<string>();
  const out = rows.map((x) => ({
    name: x.display_name, lat: +x.lat, lon: +x.lon, kind: x.addresstype || x.type,
    bbox: x.boundingbox ? [+x.boundingbox[2], +x.boundingbox[0], +x.boundingbox[3], +x.boundingbox[1]] : undefined,
  }) as Place).filter((p) => { const k = p.name + '|' + p.kind; if (seen.has(k)) return false; seen.add(k); return true; });
  cache.set(key, out);
  return out;
}

export function debounce<A extends unknown[]>(fn: (...a: A) => void, ms: number): (...a: A) => void {
  let h = 0;
  return (...a: A) => { clearTimeout(h); h = window.setTimeout(() => fn(...a), ms); };
}
