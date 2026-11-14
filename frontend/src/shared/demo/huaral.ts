// Bundled Huaral demo dataset, used only when the backend cannot be reached. Every value here is
// ILLUSTRATIVE DEMO DATA (plausible for the lower Huaral valley, consistent with the approved hero) and the UI
// labels it "Demo data" wherever it appears. Real values come from the backend's evidence package.
import type { AnalysisState, Bi, ChatMessage, EvidencePackage, Lang, Reading, Section } from '../contract';

export const DEMO_ID = 'demo-huaral';
const LAT = -11.4729, LON = -77.2061;

/** The backend's Huaral example plot (~5.1 ha, traced on Sentinel-2 2026-11-13), [lon, lat]. */
export function huaralRing(): [number, number][] {
  return [[-77.207403, -11.471998], [-77.206417, -11.471659], [-77.204745, -11.472721], [-77.205638, -11.473648], [-77.206692, -11.474191], [-77.207426, -11.473287]];
}

export const HUARAL_FIELD = { center: { lat: LAT, lon: LON }, ring: huaralRing(), area_ha: 5.1, label: 'Huaral valley, Peru' };

export const DEMO_EXAMPLES = [{ id: 'huaral', name: { en: 'Huaral valley, Peru', es: 'Valle de Huaral, Perú' } as Bi, field: HUARAL_FIELD }];

const bi = (en: string, es: string): Bi => ({ en, es });
const months = ['2025-12', '2026-01', '2026-02', '2026-03', '2026-04', '2026-05', '2026-06', '2026-07', '2026-08', '2026-09', '2026-10', '2026-11'];
const normal = [21.0, 22.5, 23.6, 23.2, 21.4, 19.3, 17.6, 16.7, 16.4, 16.8, 17.8, 19.2];
const anom = [0.6, 0.9, 1.3, 1.1, 1.4, 1.6, 1.2, 1.0, 1.1, 1.3, 1.4, 1.2];
const nrain = [0.6, 1.2, 2.0, 1.8, 0.4, 0.3, 0.6, 0.9, 1.0, 0.8, 0.5, 0.4];
const rain = [0.2, 0.8, 1.1, 0.9, 0.1, 0.3, 0.4, 0.6, 0.7, 0.2, 0.1, 0.6];

