// Export buttons: technical sheet (backend HTML, print to PDF), CSV of all series, share link to the clipboard.
import { csvUrl, evidenceCsv, share, sheetUrl, type ClientAnalysis } from '../../shared/api';
import { esc, toast } from '../../shared/dom';
import { lang, t } from '../../shared/i18n';
import { icon } from '../../shared/icons';

export function exportButtons(a: ClientAnalysis, big = false): string {
  return `<div class="exports${big ? ' exports--big' : ''}">
    <button type="button" class="btn ${big ? 'btn--solid' : 'btn--glass btn--sm'}" data-export="sheet">${icon('sheet')}${esc(t(big ? 'ex_sheet_long' : 'ex_sheet'))}</button>
    <button type="button" class="btn btn--glass btn--sm" data-export="csv">${icon('csv')}${esc(t('ex_csv'))}</button>
    ${a.readOnly ? '' : `<button type="button" class="btn btn--glass btn--sm" data-export="share">${icon('link')}${esc(t('ex_share'))}</button>`}
  </div>`;
}

function download(name: string, text: string) {
  const url = URL.createObjectURL(new Blob(['﻿' + text], { type: 'text/csv;charset=utf-8' }));
  const a = document.createElement('a');
  a.href = url; a.download = name; document.body.append(a); a.click(); a.remove();
  setTimeout(() => URL.revokeObjectURL(url), 2000);
}

async function copy(text: string) {
  try { await navigator.clipboard.writeText(text); toast(t('copied')); }
  catch { toast(t('copy_failed', { url: text }), 8000); }
}

/** Delegated handler: call once on a root element. */
export function wireExports(root: HTMLElement, get: () => ClientAnalysis | null): void {
  root.addEventListener('click', async (e) => {
    const b = (e.target as HTMLElement).closest<HTMLButtonElement>('[data-export]');
    const a = get();
    if (!b || !a) return;
    const kind = b.dataset.export;
    if (kind === 'sheet') {
      if (a.offline) { toast(t('ex_sheet_offline'), 6000); return; }
      window.open(sheetUrl(a.id, lang()), '_blank', 'noopener');
    } else if (kind === 'csv') {
      if (a.offline && a.evidence) download('field-shift-huaral-DEMO.csv', evidenceCsv(a.evidence));
      else window.open(csvUrl(a.id), '_blank', 'noopener');
    } else if (kind === 'share') {
      b.disabled = true;
      try { await copy(await share(a)); } catch { toast(t('ws_send_err')); } finally { b.disabled = false; }
    }
  });
}
