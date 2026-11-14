// HTTP contract between the frontend and the Rust backend (all under /api). The backend conforms to these shapes.
// Every number shown to the farmer comes from a Reading: value + source + date + scale badge.

export type Lang = 'en' | 'es';
export type Badge = 'field-level' | 'regional context';
export type SectionKey = 'field' | 'soil' | 'pests' | 'nutrition' | 'rotation' | 'sheet';
export type SectionState = 'empty' | 'progress' | 'ready' | 'gaps' | 'locked';
export type Bi = { en: string; es: string };

export interface FieldGeom {
  center: { lat: number; lon: number };
  /** [lon, lat] vertices, not closed; empty for a point field. */
  ring: [number, number][];
}

export interface Reading {
  id: string;                 // 'temp_anomaly' | 'soil_moisture' | 'ndvi' | 'rain_30d' | 'water_balance' | 'pest_gen' | 'slope' | 'grace' | 'projection' ...
  label: Bi;
  value: number | null;
  unit: string;
  /** Plain words first: "drier than normal". */
  verdict: Bi;
  /** One-line key number for the amber accent, if this reading is the finding on screen. */
  key?: boolean;
  source: string;             // "NASA POWER", "SMAP L4", "Sentinel-2 L2A" ...
  dataset: string;            // dataset id / version
  date: string;               // ISO date or period "1984–2026"
  resolution: string;         // "10 m", "9 km" ...
  badge: Badge;
  license: string;
  method?: string;
  series?: { t: string; v: number | null }[];
  /** Optional (frontend addition): latest values come from near-real-time streams and may change. */
  provisional?: boolean;
  /** Optional (frontend addition): short caveat shown with the reading. */
  note?: Bi;
}

export interface Gap { id: string; text: Bi }

export interface EvidencePackage {
  field: FieldGeom & { area_ha: number; label: string };
  generated_at: string;
  readings: Reading[];
  /** Monthly this year vs 40-year normal band (POWER). */
  climate_monthly: { month: string; t_mean: number | null; normal_mean: number; normal_lo: number; normal_hi: number; rain: number | null; normal_rain: number }[];
  ndvi_series: { date: string; ndvi: number; sensor: string; scene: string }[];
  pests: { name: string; scientific: string; generations_now: number | null; generations_normal: number | null; citation: string }[];
  projection: { horizon: string; scenario: string; d_temp_c: number | null; d_rain_pct: number | null; models: string[] } | null;
  imagery: { snapshot_url: string; credit: string; date: string } | null;
  gaps: Gap[];
  /** Optional (frontend addition): plain-words alerts derived from the readings; the UI derives them itself when absent. */
  alerts?: { id: string; level: 'info' | 'watch' | 'warning'; text: Bi; source: string; date: string }[];
}

export interface CropRecord {
  key: string;
  names: Bi;
  scientific: string;
  family: string;
  crop_type: 'annual' | 'semi-perennial' | 'perennial';
  origin: 'seed' | 'ai-researched';
}

export interface SectionContent {
  title: Bi;
  summary: Bi;
  items: { title: Bi; body: Bi; citations?: { title: string; url: string }[] }[];
  /** Rotation only (annual crops): three strategies compared, current included. */
  strategies?: {
    name: Bi; timeline: string[]; current?: boolean; best_for?: Bi;
    scores: { soil: number; water: number; pest_break: number; income: number; risk: number };
    why: Bi; evidence: Bi; gaps: Bi; risks: Bi;
  }[];
}

export interface Section { key: SectionKey; state: SectionState; note?: Bi; content?: SectionContent }

export interface ChatMessage {
  id: number;
  role: 'user' | 'assistant' | 'status';
  body: string;
  photos?: { url: string; tag?: string }[];
  created_at: string;
}

export interface AnalysisState {
  id: string;
  lang: Lang;
  location_mode: 'gps' | 'example' | 'map';
  example_site: string | null;
  field: FieldGeom & { area_ha: number; label: string };
  crop_current: CropRecord | null;
  crop_history: CropRecord[];
  messages: ChatMessage[];
  sections: Section[];
  evidence: EvidencePackage | null;
  /** True when the consultant runs the labelled scripted demo instead of the live model. */
  demo_mode: boolean;
  /** Optional (frontend addition): one-line pre-diagnosis headline and confidence for the rotation view. */
  prediagnosis?: { headline: Bi; confidence: 'low' | 'medium' | 'high' } | null;
  /** Iteration 2: the guided sequence of topics the consultant works through, in order. */
  steps?: Step[];
  /** Key of the first step not yet done (null when all are done and the sheet is ready). */
  next_step?: SectionKey | null;
}

