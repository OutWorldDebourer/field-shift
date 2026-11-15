// Quick replies under the consultant's last message (iteration 4): draft answers to the active topic plus "I don't
// know"; once every topic is covered, the recommended follow-ups (forecast, water balance, the crop's own pests). A click only fills the composer, so
// the farmer can edit before sending (no paid turn is started by a suggestion).
import type { ClientAnalysis } from '../../shared/api';
import { esc } from '../../shared/dom';
import { pick, t } from '../../shared/i18n';
import { icon } from '../../shared/icons';
import { forecastNow, outlookNow } from './forecast';
import { stepsOf, TITLE } from './progress';
import { topicsFor } from './topics';
import type { StrKey } from '../../shared/strings';

type QR = { label: string; text: string };

/** Draft answers per topic (the farmer edits the blanks before sending). */
const DRAFTS: Record<string, StrKey[]> = {
  field: ['qr_field_1', 'qr_field_2'], soil: ['qr_soil_1', 'qr_soil_2'], pests: ['qr_pests_1', 'qr_pests_2'],
  nutrition: ['qr_nut_1', 'qr_nut_2'], rotation: ['qr_rot_1', 'qr_rot_2'],
};

export function quickReplies(a: ClientAnalysis): QR[] {
  const active = stepsOf(a).find((s) => s.state === 'active' && s.key !== 'sheet');
  if (active) {
    const topic = pick(active.title) || t(TITLE[active.key]);
    const drafts = (DRAFTS[active.key] ?? []).map((k) => ({ label: t(k), text: t(k) }));
    return [...drafts, { label: t('qr_dk'), text: t('qr_dk_text', { t: topic.toLowerCase() }) }].slice(0, 3);
  }
  // Every topic covered: the recommended follow-ups (forecast, water balance, the crop's own pests).
  return a.evidence ? topicsFor(a.evidence, forecastNow(a), outlookNow(a)).slice(0, 3).map((x) => ({ label: x.label, text: x.prompt })) : [];
}

export function quickRepliesHtml(a: ClientAnalysis): string {
  if (a.readOnly) return '';
  const last = [...a.messages].reverse().find((m) => m.role === 'user' || m.role === 'assistant');
  if (!last || last.role !== 'assistant') return '';
  const list = quickReplies(a);
  if (!list.length) return '';
  const active = stepsOf(a).some((s) => s.state === 'active' && s.key !== 'sheet');
  return `<li class="qr" aria-label="${esc(t('qr_label'))}"><ul class="qr__list">${list.map((q) =>
    `<li><button type="button" class="qr__b" data-qr="${esc(q.text)}">${esc(q.label)}</button></li>`).join('')}</ul>
    ${active ? `<button type="button" class="qr__form" data-intake aria-haspopup="dialog">${icon('clipboard')}<span>${esc(t('qr_form'))}</span></button>` : ''}</li>`;
}
