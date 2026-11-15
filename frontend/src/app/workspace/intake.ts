// "Load information": a glass side sheet (dialog, focus-trapped, Esc closes) with the IntakeForm fields grouped
// by topic. Submit → POST /api/analyses/:id/intake → the analysis comes back with the topics it covers completed.
// Example sites open it pre-filled with the labelled example answers (and once automatically per analysis), and
// the chat shows a card that opens it.
import { exampleIntake, submitIntake, suggestCrops, type ClientAnalysis, type CropSuggestion } from '../../shared/api';
import type { IntakeForm, SectionKey, Step } from '../../shared/contract';
import { esc, toast } from '../../shared/dom';
import { lang, onLang, pick, t } from '../../shared/i18n';
import { icon, type IconName } from '../../shared/icons';
import type { StrKey } from '../../shared/strings';
import { debounce } from '../field/search';
import { allDone, newlyDone, stepsOf, TITLE } from './progress';
import { SEC_ICON } from './rail';
import './intake.css';

type TextKey = Exclude<keyof IntakeForm, 'crops_before'>;
type Field = { k: keyof IntakeForm; label: StrKey; ph: StrKey; kind?: 'crop' | 'chips' | 'long' };
export const GROUPS: { key: SectionKey; fields: Field[] }[] = [
  { key: 'field', fields: [
    { k: 'crop_now', label: 'in_crop_now', ph: 'in_crop_now_ph', kind: 'crop' },
    { k: 'crops_before', label: 'in_crops_before', ph: 'in_crops_before_ph', kind: 'chips' },
    { k: 'irrigation', label: 'in_irrigation', ph: 'in_irrigation_ph' }] },
  { key: 'soil', fields: [
    { k: 'soil_type', label: 'in_soil_type', ph: 'in_soil_type_ph' },
    { k: 'organic_matter', label: 'in_organic_matter', ph: 'in_organic_matter_ph' },
    { k: 'residues', label: 'in_residues', ph: 'in_residues_ph' }] },
  { key: 'pests', fields: [
    { k: 'pests_seen', label: 'in_pests_seen', ph: 'in_pests_seen_ph', kind: 'long' },
    { k: 'products_recent', label: 'in_products_recent', ph: 'in_products_recent_ph' }] },
  { key: 'nutrition', fields: [
    { k: 'last_fertilizer', label: 'in_last_fertilizer', ph: 'in_last_fertilizer_ph' },
    { k: 'soil_test', label: 'in_soil_test', ph: 'in_soil_test_ph' }] },
  { key: 'rotation', fields: [
    { k: 'next_crop', label: 'in_next_crop', ph: 'in_next_crop_ph', kind: 'crop' },
    { k: 'harvest_use', label: 'in_harvest_use', ph: 'in_harvest_use_ph' }] },
];