const readings: Reading[] = [
  {
    id: 'temp_anomaly', label: bi('Temperature', 'Temperatura'), value: 1.2, unit: '°C', key: true,
    verdict: bi('warmer than the 40-year normal', 'más cálido que la normal de 40 años'),
    source: 'NASA POWER', dataset: 'POWER daily T2M (MERRA-2)', date: '1984–2026', resolution: '~50 km', badge: 'regional context',
    license: 'NASA open data (no restrictions)', method: 'Last 30 days vs same days 1984–2026',
    series: months.map((m, i) => ({ t: m, v: anom[i] })),
  },
  {
    id: 'soil_moisture', label: bi('Soil moisture', 'Humedad del suelo'), value: null, unit: 'm³/m³',
    verdict: bi('drier than normal', 'más seco de lo normal'),
    source: 'SMAP L4', dataset: 'SPL4SMGP v7 surface soil moisture', date: '2026-11-10', resolution: '9 km', badge: 'regional context',
    license: 'NASA open data (no restrictions)', method: 'Weekly mean vs 2015–2025 same week',
    series: [0.24, 0.23, 0.23, 0.22, 0.21, 0.2, 0.19, 0.18].map((v, i) => ({ t: `w${i}`, v })),
  },
  {
    id: 'ndvi', label: bi('Crop vigor', 'Vigor del cultivo'), value: 0.48, unit: '',
    verdict: bi('moderate, typical of peas at pod-fill', 'moderado, típico de arveja en llenado de vaina'),
    source: 'Sentinel-2 L2A', dataset: 'NDVI field mean (B08, B04)', date: '2026-11-09', resolution: '10 m', badge: 'field-level',
    license: 'Contains modified Copernicus Sentinel data 2026',
    series: [0.18, 0.22, 0.31, 0.38, 0.44, 0.47, 0.48].map((v, i) => ({ t: `s${i}`, v })),
  },
  {
    id: 'rain_30d', label: bi('Rain, last 30 days', 'Lluvia, últimos 30 días'), value: 0.6, unit: 'mm',
    verdict: bi('normal for the dry season', 'normal para la temporada seca'),
    source: 'GPM IMERG Late', dataset: 'GPM_3IMERGDL v07', date: '2026-11-12', resolution: '10 km', badge: 'regional context',
    license: 'NASA open data (no restrictions)', series: rain.map((v, i) => ({ t: months[i], v })),
  },
  {
    id: 'water_balance', label: bi('Water balance', 'Balance hídrico'), value: -42, unit: 'mm',
    verdict: bi('irrigation has to cover the deficit', 'el riego debe cubrir el déficit'),
    source: 'NASA POWER · FAO-56', dataset: 'ET0 Penman-Monteith − rain, 30 days', date: '2026-11-13', resolution: '~50 km', badge: 'regional context',
    license: 'NASA open data (no restrictions)', method: 'FAO-56 reference ET minus rainfall',
    series: [-30, -33, -35, -36, -39, -41, -42].map((v, i) => ({ t: `d${i}`, v })),
  },
  {
    id: 'pest_gen', label: bi('Aphid pressure', 'Presión de pulgones'), value: 1.8, unit: 'gen.',
    verdict: bi('more aphid generations than normal', 'más generaciones de pulgón que lo normal'),
    source: 'NASA POWER · degree-days', dataset: 'T2M_MAX/T2M_MIN, base 4 °C', date: '2026-08-20 → 2026-11-13', resolution: '~50 km', badge: 'regional context',
    license: 'NASA open data (no restrictions)', method: 'Degree-days since sowing ÷ degree-days per generation (demo threshold)',
    series: [0.2, 0.5, 0.8, 1.1, 1.4, 1.6, 1.8].map((v, i) => ({ t: `g${i}`, v })),
  },
  {
    id: 'slope', label: bi('Slope', 'Pendiente'), value: 1.2, unit: '°',
    verdict: bi('flat, low erosion risk', 'plano, bajo riesgo de erosión'),
    source: 'NASADEM', dataset: 'NASADEM_HGT v001', date: '2000', resolution: '30 m', badge: 'field-level', license: 'NASA open data (no restrictions)',
  },
  {
    id: 'grace', label: bi('Water storage trend', 'Tendencia de agua almacenada'), value: 0.24, unit: 'cm/yr',
    verdict: bi('stable to slightly rising since 2002', 'estable o en leve alza desde 2002'),
    source: 'GRACE/GRACE-FO', dataset: 'JPL mascon CRI v4', date: '2002–2026', resolution: '~300 km', badge: 'regional context', license: 'NASA open data (no restrictions)',
    series: [-2.1, -1.4, 0.3, -0.5, 1.2, 0.8, 1.9, 2.4].map((v, i) => ({ t: `y${i}`, v })),
  },
];

