// Typed client for the Rust backend (/api, see contract.ts) with timeouts and an offline fallback to the
// bundled Huaral demo dataset. Anything served from the bundle is flagged `offline` so the UI can say "Demo data".
import { getViewContext } from './viewctx';
import type { AnalysisState, Bi, ChatMessage, EvidencePackage, FieldGeom, Forecast, GlossaryTerm, IntakeForm, Lang, Outlook, Section, SectionKey, Step } from './contract';
import { DEMO_CROPS, DEMO_EXAMPLES, DEMO_EVIDENCE, DEMO_ID, HUARAL_INTAKE_EXAMPLE, demoAnalysis, demoReply } from './demo/huaral';

/** facts: iteration 3 — what the farmer has told so far (AnalysisState.facts in the contract notes). */
export type ClientAnalysis = AnalysisState & { offline?: boolean; readOnly?: boolean; shareToken?: string; facts?: IntakeForm | null };
export type Example = { id: string; name: Bi; field: FieldGeom & { area_ha: number; label: string }; intake_example?: IntakeForm; intake_example_es?: IntakeForm };
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

export type TurnResult = { messages: ChatMessage[]; sections: Section[]; demo_mode?: boolean; steps?: Step[]; next_step?: SectionKey | null };

const EXT: Record<string, string> = { 'image/jpeg': 'jpg', 'image/png': 'png', 'image/webp': 'webp', 'image/heic': 'heic', 'image/heif': 'heif', 'image/gif': 'gif' };

/** `?demo=1` in the page URL asks the backend for the scripted consultant (test turns that cost no API credit). */
const forceDemo = (): boolean => { try { return new URLSearchParams(location.search).get('demo') === '1'; } catch { return false; } };

export async function sendMessage(a: ClientAnalysis, text: string, photos: Blob[], lang: Lang): Promise<TurnResult> {
  if (a.offline) {
    await new Promise((r) => setTimeout(r, 1400));
    const now = new Date().toISOString();
    const base = a.messages.reduce((m, x) => Math.max(m, x.id), 0);
    const mine: ChatMessage = { id: base + 1, role: 'user', body: text, created_at: now, photos: photos.map((b) => ({ url: URL.createObjectURL(b), ...(b.type === 'application/pdf' ? { kind: 'pdf', name: b instanceof File ? b.name : 'document.pdf', bytes: b.size } : {}) })) };
    const bi = demoReply(lang, photos.length > 0);
    const reply = { id: base + 2, role: 'assistant', body: bi[lang], _bi: bi, created_at: now } as ChatMessage;
    const sections = a.sections.map((s) => (photos.length && s.key === 'pests' ? { ...s, state: 'ready' as const, note: undefined } : s));
    return { messages: [...a.messages, mine, reply], sections, demo_mode: true };
  }
  const fd = new FormData();
  fd.append('text', text);
  fd.append('lang', lang);
  if (forceDemo()) fd.append('demo', '1');
  const ctx = getViewContext();
  fd.append('view', ctx.view);
  if (ctx.focus) fd.append('focus', ctx.focus);
  // Images keep the iteration-2 naming; PDFs (iteration 3) go up with their real file name and type.
  photos.forEach((b, i) => fd.append('photos', b, b.type === 'application/pdf' ? (b instanceof File && b.name ? b.name : `document-${i + 1}.pdf`) : `photo-${i + 1}.${EXT[b.type] ?? 'jpg'}`));
  return req(`/api/consultant/${encodeURIComponent(a.id)}/message`, { method: 'POST', body: fd, timeout: 180000 });
}

/** The consultant's opening, written for this field (place, coordinates, area). Idempotent on the server. */
export async function startConsultant(a: ClientAnalysis, lang: Lang): Promise<TurnResult> {
  return req(`/api/consultant/${encodeURIComponent(a.id)}/start`, { method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify({ lang, demo: forceDemo() }), timeout: 60000 });
}

// ---------------------------------------------------------------- iteration 3

/** POST the "load information" answers; the backend completes the topics they cover and returns the analysis. */
export async function submitIntake(a: ClientAnalysis, form: IntakeForm): Promise<ClientAnalysis> {
  const q = forceDemo() ? '?demo=1' : '';
  return req<AnalysisState>(`/api/analyses/${encodeURIComponent(a.id)}/intake${q}`, { ...json(form), timeout: 240000 });
}