const UNKNOWN = 'unknown';
export const isUnknown = (v: string | undefined): boolean => !!v && /^(unknown|no s[eé]|i don'?t know|desconocido)$/i.test(v.trim());

export const isExample = (a: ClientAnalysis): boolean => !!a.example_site || a.location_mode === 'example';

/** Chat card for example sites until every topic is done. */
export function exampleCardHtml(a: ClientAnalysis, steps: Step[], inline = false): string {
  if (!isExample(a) || a.readOnly || allDone(steps)) return '';
  const body = `<div class="excard glass"><span class="excard__ico" aria-hidden="true">${icon('clipboard')}</span>
    <p class="excard__t">${esc(t('in_card_t'))}</p>
    <button type="button" class="btn btn--solid btn--sm" data-intake aria-haspopup="dialog">${icon('clipboard')}${esc(t('in_card_btn'))}</button></div>`;
  return inline ? body : `<li class="msg chat__excard">${body}</li>`;
}

const autoKey = (id: string) => `fs-intake-auto:${id}`;

export interface Intake {
  open(from?: HTMLElement | null): void; autoOpen(): void; close(): void;
  /** The first step of a new analysis: one form for everything, no model call. `onSkip` runs when the farmer
   *  prefers to chat (skip button, close or Esc); `onPhotos` receives the optional photos after a submit. */
  openFirst(onSkip: () => void, onPhotos: (files: File[]) => void): void;
}

/** A new analysis: no consultant message and no topic done yet. */
export const isFresh = (a: ClientAnalysis): boolean =>
  !a.readOnly && !a.offline && !a.messages.some((m) => m.role === 'assistant') && !stepsOf(a).some((s) => s.state === 'done');

export function mountIntake(root: HTMLElement, getA: () => ClientAnalysis, apply: (next: ClientAnalysis) => void): Intake {
  root.insertAdjacentHTML('beforeend', `<div class="in-scrim" hidden></div><aside class="in glass" role="dialog" aria-modal="true" aria-labelledby="in-t" hidden></aside>`);
  const panel = root.querySelector<HTMLElement>('.in')!, scrim = root.querySelector<HTMLElement>('.in-scrim')!;
  let returnTo: HTMLElement | null = null;
  let values: IntakeForm = {};
  let example = false;
  let busy = false;
  let first: { onSkip: () => void; onPhotos: (files: File[]) => void } | null = null;
  let photos: File[] = [];

  // ---------- render ----------
  const fieldHtml = (f: Field): string => {
    const id = `in-${f.k}`;
    const v = values[f.k];
    const dk = Array.isArray(v) ? v.some(isUnknown) : isUnknown(v);
    const label = t(f.label);
    let control: string;
    if (f.kind === 'chips') {
      const chips = Array.isArray(v) && !dk ? v : [];
      control = `<div class="in-chips" data-chips>${chips.map((c) => chipHtml(c)).join('')}
        <input id="${id}" data-k="${f.k}" autocomplete="off" role="combobox" aria-expanded="false" aria-controls="${id}-list" aria-autocomplete="list" placeholder="${esc(t(f.ph))}"${dk ? ' disabled' : ''}></div>`;
    } else if (f.kind === 'long') {
      control = `<textarea id="${id}" data-k="${f.k}" rows="2" placeholder="${esc(t(f.ph))}"${dk ? ' disabled' : ''}>${esc(dk ? '' : (v as string) ?? '')}</textarea>`;
    } else {
      const combo = f.kind === 'crop' ? ` autocomplete="off" role="combobox" aria-expanded="false" aria-controls="${id}-list" aria-autocomplete="list"` : '';
      control = `<input id="${id}" data-k="${f.k}"${combo} value="${esc(dk ? '' : (v as string) ?? '')}" placeholder="${esc(t(f.ph))}"${dk ? ' disabled' : ''}>`;
    }
    const list = f.kind === 'crop' || f.kind === 'chips' ? `<ul id="${id}-list" class="crop__list" role="listbox" hidden></ul>` : '';
    return `<div class="in-f${dk ? ' is-dk' : ''}" data-f="${f.k}"${f.kind ? ` data-kind="${f.kind}"` : ''}>
      <div class="in-f__top"><label for="${id}">${esc(label)}</label>
        <button type="button" class="in-dk" data-dk="${f.k}" aria-pressed="${dk}" aria-label="${esc(t('in_dk_for', { x: label }))}">${esc(t('in_dk'))}</button></div>
      <div class="in-f__ctl">${control}${list}</div></div>`;
  };
  const chipHtml = (c: string) => `<span class="in-chip" data-chip="${esc(c)}">${esc(c)}<button type="button" data-unchip aria-label="${esc(t('in_remove_chip', { x: c }))}">${icon('close')}</button></span>`;

  function render() {
    const lede = (first ? `<p class="in__lede in__lede--first">${esc(t('in_first_lede'))}</p>` : '')
      + (example ? `<p class="in__banner">${icon('info')}<span>${esc(t('in_example'))}</span></p>` : first ? '' : `<p class="in__lede">${esc(t('in_lede'))}</p>`);
    const photoBox = first ? `<fieldset class="in-g in-g--photos"><legend class="in-g__t">${icon('camera')}<span class="serif">${esc(t('in_photos'))}</span></legend>
      <p class="in-f__hint">${esc(t('in_photos_hint'))}</p>
      <label class="btn btn--glass btn--sm in-photos__btn">${icon('camera')}<span>${esc(t('in_photos_add'))}</span><input type="file" accept="image/*" multiple data-in-photos hidden></label>
      <p class="in-photos__n mono" data-in-photos-n>${photos.length ? esc(t('in_photos_n', { n: photos.length })) : ''}</p></fieldset>` : '';
    panel.innerHTML = `<header class="in__head"><div><h2 class="serif" id="in-t">${esc(t('in_title'))}</h2></div>
        <button type="button" class="icon-btn" data-in-close aria-label="${esc(t('close'))}">${icon('close')}</button></header>
      <form class="in__form" novalidate>
        <div class="in__body">${lede}
          ${GROUPS.map((g) => `<fieldset class="in-g"><legend class="in-g__t">${icon(SEC_ICON[g.key] as IconName)}<span class="serif">${esc(t(TITLE[g.key]))}</span></legend>
            ${g.fields.map(fieldHtml).join('')}</fieldset>`).join('')}
          ${photoBox}
        </div>
        <footer class="in__foot"><p class="status-line in__busy" hidden><i class="tick"></i><span>${esc(t('in_busy'))}</span></p>
          <div class="in__acts"><button type="button" class="btn btn--glass btn--sm" data-in-close>${esc(t(first ? 'in_skip' : 'in_cancel'))}</button>
          <button type="submit" class="btn btn--solid btn--sm">${icon('check')}${esc(t('in_submit'))}</button></div></footer>
      </form>`;
    wireCombos();
  }

  // ---------- values ----------
  function collect(): IntakeForm {
    const out: IntakeForm = {};
    panel.querySelectorAll<HTMLElement>('.in-f').forEach((box) => {
      const k = box.dataset.f as keyof IntakeForm;
      const dk = box.classList.contains('is-dk');
      if (k === 'crops_before') {
        if (dk) { out.crops_before = [UNKNOWN]; return; }
        const chips = [...box.querySelectorAll<HTMLElement>('[data-chip]')].map((c) => c.dataset.chip!.trim()).filter(Boolean);
        const extra = box.querySelector<HTMLInputElement>('input')!.value.split(',').map((s) => s.trim()).filter(Boolean);
        const all = [...chips, ...extra];
        if (all.length) out.crops_before = all;
        return;
      }
      if (dk) { out[k as TextKey] = UNKNOWN; return; }
      const v = box.querySelector<HTMLInputElement | HTMLTextAreaElement>('input, textarea')!.value.trim();
      if (v) out[k as TextKey] = v;
    });
    return out;
  }

  async function prefill(a: ClientAnalysis): Promise<IntakeForm> {
    const facts = Object.fromEntries(Object.entries(a.facts ?? {}).filter(([, v]) => (Array.isArray(v) ? v.length : !!v))) as IntakeForm;
    const fromCrops: IntakeForm = {};
    if (a.crop_current) fromCrops.crop_now = pick(a.crop_current.names);
    if (a.crop_history?.length) fromCrops.crops_before = a.crop_history.map((c) => pick(c.names));
    const ex = example ? await exampleIntake(a.example_site, lang()).catch(() => null) : null;
    return { ...(ex ?? {}), ...fromCrops, ...facts };
  }

  // ---------- open / close ----------
  async function open(from: HTMLElement | null = null) {
    const a = getA();
    if (!a || a.readOnly) return;
    returnTo = from ?? (document.activeElement as HTMLElement | null);
    example = isExample(a);
    values = await prefill(a);
    render();
    panel.hidden = false; scrim.hidden = false;
    requestAnimationFrame(() => { panel.classList.add('is-open'); panel.querySelector<HTMLElement>('[data-in-close]')?.focus(); });
  }
  function close() {
    if (panel.hidden || busy) return;
    panel.classList.remove('is-open'); scrim.hidden = true;
    setTimeout(() => { panel.hidden = true; }, 260);
    returnTo?.focus?.();
    // Leaving the first-step form without submitting = "I'd rather chat": the consultant opens the conversation.
    if (first) { const f = first; first = null; photos = []; f.onSkip(); }
  }
  function openFirst(onSkip: () => void, onPhotos: (files: File[]) => void) {
    const a = getA();
    if (!a || !isFresh(a)) { onSkip(); return; }
    first = { onSkip, onPhotos }; photos = [];
    try { localStorage.setItem(autoKey(a.id), '1'); } catch { /* storage blocked */ }
    void open(null);
  }
  function autoOpen() {
    const a = getA();
    if (!a || !isExample(a) || a.readOnly || a.offline || allDone(stepsOf(a))) return;
    try { if (localStorage.getItem(autoKey(a.id))) return; localStorage.setItem(autoKey(a.id), '1'); } catch { /* storage blocked: still open once */ }
    setTimeout(() => { void open(null); }, 700);
  }

  // ---------- submit ----------
  async function submit() {
    const a = getA();
    if (!a || busy) return;
    if (a.offline) { toast(t('in_offline'), 5000); return; }
    const form = collect();
    if (!Object.keys(form).length) { toast(t('in_empty')); return; }
    busy = true;
    panel.classList.add('is-busy');
    panel.querySelector<HTMLElement>('.in__busy')!.hidden = false;
    panel.querySelectorAll<HTMLButtonElement>('button').forEach((b) => { b.disabled = true; });
    const before = stepsOf(a);
    try {
      const next = await submitIntake(a, form);
      busy = false;
      panel.classList.remove('is-busy');
      const done = first; first = null;
      close();
      apply(next);
      if (done && photos.length) done.onPhotos(photos);
      photos = [];
      const n = newlyDone(before, stepsOf({ ...a, ...next })).length;
      toast(n ? t('in_done', { n }) : t('in_done0'), 4200);
    } catch {
      busy = false;
      panel.classList.remove('is-busy');
      panel.querySelector<HTMLElement>('.in__busy')!.hidden = true;
      panel.querySelectorAll<HTMLButtonElement>('button').forEach((b) => { b.disabled = false; });
      toast(t('in_err'), 6000);
    }
  }

  // ---------- events ----------
  panel.addEventListener('submit', (e) => { e.preventDefault(); void submit(); });
  panel.addEventListener('change', (e) => {
    const inp = (e.target as HTMLElement).closest<HTMLInputElement>('[data-in-photos]');
    if (!inp?.files) return;
    photos = [...photos, ...Array.from(inp.files)].slice(0, 10);
    inp.value = '';
    const n = panel.querySelector<HTMLElement>('[data-in-photos-n]');
    if (n) n.textContent = t('in_photos_n', { n: photos.length });
  });
  panel.addEventListener('click', (e) => {
    const el = e.target as HTMLElement;
    if (el.closest('[data-in-close]')) { close(); return; }
    const dk = el.closest<HTMLButtonElement>('[data-dk]');
    if (dk) {
      const box = dk.closest<HTMLElement>('.in-f')!;
      const on = !box.classList.contains('is-dk');
      box.classList.toggle('is-dk', on);
      dk.setAttribute('aria-pressed', String(on));
      const ctl = box.querySelector<HTMLInputElement | HTMLTextAreaElement>('input, textarea')!;
      ctl.disabled = on;
      if (on) { ctl.value = ''; box.querySelectorAll('[data-chip]').forEach((c) => c.remove()); } else ctl.focus();
      return;
    }
    const un = el.closest<HTMLElement>('[data-unchip]');
    if (un) { const box = un.closest<HTMLElement>('.in-f')!; un.parentElement!.remove(); box.querySelector('input')?.focus(); }
  });
  scrim.addEventListener('click', close);
  panel.addEventListener('keydown', (e) => {
    if (e.key === 'Escape') {
      if ((e.target as HTMLElement).getAttribute('aria-expanded') === 'true') return; // the combobox closes its list first
      e.stopPropagation(); close(); return;
    }
    if (e.key !== 'Tab') return;
    const f = [...panel.querySelectorAll<HTMLElement>('button:not([disabled]), input:not([disabled]), textarea:not([disabled]), a[href]')].filter((x) => x.offsetParent !== null);
    if (!f.length) return;
    const first = f[0], last = f[f.length - 1];
    if (e.shiftKey && document.activeElement === first) { last.focus(); e.preventDefault(); }
    else if (!e.shiftKey && document.activeElement === last) { first.focus(); e.preventDefault(); }
  });
  onLang(() => { if (!panel.hidden && !busy) { values = collect(); render(); } });

  // Crop suggestions (never a restriction): plain combobox for single crops, chips for the earlier crops.
  function wireCombos() {
    panel.querySelectorAll<HTMLElement>('.in-f[data-kind="crop"], .in-f[data-kind="chips"]').forEach((box) => {
      const input = box.querySelector('input')!, list = box.querySelector('ul')!;
      const chips = box.dataset.kind === 'chips';
      let sugg: CropSuggestion[] = [];
      let act = -1;
      const show = (o: boolean) => { list.hidden = !o; input.setAttribute('aria-expanded', String(o)); };
      const paint = () => {
        list.innerHTML = sugg.map((s, i) => `<li role="option" id="${input.id}-o${i}" aria-selected="${i === act}" data-i="${i}">${esc(pick(s.label))} <i>${esc(s.scientific)}</i></li>`).join('');
        input.setAttribute('aria-activedescendant', act >= 0 ? `${input.id}-o${act}` : '');
        show(sugg.length > 0);
      };
      const commit = (name: string) => {
        show(false);
        const v = name.trim();
        if (!v) return;
        if (chips) { input.insertAdjacentHTML('beforebegin', chipHtml(v)); input.value = ''; } else input.value = v;
      };
      const fetchS = debounce(async (v: string) => { sugg = await suggestCrops(v); act = -1; paint(); }, 250);
      input.addEventListener('input', () => fetchS(input.value));
      input.addEventListener('keydown', (e) => {
        if (e.key === 'ArrowDown' && sugg.length && !list.hidden) { act = (act + 1) % sugg.length; paint(); e.preventDefault(); }
        else if (e.key === 'ArrowUp' && sugg.length && !list.hidden) { act = (act - 1 + sugg.length) % sugg.length; paint(); e.preventDefault(); }
        else if (e.key === 'Enter' && (chips || act >= 0)) { e.preventDefault(); commit(act >= 0 && !list.hidden ? pick(sugg[act].label) : input.value); }
        else if (e.key === ',' && chips) { e.preventDefault(); commit(input.value); }
        else if (e.key === 'Backspace' && chips && !input.value) box.querySelector('[data-chip]:last-of-type')?.remove();
        else if (e.key === 'Escape' && !list.hidden) { show(false); e.stopPropagation(); }
      });
      input.addEventListener('blur', () => setTimeout(() => show(false), 150));
      list.addEventListener('mousedown', (e) => { const li = (e.target as HTMLElement).closest<HTMLElement>('[data-i]'); if (li) { e.preventDefault(); commit(pick(sugg[+li.dataset.i!].label)); } });
    });
  }

  return { open: (from) => { void open(from ?? null); }, autoOpen, close, openFirst };
}
