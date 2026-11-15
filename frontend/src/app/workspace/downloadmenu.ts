// "Download" menu in the top bar: technical sheet, data (CSV), package (ZIP) and share link, always one click away.
// The items carry data-export, so the delegated handler in export.ts (wireExports) runs them.
import type { ClientAnalysis } from '../../shared/api';
import { esc } from '../../shared/dom';
import { t } from '../../shared/i18n';
import { icon } from '../../shared/icons';
import './downloadmenu.css';

export function downloadMenuHtml(): string {
  return `<div class="dlm" id="dlm">
    <button type="button" class="pill dlm__btn" id="dlm-btn" aria-haspopup="menu" aria-expanded="false" aria-controls="dlm-list">
      ${icon('download')}<span data-i18n="dl_menu">${esc(t('dl_menu'))}</span><span class="sr-only dlm__next"></span></button>
    <div class="dlm__list glass" id="dlm-list" role="menu" aria-labelledby="dlm-btn" hidden></div>
  </div>`;
}

function items(a: ClientAnalysis | null): string {
  const row = (kind: string, ic: string, title: string, sub: string) =>
    `<button type="button" role="menuitem" class="dlm__item" data-export="${kind}" tabindex="-1">${icon(ic)}<span><b>${esc(t(title as never))}</b><small>${esc(t(sub as never))}</small></span></button>`;
  return [
    row('sheet', 'sheet', 'ex_sheet', 'dl_sheet_sub'),
    row('csv', 'csv', 'ex_csv', 'dl_csv_sub'),
    a?.offline ? '' : row('zip', 'download', 'ex_zip', 'dl_zip_sub'),
    a?.readOnly ? '' : row('share', 'link', 'ex_share', 'dl_share_sub'),
  ].join('');
}

/** Opens/closes the menu, keyboard navigation (arrows, Home/End, Esc), closes on outside click or after a choice. */
export function wireDownloadMenu(root: HTMLElement, get: () => ClientAnalysis | null): void {
  const btn = root.querySelector<HTMLButtonElement>('#dlm-btn');
  const list = root.querySelector<HTMLElement>('#dlm-list');
  if (!btn || !list) return;
  const entries = () => [...list.querySelectorAll<HTMLButtonElement>('[role=menuitem]')];
  const close = (focusBtn = false) => { list.hidden = true; btn.setAttribute('aria-expanded', 'false'); if (focusBtn) btn.focus(); };
  const open = () => {
    list.innerHTML = items(get());
    list.hidden = false; btn.setAttribute('aria-expanded', 'true');
    entries()[0]?.focus();
  };
  btn.addEventListener('click', () => (list.hidden ? open() : close()));
  btn.addEventListener('keydown', (e) => { if (e.key === 'ArrowDown') { e.preventDefault(); open(); } });
  list.addEventListener('keydown', (e) => {
    const all = entries(); const i = all.indexOf(document.activeElement as HTMLButtonElement);
    const go = (k: number) => { e.preventDefault(); all[(k + all.length) % all.length]?.focus(); };
    if (e.key === 'ArrowDown') go(i + 1); else if (e.key === 'ArrowUp') go(i - 1);
    else if (e.key === 'Home') go(0); else if (e.key === 'End') go(all.length - 1);
    else if (e.key === 'Escape') { e.preventDefault(); close(true); }
    else if (e.key === 'Tab') close();
  });
  // The export runs through the delegated handler; close right after the click bubbles.
  list.addEventListener('click', (e) => { if ((e.target as HTMLElement).closest('[data-export]')) setTimeout(() => close(), 0); });
  document.addEventListener('click', (e) => { if (!list.hidden && !(e.target as HTMLElement).closest('#dlm')) close(); });
}