/** "Evaluate your idea": score any crop as a fourth rotation column; returns the analysis. */
export async function evaluateCrop(a: ClientAnalysis, crop: string): Promise<ClientAnalysis> {
  return req<AnalysisState>(`/api/analyses/${encodeURIComponent(a.id)}/rotation/evaluate`, { ...json({ crop }), timeout: 240000 });
}

/** Example answers for an example site: the backend's `intake_example`, else the bundled labelled sample. */
export async function exampleIntake(site: string | null, l: Lang): Promise<IntakeForm | null> {
  const { list } = await examples();
  const ex = list.find((x) => x.id === (site ?? 'huaral')) ?? list[0];
  const remote = l === 'es' ? ex?.intake_example_es ?? ex?.intake_example : ex?.intake_example;
  if (remote && Object.keys(remote).length) return remote;
  return (site ?? 'huaral') === 'huaral' ? HUARAL_INTAKE_EXAMPLE[l] : null;
}

// ---------------------------------------------------------------- iteration 2

/** NASA GEOS-FP forecast for the field's cell; null when the backend or the network does not answer. */
export async function getForecast(lat: number, lon: number, days: number): Promise<Forecast | null> {
  try {
    const f = await req<Forecast>(`/api/forecast?lat=${lat.toFixed(5)}&lon=${lon.toFixed(5)}&days=${days}`, { timeout: 90000 });
    return f && Array.isArray(f.days) ? f : null;
  } catch { return null; }
}

/** Season outlook from POWER normals + current anomaly (two scenarios); null when unavailable. */
export async function getOutlook(lat: number, lon: number, months: number): Promise<Outlook | null> {
  try {
    const o = await req<Outlook>(`/api/power/outlook?lat=${lat.toFixed(5)}&lon=${lon.toFixed(5)}&months=${months}`, { timeout: 30000 });
    return o && Array.isArray(o.months) ? o : null;
  } catch { return null; }
}

export async function getGlossary(): Promise<GlossaryTerm[] | null> {
  try {
    const g = await req<GlossaryTerm[] | { terms?: GlossaryTerm[] }>('/api/glossary', { timeout: 6000 });
    const list = Array.isArray(g) ? g : g.terms ?? [];
    return list.length ? list : null;
  } catch { return null; }
}

/** Speech to text through the backend (Gemini). Throws on any failure so the caller can fall back. */
export async function transcribe(audio: Blob, lang: Lang): Promise<string> {
  const fd = new FormData();
  const ext = audio.type.includes('ogg') ? 'ogg' : audio.type.includes('mp4') ? 'm4a' : audio.type.includes('wav') ? 'wav' : 'webm';
  fd.append('audio', audio, `dictation.${ext}`);
  fd.append('lang', lang);
  const r = await req<{ text?: string }>('/api/consultant/transcribe', { method: 'POST', body: fd, timeout: 60000 });
  if (typeof r.text !== 'string') throw new ApiError(502, 'no text');
  return r.text;
}

export const packageUrl = (id: string) => `/api/export/${encodeURIComponent(id)}/package.zip`;

export async function suggestCrops(q: string): Promise<CropSuggestion[]> {
  const local = () => {
    const n = q.trim().toLowerCase();
    return DEMO_CROPS.filter((c) => [c.label.en, c.label.es, c.scientific, c.key].some((s) => s.toLowerCase().includes(n))).slice(0, 6);
  };
  if (!q.trim()) return [];
  type Raw = { key: string; name?: { en?: string; es?: string }; label?: Bi; scientific_name?: string; scientific?: string };
  try {
    // The backend answers { suggestions: [...], accepts_any: true }; older shapes were a bare list.
    const r = await req<Raw[] | { suggestions?: Raw[] }>(`/api/crops/suggest?q=${encodeURIComponent(q)}`, { timeout: 3000 });
    const rows = Array.isArray(r) ? r : r.suggestions ?? [];
    return rows.map((x) => ({
      key: x.key,
      label: x.label ?? { en: x.name?.en ?? x.key, es: x.name?.es ?? x.name?.en ?? x.key },
      scientific: x.scientific ?? x.scientific_name ?? '',
    }));
  }
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
