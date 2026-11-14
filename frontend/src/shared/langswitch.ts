// EN | ES switch markup and wiring; keeps aria-pressed in sync with the current language.
import type { Lang } from './contract';
import { lang, onLang, setLang, t } from './i18n';

export function langSwitchHtml(): string {
  return `<div class="lang" role="group" aria-label="${t('lang_label')}" data-i18n-attr="aria-label:lang_label">
    <button type="button" data-lang="en" lang="en" aria-label="English">EN</button><span aria-hidden="true">|</span><button type="button" data-lang="es" lang="es" aria-label="Español">ES</button></div>`;
}

export function wireLangSwitch(root: ParentNode = document): void {
  const sync = () => root.querySelectorAll<HTMLButtonElement>('[data-lang]').forEach((b) => b.setAttribute('aria-pressed', String(b.dataset.lang === lang())));
  root.querySelectorAll<HTMLButtonElement>('[data-lang]').forEach((b) => b.addEventListener('click', () => setLang(b.dataset.lang as Lang)));
  onLang(sync);
  sync();
}
