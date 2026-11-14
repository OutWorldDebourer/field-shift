// "What you told us": the facts the consultant has recorded (AnalysisState.facts, an IntakeForm), as a short
// labelled list in the "Your field" section card, so the farmer can check what was understood.
import type { ClientAnalysis } from '../../shared/api';
import type { IntakeForm } from '../../shared/contract';
import { esc } from '../../shared/dom';
import { t } from '../../shared/i18n';
import { GROUPS, isUnknown } from './intake';

export function factsHtml(a: ClientAnalysis): string {
  const f: IntakeForm = a.facts ?? {};
  const rows = GROUPS.flatMap((g) => g.fields).map((fd) => {
    const v = f[fd.k];
    const list = Array.isArray(v) ? v.filter(Boolean) : v ? [v] : [];
    if (!list.length) return '';
    const text = list.map((x) => (isUnknown(x) ? t('in_dk') : x)).join(', ');
    return `<div><dt>${esc(t(fd.label))}</dt><dd>${esc(text)}</dd></div>`;
  }).filter(Boolean);
  if (!rows.length) return '';
  return `<div class="facts"><h4 class="mono">${esc(t('facts_t'))}</h4><dl>${rows.join('')}</dl></div>`;
}
