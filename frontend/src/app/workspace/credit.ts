// The Copernicus licence sentence must stay verbatim (English); in Spanish a short gloss is shown next to it.
import { lang, t } from '../../shared/i18n';

export function copernicusGloss(credit: string): string {
  if (lang() !== 'es') return '';
  const m = credit.match(/Contains modified Copernicus Sentinel data(?:\s+(\d{4}))?/i);
  if (!m) return '';
  const by = credit.match(/processed by ([^·,(]+)/i);
  return t('cop_gloss', { y: m[1] ? ' ' + m[1] : '', by: by ? t('cop_by', { x: by[1].trim() }) : '' });
}
