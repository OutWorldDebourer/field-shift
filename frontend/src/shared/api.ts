// Typed client for the Rust backend (/api, see contract.ts) with timeouts and an offline fallback to the
// bundled Huaral demo dataset. Anything served from the bundle is flagged `offline` so the UI can say "Demo data".
import type { AnalysisState, Bi, ChatMessage, EvidencePackage, FieldGeom, Lang, Section } from './contract';
import { DEMO_CROPS, DEMO_EXAMPLES, DEMO_EVIDENCE, DEMO_ID, demoAnalysis, demoReply } from './demo/huaral';

export type ClientAnalysis = AnalysisState & { offline?: boolean; readOnly?: boolean; shareToken?: string };
export type Example = { id: string; name: Bi; field: FieldGeom & { area_ha: number; label: string } };
export type CropSuggestion = { key: string; label: Bi; scientific: string };
export type SnapshotMeta = { scene_id?: string; date?: string; source?: string; credit?: string; bbox?: [number, number, number, number]; resolution_m?: number; badge?: string; license?: string };

export class ApiError extends Error {
  constructor(public status: number, message: string) { super(message); }
}

async function req<T>(path: string, init: RequestInit & { timeout?: number } = {}): Promise<T> {
  const ctl = new AbortController();
  const timer = setTimeout(() => ctl.abort(), init.timeout ?? 8000);
  try {
    const r = await fetch(path, { ...init, signal: ctl.signal, headers: { Accept: 'application/json', ...(init.headers || {}) } });
    if (!r.ok) throw new ApiError(r.status, `${r.status} ${r.statusText}`);
    const ct = r.headers.get('content-type') || '';
    if (!ct.includes('json')) throw new ApiError(502, 'not json (backend not running?)');
    return (await r.json()) as T;
  } finally {
    clearTimeout(timer);
  }
}

const json = (body: unknown): RequestInit => ({ method: 'POST', body: JSON.stringify(body), headers: { 'Content-Type': 'application/json' } });

/** True for network failures, timeouts and 5xx — the cases where the demo fallback is honest to use. */
export const isOffline = (e: unknown): boolean => !(e instanceof ApiError) || e.status >= 500 || e.status === 404;

export async function examples(): Promise<{ list: Example[]; offline: boolean }> {
  try { return { list: await req<Example[]>('/api/analyses/examples', { timeout: 5000 }), offline: false }; }
  catch { return { list: DEMO_EXAMPLES, offline: true }; }
}

export async function createAnalysis(body: { field: FieldGeom; location_mode: AnalysisState['location_mode']; example_site?: string | null; lang: Lang; label?: string }): Promise<ClientAnalysis> {
  try {
    return await req<AnalysisState>('/api/analyses', { ...json(body), timeout: 12000 });
  } catch (e) {
    if (body.example_site === 'huaral' && isOffline(e)) return { ...demoAnalysis(body.lang), offline: true };
    throw e;
  }
}

export async function getAnalysis(id: string, lang: Lang): Promise<ClientAnalysis> {
  if (id === DEMO_ID) return { ...demoAnalysis(lang), offline: true };
  return req<AnalysisState>(`/api/analyses/${encodeURIComponent(id)}`, { timeout: 10000 });
}

export async function getShared(token: string): Promise<ClientAnalysis> {
  const a = await req<AnalysisState>(`/api/export/shared/${encodeURIComponent(token)}`, { timeout: 10000 });
  return { ...a, readOnly: true, shareToken: token };
}

export async function buildEvidence(a: ClientAnalysis): Promise<EvidencePackage> {
  if (a.offline) return DEMO_EVIDENCE;
  return req<EvidencePackage>(`/api/analyses/${encodeURIComponent(a.id)}/evidence`, { method: 'POST', timeout: 120000 });
}

