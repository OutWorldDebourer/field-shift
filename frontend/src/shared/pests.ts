// Pest names in the UI language. The backend sends `name` as a bilingual object (iteration 2) or, in older
// packages, as an English string; the local map covers the pests the POWER degree-day models track.
import type { Bi } from './contract';
import { lang, pick } from './i18n';

const ES: Record<string, string> = {
  'green peach aphid': 'pulgón verde del duraznero',
  'sweetpotato / silverleaf whitefly': 'mosca blanca',
  'sweetpotato whitefly': 'mosca blanca',
  'silverleaf whitefly': 'mosca blanca',
  'fall armyworm': 'gusano cogollero',
  'two-spotted spider mite': 'arañita roja',
  'spider mite': 'arañita roja',
  'pea leafminer': 'mosca minadora',
  'leafminer': 'mosca minadora',
};

export type PestName = string | Bi | null | undefined;

/** The pest's common name in the current language. */
export function pestName(n: PestName): string {
  if (!n) return '';
  if (typeof n === 'object') return pick(n);
  if (lang() === 'es') return ES[n.trim().toLowerCase()] ?? n;
  return n;
}

/** A pest row's name: `name_bi` (iteration 2) when present, else `name`. */
type Row = { name: PestName; name_bi?: Bi | null };
const nameOf = (n: PestName | Row): PestName => (n && typeof n === 'object' && 'name' in n ? ((n as Row).name_bi ?? (n as Row).name) : (n as PestName));

/** Capitalised for the start of a line. Accepts a name or a whole pest row. */
export const pestTitle = (n: PestName | Row): string => { const s = pestName(nameOf(n)); return s.charAt(0).toUpperCase() + s.slice(1); };
export const pestLower = (n: PestName | Row): string => pestName(nameOf(n));
