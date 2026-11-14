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
}

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