export async function sendMessage(a: ClientAnalysis, text: string, photos: Blob[], lang: Lang): Promise<{ messages: ChatMessage[]; sections: Section[]; demo_mode?: boolean }> {
  if (a.offline) {
    await new Promise((r) => setTimeout(r, 1400));
    const now = new Date().toISOString();
    const base = a.messages.reduce((m, x) => Math.max(m, x.id), 0);
    const mine: ChatMessage = { id: base + 1, role: 'user', body: text, created_at: now, photos: photos.map((b) => ({ url: URL.createObjectURL(b) })) };
    const reply: ChatMessage = { id: base + 2, role: 'assistant', body: demoReply(lang, photos.length > 0)[lang], created_at: now };
    const sections = a.sections.map((s) => (photos.length && s.key === 'pests' ? { ...s, state: 'ready' as const, note: undefined } : s));
    return { messages: [...a.messages, mine, reply], sections, demo_mode: true };
  }
  const fd = new FormData();
  fd.append('text', text);
  fd.append('lang', lang);
  photos.forEach((b, i) => fd.append('photos', b, `photo-${i + 1}.jpg`));
  return req(`/api/consultant/${encodeURIComponent(a.id)}/message`, { method: 'POST', body: fd, timeout: 120000 });
}

export async function suggestCrops(q: string): Promise<CropSuggestion[]> {
  const local = () => {
    const n = q.trim().toLowerCase();
    return DEMO_CROPS.filter((c) => [c.label.en, c.label.es, c.scientific, c.key].some((s) => s.toLowerCase().includes(n))).slice(0, 6);
  };
  if (!q.trim()) return [];
  try { return await req<CropSuggestion[]>(`/api/crops/suggest?q=${encodeURIComponent(q)}`, { timeout: 3000 }); }
  catch { return local(); }
}

export function snapshotUrl(lat: number, lon: number, sizeM: number, px = 1024): string {
  return `/api/imagery/snapshot.png?lat=${lat.toFixed(6)}&lon=${lon.toFixed(6)}&size_m=${Math.round(sizeM)}&px=${px}`;
}

export async function snapshotMeta(lat: number, lon: number, sizeM: number, px = 1024): Promise<SnapshotMeta | null> {
  try { return await req<SnapshotMeta>(snapshotUrl(lat, lon, sizeM, px).replace('snapshot.png', 'snapshot.json'), { timeout: 15000 }); }
  catch { return null; }
}

export const sheetUrl = (id: string, lang: Lang) => `/api/export/${encodeURIComponent(id)}/sheet?lang=${lang}`;
export const csvUrl = (id: string) => `/api/export/${encodeURIComponent(id)}/series.csv`;

export async function share(a: ClientAnalysis): Promise<string> {
  if (a.offline) return new URL(`app.html?a=${DEMO_ID}`, location.href).href;
  const r = await req<{ url: string; token?: string }>(`/api/export/${encodeURIComponent(a.id)}/share`, { method: 'POST', timeout: 8000 });
  return new URL(r.url, location.href).href;
}

/** Client-side CSV of the evidence series (used for the offline demo). */
export function evidenceCsv(ev: EvidencePackage): string {
  const rows: string[][] = [['series', 'time', 'value', 'unit', 'source', 'dataset', 'resolution', 'badge', 'licence', 'note']];
  const note = 'DEMO DATA - illustrative values';
  for (const r of ev.readings) for (const p of r.series ?? []) rows.push([r.id, p.t, p.v == null ? '' : String(p.v), r.unit, r.source, r.dataset, r.resolution, r.badge, r.license, note]);
  for (const m of ev.climate_monthly) rows.push(['t_mean_monthly', m.month, String(m.t_mean ?? ''), '°C', 'NASA POWER', 'T2M', '~50 km', 'regional context', 'NASA open data', note]);
  for (const n of ev.ndvi_series) rows.push(['ndvi', n.date, String(n.ndvi), '', n.sensor, n.scene, '10 m', 'field-level', 'Contains modified Copernicus Sentinel data', note]);
  return rows.map((r) => r.map((c) => (/[",\n]/.test(c) ? `"${c.replace(/"/g, '""')}"` : c)).join(',')).join('\n');
}