export const DEMO_EVIDENCE: EvidencePackage = {
  field: HUARAL_FIELD,
  generated_at: '2026-11-13T14:00:00Z',
  readings,
  climate_monthly: months.map((m, i) => ({
    month: m, t_mean: +(normal[i] + anom[i]).toFixed(1), normal_mean: normal[i], normal_lo: +(normal[i] - 0.7).toFixed(1), normal_hi: +(normal[i] + 0.7).toFixed(1),
    rain: rain[i], normal_rain: nrain[i],
  })),
  ndvi_series: [
    ['2026-04-08', 0.32], ['2026-04-28', 0.46], ['2026-05-18', 0.55], ['2026-06-12', 0.52], ['2026-07-02', 0.21], ['2026-07-22', 0.15],
    ['2026-08-16', 0.18], ['2026-09-05', 0.26], ['2026-09-25', 0.36], ['2026-10-15', 0.44], ['2026-10-30', 0.47], ['2026-11-09', 0.48],
  ].map(([d, v]) => ({ date: d as string, ndvi: v as number, sensor: 'Sentinel-2 L2A', scene: 'demo' })),
  pests: [
    { name: 'Pea aphid', scientific: 'Acyrthosiphon pisum', generations_now: 6.1, generations_normal: 4.3, citation: 'Demo threshold: base 4 °C; verify against the cited source in the technical sheet' },
    { name: 'Leaf miner', scientific: 'Liriomyza huidobrensis', generations_now: 3.4, generations_normal: 2.8, citation: 'Demo threshold: base 8 °C; verify against the cited source in the technical sheet' },
  ],
  projection: { horizon: '2040', scenario: 'SSP2-4.5', d_temp_c: 1.8, d_rain_pct: -4, models: ['NEX-GDDP-CMIP6 ensemble (open-licence models)'] },
  imagery: null,
  gaps: [
    { id: 'fertilizer', text: bi('Last fertilizer used and its date', 'Último fertilizante usado y su fecha') },
    { id: 'photo', text: bi('A sharp photo of the underside of a leaf', 'Una foto nítida del envés de una hoja') },
    { id: 'water', text: bi('Irrigation source and schedule', 'Fuente y frecuencia de riego') },
  ],
  alerts: [
    { id: 'aphid', level: 'warning', text: bi('Aphid generation window: warm nights speed up the next generation', 'Ventana de generación de pulgones: las noches cálidas aceleran la siguiente'), source: 'NASA POWER · degree-days', date: '2026-11-13' },
    { id: 'deficit', level: 'watch', text: bi('Irrigation deficit: about 42 mm in 30 days must come from irrigation', 'Déficit de riego: unos 42 mm en 30 días deben venir del riego'), source: 'NASA POWER · FAO-56', date: '2026-11-13' },
    { id: 'heat', level: 'info', text: bi('Warmer than normal for 9 of the last 12 months', 'Más cálido de lo normal en 9 de los últimos 12 meses'), source: 'NASA POWER', date: '1984–2026' },
  ],
};

