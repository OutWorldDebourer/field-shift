// Bundled bilingual glossary (used when GET /api/glossary does not answer) and the map from reading ids to terms.
import type { GlossaryTerm } from './contract';

const g = (key: string, en: string, es: string, sen: string, ses: string, scale?: [string, string]): GlossaryTerm =>
  ({ key, term: { en, es }, short: { en: sen, es: ses }, ...(scale ? { scale: { en: scale[0], es: scale[1] } } : {}) });

export const GLOSSARY: GlossaryTerm[] = [
  g('ndvi', 'Crop vigor (NDVI)', 'Vigor del cultivo (NDVI)',
    'How green and dense the plants look from space, from the red and near-infrared light they reflect. It shows vigor, not the cause of a problem.',
    'Qué tan verdes y densas se ven las plantas desde el espacio, según la luz roja e infrarroja que reflejan. Muestra el vigor, no la causa de un problema.',
    ['0 to 1: below 0.2 bare soil · 0.2–0.4 sparse · 0.4–0.6 moderate · above 0.6 dense', '0 a 1: menos de 0,2 suelo desnudo · 0,2–0,4 escaso · 0,4–0,6 moderado · más de 0,6 denso']),
  g('normal', 'Temperature vs normal', 'Temperatura vs. normal',
    'How much warmer or cooler the last 30 days were than the same days in a 40-year normal at this place.',
    'Cuánto más cálidos o fríos fueron los últimos 30 días frente a los mismos días de una normal de 40 años en este lugar.'),
  g('et0', 'ET₀ (reference evapotranspiration)', 'ET₀ (evapotranspiración de referencia)',
    'The water a well-watered reference grass would lose to the air through soil and leaves, in mm. Hot, dry, windy and sunny days raise it.',
    'El agua que perdería al aire un pasto de referencia bien regado, por el suelo y las hojas, en mm. Los días calurosos, secos, ventosos y soleados la aumentan.'),
  g('water_balance', 'Water balance', 'Balance hídrico',
    'Rain minus crop water demand (ET₀) over the period. A negative number is the water irrigation has to supply.',
    'Lluvia menos la demanda de agua del cultivo (ET₀) en el periodo. Un número negativo es el agua que debe aportar el riego.'),
  g('gdd', 'Growing degree-days', 'Grados-día de crecimiento',
    'Heat accumulated above the temperature at which a plant or insect starts to develop. More degree-days mean faster development.',
    'Calor acumulado por encima de la temperatura a la que una planta o un insecto empieza a desarrollarse. Más grados-día significan un desarrollo más rápido.'),
  g('pest_gen', 'Pest generations', 'Generaciones de plagas',
    'How many life cycles a pest can complete with this season’s heat, from published degree-day thresholds, compared with a normal season. A model of risk, not a count of insects.',
    'Cuántos ciclos de vida puede completar una plaga con el calor de esta temporada, según umbrales de grados-día publicados, frente a una temporada normal. Un modelo de riesgo, no un conteo de insectos.'),
  g('regional', 'Regional context vs field-level', 'Contexto regional vs. a nivel de campo',
    'Field-level readings have pixels small enough to see your field (10–30 m). Regional context comes from pixels kilometres wide: the area around you, not your field alone.',
    'Las lecturas a nivel de campo tienen píxeles tan pequeños que ven tu campo (10–30 m). El contexto regional viene de píxeles de kilómetros: la zona alrededor, no solo tu campo.'),
  g('provisional', 'Provisional', 'Provisional',
    'The latest weeks come from near-real-time data streams. They can change slightly when the final data are published.',
    'Las últimas semanas vienen de datos casi en tiempo real. Pueden cambiar un poco cuando se publiquen los datos finales.'),
  g('power', 'NASA POWER', 'NASA POWER',
    'NASA’s daily weather and solar data for any point on Earth since 1981 (about 50 km cells): temperature, rain, humidity, wind, sunlight.',
    'Datos diarios de clima y radiación solar de la NASA para cualquier punto de la Tierra desde 1981 (celdas de unos 50 km): temperatura, lluvia, humedad, viento, sol.'),
  g('smap', 'SMAP', 'SMAP',
    'NASA satellite that measures moisture in the top layer of soil (model product about 9 km). It describes the area, not your furrows.',
    'Satélite de la NASA que mide la humedad de la capa superior del suelo (producto de modelo de unos 9 km). Describe la zona, no tus surcos.'),
  g('imerg', 'GPM IMERG', 'GPM IMERG',
    'Rainfall estimated every half hour from a constellation of satellites (about 10 km).',
    'Lluvia estimada cada media hora con una constelación de satélites (unos 10 km).'),
  g('grace', 'GRACE-FO', 'GRACE-FO',
    'Twin satellites that weigh changes in water stored underground and on the surface over large regions (hundreds of km).',
    'Satélites gemelos que pesan los cambios del agua almacenada bajo tierra y en la superficie en grandes regiones (cientos de km).'),
  g('nexgddp', 'NEX-GDDP-CMIP6', 'NEX-GDDP-CMIP6',
    'NASA’s downscaled climate model projections (about 25 km). We show the median and the range of several models.',
    'Proyecciones de modelos climáticos de la NASA a mayor detalle (unos 25 km). Mostramos la mediana y el rango de varios modelos.'),
  g('ssp', 'SSP2-4.5', 'SSP2-4.5',
    'A middle-of-the-road emissions scenario used by climate models: neither the best nor the worst case.',
    'Un escenario intermedio de emisiones que usan los modelos climáticos: ni el mejor ni el peor caso.'),
  g('geosfp', 'GEOS-FP forecast', 'Pronóstico GEOS-FP',
    'NASA’s global weather forecast model (about 25 km), updated several times a day. Regional context for the next days.',
    'El modelo global de pronóstico del tiempo de la NASA (unos 25 km), actualizado varias veces al día. Contexto regional para los próximos días.'),
  g('outlook', 'Season outlook', 'Perspectiva de temporada',
    'Our estimate for the coming months from the 40-year normal, in two scenarios: if the current warmth persists, or if it returns to normal. Not a forecast.',
    'Nuestra estimación para los próximos meses a partir de la normal de 40 años, en dos escenarios: si el calor actual sigue o si vuelve a lo normal. No es un pronóstico.'),
  g('sentinel2', 'Sentinel-2 / Landsat', 'Sentinel-2 / Landsat',
    'Optical satellites that photograph every field every few days at 10–30 m. We use the clear images to measure crop vigor.',
    'Satélites ópticos que fotografían cada campo cada pocos días a 10–30 m. Usamos las imágenes despejadas para medir el vigor del cultivo.'),
  g('nasadem', 'NASADEM', 'NASADEM',
    'NASA’s elevation model of the land surface (30 m), used for slope and runoff risk.',
    'El modelo de elevación del terreno de la NASA (30 m), usado para la pendiente y el riesgo de escorrentía.'),
];

/** Glossary term for a reading id. */
export function termFor(readingId: string): string {
  if (readingId.startsWith('temp')) return 'normal';
  if (readingId.startsWith('soil')) return 'smap';
  if (readingId.startsWith('rain')) return 'imerg';
  if (readingId.startsWith('ndvi') || readingId.startsWith('vigor')) return 'ndvi';
  if (readingId.startsWith('water')) return 'water_balance';
  if (readingId.startsWith('et0')) return 'et0';
  if (readingId.startsWith('pest')) return 'pest_gen';
  if (readingId === 'slope') return 'nasadem';
  if (readingId === 'grace') return 'grace';
  if (readingId === 'projection') return 'nexgddp';
  return 'regional';
}

/** Backend terms first, then any bundled term the backend did not send. */
export function mergeGlossary(remote: GlossaryTerm[] | null): GlossaryTerm[] {
  if (!remote?.length) return GLOSSARY;
  const keys = new Set(remote.map((x) => x.key));
  return [...remote, ...GLOSSARY.filter((x) => !keys.has(x.key))];
}
