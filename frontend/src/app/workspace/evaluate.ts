// "Evaluate your idea" in Rotation (iteration 4, phase C): the farmer types any crop and gets a fourth column,
// "Your proposal", scored by the same engine (POST /api/analyses/:id/rotation/evaluate). Crop suggestions while
// typing, never a restriction. AI research runs only for a crop outside the knowledge base (it can take a minute).
import { evaluateCrop, suggestCrops, type ClientAnalysis } from '../../shared/api';
import { esc, toast } from '../../shared/dom';
import { pick, t } from '../../shared/i18n';
import { icon } from '../../shared/icons';
import { debounce } from '../field/search';

export function evaluateFormHtml(a: ClientAnalysis, current = ''): string {
  if (a.readOnly || !a.crop_current) return '';
  return `<form class="rot__eval glass" data-eval novalidate>
    <label for="eval-crop" class="serif rot__eval-t">${esc(t('ev_title'))}</label>
    <p class="rot__eval-h">${esc(t('ev_hint'))}</p>
    <div class="rot__eval-row"><input id="eval-crop" list="eval-list" autocomplete="off" placeholder="${esc(t('ev_ph'))}" maxlength="80" value="${esc(current)}">
      <datalist id="eval-list"></datalist>
      <button type="submit" class="btn btn--solid btn--sm">${icon('rotation')}<span>${esc(t(current ? 'ev_again' : 'ev_go'))}</span></button></div>
    <p class="rot__eval-s mono" data-eval-status aria-live="polite"></p>
  </form>`;
}

/** Wires the form after the Rotation view renders. `apply` receives the analysis with the new column. */
export function wireEvaluate(root: HTMLElement, getA: () => ClientAnalysis, apply: (next: ClientAnalysis) => void): void {
  const form = root.querySelector<HTMLFormElement>('[data-eval]');
  if (!form) return;
  const input = form.querySelector<HTMLInputElement>('input')!, list = form.querySelector<HTMLDataListElement>('datalist')!;
  const status = form.querySelector<HTMLElement>('[data-eval-status]')!, btn = form.querySelector<HTMLButtonElement>('button')!;
  input.addEventListener('input', debounce(async () => {
    const s = await suggestCrops(input.value).catch(() => []);
    list.innerHTML = s.map((c) => `<option value="${esc(pick(c.label))}">${esc(c.scientific ?? '')}</option>`).join('');
  }, 250));
  form.addEventListener('submit', async (e) => {
    e.preventDefault();
    const crop = input.value.trim();
    const a = getA();
    if (!crop || !a) { input.focus(); return; }
    if (a.offline) { toast(t('in_offline'), 5000); return; }
    btn.disabled = true; input.disabled = true;
    status.textContent = t('ev_busy', { c: crop });
    try {
      const next = await evaluateCrop(a, crop);
      apply(next);
      toast(t('ev_done', { c: crop }), 4000);
      // The result can sit screens below the form on a phone: bring the new column into view.
      setTimeout(() => {
        const card = document.querySelector<HTMLElement>('.strat.is-proposal');
        card?.scrollIntoView({ block: 'start', behavior: matchMedia('(prefers-reduced-motion: reduce)').matches ? 'auto' : 'smooth' });
        const h = card?.querySelector<HTMLElement>('h3'); h?.setAttribute('tabindex', '-1'); h?.focus({ preventScroll: true });
      }, 80);
    } catch {
      status.textContent = t('ev_err');
      btn.disabled = false; input.disabled = false;
    }
  });
}