const sections: Section[] = [
  { key: 'field', state: 'ready', content: { title: bi('Your field', 'Tu campo'), summary: bi('Warmer than normal, soil drier than the regional normal, crop vigor moderate for peas at pod-fill.', 'Más cálido de lo normal, suelo más seco que la normal regional, vigor moderado para arveja en llenado de vaina.'), items: [] } },
  {
    key: 'soil', state: 'ready',
    content: {
      title: bi('Soil', 'Suelo'),
      summary: bi('Feed the soil after the peas: green manure and compost, and keep residue off the field edge.', 'Alimenta el suelo después de la arveja: abono verde y compost, y saca el rastrojo del borde del campo.'),
      items: [
        { title: bi('Green manure after harvest', 'Abono verde después de la cosecha'), body: bi('Oat with vetch for 10–12 weeks, cut and incorporated before flowering.', 'Avena con vicia por 10–12 semanas, cortada e incorporada antes de la floración.') },
        { title: bi('Trichoderma-enriched compost', 'Compost enriquecido con Trichoderma'), body: bi('Helps against root rots after a lettuce cycle; apply on moist soil.', 'Ayuda contra pudriciones de raíz tras la lechuga; aplicar con suelo húmedo.') },
        { title: bi('No residue piles at the field edge', 'Sin montones de rastrojo en el borde'), body: bi('Piles shelter aphids and leaf miners between seasons.', 'Los montones refugian pulgones y minadores entre temporadas.') },
      ],
    },
  },
  {
    key: 'pests', state: 'gaps', note: bi('needs a closer photo', 'necesita una foto más cercana'),
    content: {
      title: bi('Pest care', 'Manejo de plagas'),
      summary: bi('Possible aphids. The warm season adds about 1.8 extra generations. Beneficial insects: lacewings (Chrysoperla).', 'Posibles pulgones. La temporada cálida suma unas 1.8 generaciones extra. Insectos benéficos: crisopas (Chrysoperla).'),
      items: [
        { title: bi('Release lacewings (Chrysoperla)', 'Liberar crisopas (Chrysoperla)'), body: bi('Larvae eat aphids; release at the first colonies, early morning.', 'Las larvas comen pulgones; liberar con las primeras colonias, temprano.') },
        { title: bi('Beauveria bassiana', 'Beauveria bassiana'), body: bi('A beneficial fungus for soft-bodied insects; works best with humid mornings.', 'Hongo benéfico para insectos de cuerpo blando; funciona mejor con mañanas húmedas.') },
        { title: bi('Yellow sticky traps', 'Trampas amarillas'), body: bi('Count winged aphids weekly to time the release.', 'Cuenta pulgones alados cada semana para decidir la liberación.') },
      ],
    },
  },
  {
    key: 'nutrition', state: 'progress', note: bi('needs: last fertilizer used', 'falta: último fertilizante usado'),
    content: {
      title: bi('Nutrition', 'Nutrición'),
      summary: bi('Lettuce took a lot of nitrogen; peas fix part of their own. The next crop’s needs depend on your last fertilizer.', 'La lechuga consumió mucho nitrógeno; la arveja fija parte del suyo. Lo que necesita el siguiente cultivo depende de tu último fertilizante.'),
      items: [{ title: bi('Nitrogen after lettuce', 'Nitrógeno después de la lechuga'), body: bi('Leafy crops leave little nitrogen behind; a legume supplies part of it.', 'Los cultivos de hoja dejan poco nitrógeno; una leguminosa aporta parte.') }],
    },
  },
  {
    key: 'rotation', state: 'ready',
    content: {
      title: bi('Rotation', 'Rotación'),
      summary: bi('A green manure after the peas improves the soil most; swapping lettuce for maize breaks the aphid cycle best.', 'Un abono verde después de la arveja mejora más el suelo; cambiar la lechuga por maíz corta mejor el ciclo del pulgón.'),
      items: [],
      strategies: [
        {
          name: bi('Pea → lettuce', 'Arveja → lechuga'), current: true,
          timeline: ['pea', 'bare', 'lettuce', 'lettuce', 'lettuce', 'bare', 'pea', 'pea', 'pea', 'pea', 'pea', 'bare'],
          scores: { soil: 2, water: 3, pest_break: 2, income: 3, risk: 3 },
          why: bi('What you do today: two short cycles a year with a known market.', 'Lo que haces hoy: dos ciclos cortos al año con mercado conocido.'),
          evidence: bi('Aphid generations above normal (+1.8, NASA POWER degree-days); soil drier than normal (SMAP, regional).', 'Generaciones de pulgón sobre lo normal (+1.8, grados-día NASA POWER); suelo más seco de lo normal (SMAP, regional).'),
          gaps: bi('Last fertilizer and irrigation schedule unknown.', 'Se desconoce el último fertilizante y el riego.'),
          risks: bi('Lettuce and peas share aphids, so the pest never gets a break.', 'Lechuga y arveja comparten pulgones, así que la plaga nunca descansa.'),
        },
        {
          name: bi('A · swap lettuce for maize', 'A · cambiar lechuga por maíz'),
          timeline: ['pea', 'maize', 'maize', 'maize', 'maize', 'maize', 'bare', 'pea', 'pea', 'pea', 'pea', 'pea'],
          scores: { soil: 3, water: 2, pest_break: 4, income: 3, risk: 3 },
          why: bi('A cereal does not host pea aphids and uses the nitrogen the peas leave.', 'Un cereal no hospeda al pulgón de la arveja y usa el nitrógeno que deja la arveja.'),
          evidence: bi('Warm season favours maize; water balance −42 mm in 30 days means more irrigation.', 'La temporada cálida favorece al maíz; balance hídrico −42 mm en 30 días implica más riego.'),
          gaps: bi('Water allocation from the irrigation committee.', 'Dotación de agua de la junta de regantes.'),
          risks: bi('Higher water use in summer; fall armyworm in maize.', 'Mayor uso de agua en verano; gusano cogollero en el maíz.'),
        },
        {
          name: bi('B · green manure', 'B · abono verde'), best_for: bi('Best fit: improve soil', 'Mejor opción: mejorar el suelo'),
          timeline: ['pea', 'gm', 'gm', 'gm', 'lettuce', 'lettuce', 'lettuce', 'bare', 'pea', 'pea', 'pea', 'pea'],
          scores: { soil: 5, water: 3, pest_break: 4, income: 2, risk: 2 },
          why: bi('Oat with vetch rebuilds organic matter and breaks the aphid bridge between crops.', 'Avena con vicia recupera materia orgánica y corta el puente del pulgón entre cultivos.'),
          evidence: bi('Crop vigor 0.48 (Sentinel-2, field-level) and soil drier than normal point to low organic matter.', 'Vigor 0.48 (Sentinel-2, a nivel de campo) y suelo más seco de lo normal sugieren poca materia orgánica.'),
          gaps: bi('No soil test on file; seed availability for vetch.', 'No hay análisis de suelo; disponibilidad de semilla de vicia.'),
          risks: bi('One fewer cash crop this year.', 'Un cultivo comercial menos este año.'),
        },
      ],
    },
  },
  { key: 'sheet', state: 'ready' },
];