// ---------------------------------------------------------------- iteration 2 (2026-11-14)

/** One topic of the guided conversation. `done` flips when the consultant has what it needs for that topic. */
export interface Step {
  key: SectionKey;            // field → soil → pests → nutrition → rotation → sheet
  state: 'pending' | 'active' | 'done';
  title: Bi;                  // "Your field", "Soil" …
  /** What the consultant still needs for this topic, in plain words ("soil type", "last fertilizer"). */
  needs: Bi[];
  /** One-line summary of what was learned, once done. */
  summary?: Bi;
}

/** NASA GEOS-FP weather forecast for the field's cell (up to 10 days). */
export interface Forecast {
  source: string;             // "NASA GEOS-FP (GMAO)"
  run: string;                // model run, e.g. "2026-11-14 00 UTC"
  resolution: string;         // "0.25° × 0.3125° (~25 km)"
  badge: Badge;               // regional context
  license: string;
  days: { date: string; tmin: number; tmax: number; tmean: number; rain_mm: number }[];
}

/** Our estimate for the rest of the season from NASA POWER normals: two scenarios, never called a forecast. */
export interface Outlook {
  label: Bi;                  // "Estimate, not a forecast"
  months: { month: string; normal_mean: number; normal_lo: number; normal_hi: number; persist_mean: number }[];
  /** Current 30-day anomaly assumed to persist in the "persist" scenario. */
  anomaly_c: number;
  pests: { name: Bi; scientific: string; to_date: number; by_end_normal: number; by_end_persist: number }[];
  source: string; method: string;
}

export interface GlossaryTerm { key: string; term: Bi; short: Bi; scale?: Bi }

// EvidencePackage gains (optional): forecast?: Forecast | null; outlook?: Outlook | null;
// projection gains: d_temp_range?: [number, number]; baseline_period?: string; label?: string; license?: string.

// Iteration 2 endpoints:
// GET  /api/forecast?lat=&lon=&days=1..10           → Forecast            (NASA GEOS-FP, cached ~6 h)
// GET  /api/power/outlook?lat=&lon=&months=1..6     → Outlook             (NASA POWER normals + current anomaly)
// GET  /api/glossary                                → GlossaryTerm[]
// POST /api/consultant/transcribe  multipart: audio (webm/ogg/mp4/wav ≤ 10 MB), lang → { text, engine: "gemini" }
// GET  /api/export/:id/package.zip                  → ZIP: sheet.html + data.csv + photos/ (originals) + satellite.png + README.txt
// POST /api/analyses returns an AnalysisState whose first message is the consultant's greeting
//      ("I located your field at 11.48°S 77.18°W, 0.57 ha in Huaral…"), and steps/next_step.
// Photos are stored at full resolution (≤ 12 MB each); the frontend must not downscale below 4096 px.

// Endpoints (method path → body → response):
// GET  /api/health                                   → { status, database, keys }
// GET  /api/analyses/examples                        → { id, name: Bi, field: FieldGeom & {area_ha,label} }[]
// POST /api/analyses           { field, location_mode, example_site?, lang } → AnalysisState
// GET  /api/analyses/:id                             → AnalysisState
// POST /api/analyses/:id/evidence                    → EvidencePackage   (builds/refreshes; slow first time)
// POST /api/consultant/:id/message  multipart: text, lang, photos[] (≤10, image/*) → { messages: ChatMessage[]; sections: Section[] }
// GET  /api/crops/suggest?q=                         → { key, label: Bi, scientific }[]
// GET  /api/imagery/snapshot.png?lat=&lon=&size_m=   → PNG true colour (credit in header X-Imagery-Credit)
// GET  /api/export/:id/sheet                         → HTML technical sheet (print to PDF) ; /sheet.pdf when available
// GET  /api/export/:id/series.csv                    → CSV of all series
// POST /api/export/:id/share                         → { url }
