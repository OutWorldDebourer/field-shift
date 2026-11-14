// Export buttons: technical sheet (backend HTML in the same tab, it links back to the analysis), CSV of all series,
// the download package (ZIP: sheet + data + original photos + satellite image) and a share link to the clipboard.
import { csvUrl, evidenceCsv, packageUrl, share, sheetUrl, type ClientAnalysis } from '../../shared/api';
import { esc, toast } from '../../shared/dom';
import { lang, t } from '../../shared/i18n';
import { icon } from '../../shared/icons';

export function exportButtons(a: ClientAnalysis, big = false): string {
  return `<div class="exports${big ? ' exports--big' : ''}">
    <button type="button" class="btn ${big ? 'btn--solid' : 'btn--glass btn--sm'}" data-export="sheet">${icon('sheet')}${esc(t(big ? 'ex_sheet_long' : 'ex_sheet'))}</button>
    <button type="button" class="btn btn--glass btn--sm" data-export="csv">${icon('csv')}${esc(t('ex_csv'))}</button>
    ${a.offline ? '' : `<button type="button" class="btn btn--glass btn--sm" data-export="zip">${icon('download')}${esc(t('ex_zip'))}</button>`}
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

/** Check the package exists before navigating, so a missing endpoint never replaces the app with an error page. */
async function downloadZip(a: ClientAnalysis, b: HTMLButtonElement) {
  const url = packageUrl(a.id);
  b.disabled = true;
  try {
    const r = await fetch(url, { method: 'HEAD' });
    if (!r.ok || /json|html/.test(r.headers.get('content-type') ?? '')) throw new Error(String(r.status));
    const el = document.createElement('a');
    el.href = url; el.download = `field-shift-${a.id.slice(0, 8)}.zip`; document.body.append(el); el.click(); el.remove();
  } catch { toast(t('ex_zip_na'), 6000); }
  finally { b.disabled = false; }
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
      location.assign(sheetUrl(a.id, lang()));
    } else if (kind === 'csv') {
      if (a.offline && a.evidence) download('field-shift-huaral-DEMO.csv', evidenceCsv(a.evidence));
      else window.open(csvUrl(a.id), '_blank', 'noopener');
    } else if (kind === 'zip') {
      await downloadZip(a, b);
    } else if (kind === 'share') {
      b.disabled = true;
      try { await copy(await share(a)); } catch { toast(t('ws_send_err')); } finally { b.disabled = false; }
    }
  });
}
