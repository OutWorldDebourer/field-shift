// Spanish for the few English fragments the backend still sends inside units, source lines and projection
// labels (e.g. "°C vs normal", "generations", "published threshold"). Proper names stay as they are.
import { lang } from './i18n';

const ES: [RegExp, string][] = [
  [/\bvs normal\b/gi, 'vs. normal'],
  [/\bgenerations\b/gi, 'generaciones'],
  [/\bgeneration\b/gi, 'generación'],
  [/\bpublished thresholds?\b/gi, 'umbral publicado'],
  [/\bcentred on\b|\bcentered on\b/gi, 'centrado en'],
  [/\bhistorical experiment\b/gi, 'experimento histórico'],
  [/\bdegree-days\b/gi, 'grados-día'],
  [/\bdays\b/gi, 'días'],
  [/\bmodels\b/gi, 'modelos'],
  [/\babout\b/gi, 'unos'],
  [/\bgrid cell\b/gi, 'celda'],
  [/\bfield average\b/gi, 'promedio del campo'],
];

export function loc(s: string | null | undefined): string {
  if (!s) return '';
  if (lang() !== 'es') return s;
  return ES.reduce((x, [re, to]) => x.replace(re, to), s);
}
