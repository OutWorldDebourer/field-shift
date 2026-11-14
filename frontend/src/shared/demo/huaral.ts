// Offline fallback for the Huaral example. Every number here comes from `huaral-state.json`, a full AnalysisState
// exported from the live backend (real NASA POWER, Sentinel-2, NASADEM readings and cited section content).
// The UI labels it "Demo data" wherever it is used. Only the scripted chat (no figures) is written here.
import type { AnalysisState, Bi, ChatMessage, EvidencePackage, Lang } from '../contract';
import raw from './huaral-state.json';

export const DEMO_ID = 'demo-huaral';

// The export can carry UTF-8 text decoded as Windows-1252 ("dÃ­as"); repair it so Spanish reads correctly.
const CP1252: Record<string, number> = { '€': 0x80, '‚': 0x82, 'ƒ': 0x83, '„': 0x84, '…': 0x85, '†': 0x86, '‡': 0x87, 'ˆ': 0x88, '‰': 0x89, 'Š': 0x8a, '‹': 0x8b, 'Œ': 0x8c, 'Ž': 0x8e, '‘': 0x91, '’': 0x92, '“': 0x93, '”': 0x94, '•': 0x95, '–': 0x96, '—': 0x97, '˜': 0x98, '™': 0x99, 'š': 0x9a, '›': 0x9b, 'œ': 0x9c, 'ž': 0x9e, 'Ÿ': 0x9f };
function fixText(s: string): string {
  if (!/[ÃÂâ]/.test(s)) return s;
  const bytes: number[] = [];
  for (const ch of s) {
    const c = ch.charCodeAt(0);
    if (c < 256) bytes.push(c); else if (CP1252[ch] != null) bytes.push(CP1252[ch]); else return s;
  }
  try { return new TextDecoder('utf-8', { fatal: true }).decode(new Uint8Array(bytes)); } catch { return s; }
}
function fixDeep<T>(v: T): T {
  if (typeof v === 'string') return fixText(v) as unknown as T;
  if (Array.isArray(v)) return v.map(fixDeep) as unknown as T;
  if (v && typeof v === 'object') return Object.fromEntries(Object.entries(v).map(([k, x]) => [k, fixDeep(x)])) as T;
  return v;
}

const STATE = fixDeep(raw as unknown as AnalysisState);

export const HUARAL_FIELD = STATE.field;
export const DEMO_EVIDENCE: EvidencePackage = STATE.evidence!;
export const DEMO_EXAMPLES = [{ id: 'huaral', name: { en: 'Huaral valley, Peru', es: 'Valle de Huaral, Perú' } as Bi, field: HUARAL_FIELD }];

export function huaralRing(): [number, number][] { return HUARAL_FIELD.ring; }

/** A reading by id prefix from the real package, or undefined. */
export const demoReading = (prefix: string) => DEMO_EVIDENCE.readings.find((r) => r.id.startsWith(prefix));

const bi = (en: string, es: string): Bi => ({ en, es });

const MSG: { role: ChatMessage['role']; body: Bi; photos?: { url: string; tag?: string }[] }[] = [
  { role: 'assistant', body: bi('Hello. I am your field consultant. What are you growing now, and what grew here before?', 'Hola. Soy tu consultor de campo. ¿Qué siembras ahora y qué hubo antes aquí?') },
  { role: 'user', body: bi('Peas now. Lettuce before.', 'Arveja ahora. Lechuga antes.') },
  {
    role: 'user', body: bi('Here are my plants.', 'Aquí están mis plantas.'),
    photos: [{ url: '/img/pea-plant.jpg' }, { url: '/img/pea-leaf.jpg' }, { url: '/img/pea-leaf.jpg', tag: 'blurry' }, { url: '/img/pea-plant.jpg' }],
  },
  { role: 'assistant', body: bi('Thanks. Photo 3 is blurry. Take the underside of a leaf, closer.', 'Gracias. La foto 3 está borrosa. Toma el envés de una hoja, más cerca.') },
  { role: 'status', body: bi('Read NASA POWER · Sentinel-2 · NASADEM for your field', 'NASA POWER · Sentinel-2 · NASADEM leídos para tu campo') },
];

export function demoMessages(lang: Lang): ChatMessage[] {
  return MSG.map((m, i) => ({ id: i + 1, role: m.role, body: m.body[lang], photos: m.photos, created_at: '2026-11-13T14:0' + i + ':00Z' }));
}

export function demoAnalysis(lang: Lang): AnalysisState {
  const s = structuredClone(STATE);
  return { ...s, id: DEMO_ID, lang, messages: demoMessages(lang), demo_mode: true };
}

/** Scripted continuation of the example story for the offline demo consultant. */
export function demoReply(lang: Lang, hadPhotos: boolean): Bi {
  void lang;
  return hadPhotos
    ? bi('Demo mode: I cannot read new photos offline, so the example story continues. In the example, the closer photo shows small green insects on the underside, consistent with aphids. Pest care is updated. Confirm with a hand lens or your agronomist.',
         'Modo demostración: sin conexión no puedo leer fotos nuevas, así que sigue la historia de ejemplo. En el ejemplo, la foto cercana muestra insectos verdes pequeños en el envés, compatibles con pulgones. Manejo de plagas actualizado. Confírmalo con una lupa o con tu agrónomo.')
    : bi('Demo mode: the scripted consultant follows the example story. Next question in the example: what fertilizer did you use last, and when? "I don’t know" is a fine answer; it becomes an information gap.',
         'Modo demostración: el consultor guionizado sigue la historia de ejemplo. Siguiente pregunta del ejemplo: ¿qué fertilizante usaste por última vez y cuándo? "No sé" es una respuesta válida; queda como vacío de información.');
}

/** Small offline crop list so suggestions still work without the server (suggestions only, never a restriction). */
export const DEMO_CROPS: { key: string; label: Bi; scientific: string }[] = [
  ['pea', 'Peas', 'Arveja', 'Pisum sativum'], ['lettuce', 'Lettuce', 'Lechuga', 'Lactuca sativa'], ['maize', 'Maize', 'Maíz', 'Zea mays'],
  ['potato', 'Potato', 'Papa', 'Solanum tuberosum'], ['bean', 'Common bean', 'Frijol', 'Phaseolus vulgaris'], ['tomato', 'Tomato', 'Tomate', 'Solanum lycopersicum'],
  ['strawberry', 'Strawberry', 'Fresa', 'Fragaria × ananassa'], ['grape', 'Grape', 'Vid', 'Vitis vinifera'], ['mango', 'Mango', 'Mango', 'Mangifera indica'],
  ['avocado', 'Avocado', 'Palta', 'Persea americana'], ['mandarin', 'Mandarin', 'Mandarina', 'Citrus reticulata'], ['sweet_potato', 'Sweet potato', 'Camote', 'Ipomoea batatas'],
  ['onion', 'Onion', 'Cebolla', 'Allium cepa'], ['cotton', 'Cotton', 'Algodón', 'Gossypium barbadense'], ['pineapple', 'Pineapple', 'Piña', 'Ananas comosus'],
  ['alfalfa', 'Alfalfa', 'Alfalfa', 'Medicago sativa'], ['oat_vetch', 'Oat with vetch', 'Avena con vicia', 'Avena sativa + Vicia sativa'],
].map(([key, en, es, scientific]) => ({ key, label: { en, es }, scientific }));