const MSG: { role: ChatMessage['role']; body: Bi; photos?: { url: string; tag?: string }[] }[] = [
  { role: 'assistant', body: bi('Hello. I am your field consultant. What are you growing now, and what grew here before?', 'Hola. Soy tu consultor de campo. ¿Qué siembras ahora y qué hubo antes aquí?') },
  { role: 'user', body: bi('Peas now. Lettuce before.', 'Arveja ahora. Lechuga antes.') },
  {
    role: 'user', body: bi('Here are my plants.', 'Aquí están mis plantas.'),
    photos: [{ url: '/img/pea-plant.jpg' }, { url: '/img/pea-leaf.jpg' }, { url: '/img/pea-leaf.jpg', tag: 'blurry' }, { url: '/img/pea-plant.jpg' }],
  },
  { role: 'assistant', body: bi('Thanks. Photo 3 is blurry. Take the underside of a leaf, closer.', 'Gracias. La foto 3 está borrosa. Toma el envés de una hoja, más cerca.') },
  { role: 'status', body: bi('Read NASA POWER · SMAP · Sentinel-2 for your field', 'NASA POWER · SMAP · Sentinel-2 leídos para tu campo') },
];

export function demoMessages(lang: Lang): ChatMessage[] {
  return MSG.map((m, i) => ({ id: i + 1, role: m.role, body: m.body[lang], photos: m.photos, created_at: '2026-11-13T14:0' + i + ':00Z' }));
}

export function demoAnalysis(lang: Lang): AnalysisState {
  return {
    id: DEMO_ID, lang, location_mode: 'example', example_site: 'huaral', field: HUARAL_FIELD,
    crop_current: { key: 'pea', names: bi('Peas', 'Arveja'), scientific: 'Pisum sativum', family: 'Fabaceae', crop_type: 'annual', origin: 'seed' },
    crop_history: [{ key: 'lettuce', names: bi('Lettuce', 'Lechuga'), scientific: 'Lactuca sativa', family: 'Asteraceae', crop_type: 'annual', origin: 'seed' }],
    messages: demoMessages(lang),
    sections: structuredClone(sections),
    evidence: DEMO_EVIDENCE,
    demo_mode: true,
    prediagnosis: { headline: bi('Likely water stress with early aphid pressure', 'Probable estrés hídrico con presión temprana de pulgones'), confidence: 'medium' },
  };
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
