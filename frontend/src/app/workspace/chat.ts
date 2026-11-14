// Glass chat with the consultant: progress rail of the six topics, bubbles, photo tiles (reticle on tagged photos),
// PDF document chips, status line, composer with up to 10 files — full-resolution photos and PDFs (drag and drop,
// camera on phone) — voice dictation (transcribes into the box, never sends), crop inputs with suggestions, the
// "Load information" button, the PDF hint, the example-site card and the closing "enough information" card.
import { getAnalysis, sendMessage, startConsultant, suggestCrops, type ClientAnalysis, type CropSuggestion } from '../../shared/api';
import type { ChatMessage, SectionKey } from '../../shared/contract';
import { $, esc, toast } from '../../shared/dom';
import { loadbarHtml, NASA_SOURCES, startLoadbar, type Loadbar } from '../../shared/loadbar';
import { lang, onLang, pick, t } from '../../shared/i18n';
import { icon } from '../../shared/icons';
import { debounce } from '../field/search';
import { chimeOn, setChime, unlockAudio } from './chime';
import { micButtonHtml, mountDictation, voiceLineHtml } from './dictation';
import { preparePhoto, MAX_PHOTOS } from './photos';
import { ACCEPT, asPdf, docChipHtml, isPdfAttachment, isPdfFile, MAX_PDF_BYTES, previewHtml, type Attachment, type Pending } from './attach';
import { exampleCardHtml } from './intake';
import { stepsOf } from './progress';
import { doneCardHtml, railHtml } from './rail';

const TAGS: Record<string, { en: string; es: string }> = { blurry: { en: 'blurry', es: 'borrosa' } };

function photoTile(p: { url: string; tag?: string }, n: number): string {
  const demoBlur = p.tag === 'blurry' && p.url.startsWith('/img/');
  const tag = p.tag ? (TAGS[p.tag]?.[lang()] ?? p.tag) : '';
  return `<figure class="tile${demoBlur ? ' tile--blurry' : ''}"><img src="${esc(p.url)}" alt="${esc(t('ws_photo_n', { n }))}${tag ? ' · ' + esc(tag) : ''}" loading="lazy" decoding="async">
    ${p.tag ? `<div class="reticle is-locking"><i></i></div><figcaption class="tile__tag">${esc(tag)}</figcaption>` : ''}</figure>`;
}

/** Plain text with the consultant's light markdown: **bold** and "- " list lines. */
function para(s: string): string {
  const inline = (x: string) => esc(x).replace(/\*\*(.+?)\*\*/g, '<b>$1</b>');
  return s.split(/\n{2,}/).map((block) => {
    const lines = block.split('\n');
    if (lines.every((l) => /^\s*[-·•]\s+/.test(l))) return `<ul class="bul">${lines.map((l) => `<li>${inline(l.replace(/^\s*[-·•]\s+/, ''))}</li>`).join('')}</ul>`;
    return `<p>${lines.map((l) => (/^\s*-\s+/.test(l) ? '• ' + inline(l.replace(/^\s*-\s+/, '')) : inline(l))).join('<br>')}</p>`;
  }).join('');
}

export function messageHtml(m: ChatMessage): string {
  if (m.role === 'status') return `<li class="msg msg--status"><p class="status-line">${icon('orbit')}${esc(m.body)}</p></li>`;
  const all = (m.photos ?? []) as Attachment[];
  const imgs = all.filter((p) => !isPdfAttachment(p)), docs = all.filter(isPdfAttachment);
  const photos = (imgs.length ? `<div class="tiles tiles--msg">${imgs.map((p, i) => photoTile(p, i + 1)).join('')}</div>` : '')
    + (docs.length ? `<div class="doc-chips">${docs.map(docChipHtml).join('')}</div>` : '');
  if (m.role === 'assistant') return `<li class="msg msg--ai"><span class="avatar-ai" aria-hidden="true">${icon('advisor')}</span>
    <div class="msg__col"><span class="sr-only">${esc(t('ws_consultant'))}:</span><div class="bubble bubble--ai">${para(m.body)}</div>${photos}</div></li>`;
  return `<li class="msg msg--me"><div class="msg__col"><span class="sr-only">${esc(t('ws_you'))}:</span>${m.body ? `<div class="bubble bubble--me">${para(m.body)}</div>` : ''}${photos}</div></li>`;
}

export interface Chat { render(a: ClientAnalysis, just?: SectionKey[]): void; prefill(s: string): void; destroy(): void }

export function mountChat(host: HTMLElement, getA: () => ClientAnalysis | null, onUpdate: (patch: Partial<ClientAnalysis>) => void): Chat {
  const coarse = matchMedia('(pointer: coarse)').matches;
  host.innerHTML = `<header class="chat__head">
      <h2 class="sr-only">${esc(t('ws_chat_label'))}</h2>
      <div class="chat__crops">
      <button type="button" class="crops__sum" aria-expanded="false" aria-controls="crops"></button>
      <div class="crops" id="crops">
        <div class="crop" data-crop="now"><label for="crop-now" class="mono">${esc(t('ws_crop_now'))}</label>
          <input id="crop-now" autocomplete="off" role="combobox" aria-expanded="false" aria-controls="crop-now-list" aria-autocomplete="list" aria-describedby="crop-hint" placeholder="${esc(t('ws_crop_ph'))}">
          <ul id="crop-now-list" class="crop__list" role="listbox" hidden></ul></div>
        <div class="crop" data-crop="before"><label for="crop-before" class="mono">${esc(t('ws_crop_before'))}</label>
          <input id="crop-before" autocomplete="off" role="combobox" aria-expanded="false" aria-controls="crop-before-list" aria-autocomplete="list" aria-describedby="crop-hint" placeholder="${esc(t('ws_crop_ph'))}">
          <ul id="crop-before-list" class="crop__list" role="listbox" hidden></ul></div>
        <p id="crop-hint" class="sr-only">${esc(t('ws_crop_hint'))}</p>
      </div></div>
      <button type="button" class="btn btn--glass btn--sm chat__intake" data-intake aria-haspopup="dialog">${icon('clipboard')}<span>${esc(t('in_btn'))}</span></button>
    </header>
    <nav class="rail" aria-label="${esc(t('rail_label'))}"></nav>
    <ol class="chat__log" aria-live="polite" aria-label="${esc(t('ws_chat_label'))}"></ol>
    <div class="chat__pending" hidden aria-live="polite"></div>
    <form class="composer" novalidate>
      <ul class="composer__previews" hidden></ul>
      <div class="composer__row">
        <label class="icon-btn composer__cam" title="${esc(t('ws_add_photos'))}">${icon('camera')}<span class="sr-only">${esc(t('ws_add_photos'))}</span>
          <input type="file" accept="${ACCEPT}" multiple class="sr-only"></label>
        <label class="sr-only" for="composer-text">${esc(t('ws_input_ph'))}</label>
        <textarea id="composer-text" rows="1" placeholder="${esc(t('ws_input_ph'))}" enterkeyhint="send"></textarea>
        ${micButtonHtml()}
        <button type="submit" class="composer__send" aria-label="${esc(t('ws_send'))}">${icon('send')}</button>
      </div>
      ${voiceLineHtml()}
    </form>
    <div class="chat__drop" hidden><p class="serif">${esc(t('ws_drop'))}</p></div>`;

  const log = $('.chat__log', host), pending = $('.chat__pending', host);
  const form = $<HTMLFormElement>('.composer', host), text = $<HTMLTextAreaElement>('textarea', host);
  const fileIn = $<HTMLInputElement>('input[type=file]', host), previews = $('.composer__previews', host);
  const drop = $('.chat__drop', host);
  let files: Pending[] = [];
  let sending = false;

  const renderPreviews = () => {
    previews.hidden = !files.length;
    previews.innerHTML = files.map(previewHtml).join('');
  };
  previews.addEventListener('click', (e) => {
    const b = (e.target as HTMLElement).closest<HTMLElement>('[data-rm]');
    if (!b) return;
    const [f] = files.splice(+b.dataset.rm!, 1);
    URL.revokeObjectURL(f.url);
    renderPreviews();
  });
  async function addFiles(list: FileList | File[]) {
    const all = [...list];
    const ok = all.filter((f) => f.type.startsWith('image/') || isPdfFile(f));
    if (ok.length < all.length) toast(t('ws_file_skip'));
    const fit = ok.filter((f) => { if (isPdfFile(f) && f.size > MAX_PDF_BYTES) { toast(t('ws_pdf_big', { name: f.name }), 5000); return false; } return true; });
    const room = MAX_PHOTOS - files.length;
    if (fit.length > room) toast(t('ws_max_photos'));
    showPending(t('ws_status_preparing'), 4000, []);
    for (const f of fit.slice(0, Math.max(0, room))) {
      if (isPdfFile(f)) { const pdf = asPdf(f); files.push({ blob: pdf, url: URL.createObjectURL(pdf), kind: 'pdf', name: f.name }); continue; }
      const blob = await preparePhoto(f).catch(() => f);
      files.push({ blob, url: URL.createObjectURL(blob), kind: 'image', name: f.name });
    }
    hidePending();
    renderPreviews();
  }
  fileIn.addEventListener('change', () => { if (fileIn.files) void addFiles(fileIn.files); fileIn.value = ''; });

  // drag and drop (desktop)
  let depth = 0;
  host.addEventListener('dragenter', (e) => { if (e.dataTransfer?.types.includes('Files')) { depth++; drop.hidden = false; e.preventDefault(); } });
  host.addEventListener('dragover', (e) => { if (e.dataTransfer?.types.includes('Files')) e.preventDefault(); });
  host.addEventListener('dragleave', () => { depth = Math.max(0, depth - 1); if (!depth) drop.hidden = true; });
  host.addEventListener('drop', (e) => { e.preventDefault(); depth = 0; drop.hidden = true; if (e.dataTransfer?.files.length) void addFiles(e.dataTransfer.files); });

  const grow = () => { text.style.height = 'auto'; text.style.height = Math.min(160, text.scrollHeight) + 'px'; };
  text.addEventListener('input', grow);
  const dict = mountDictation(form, text, grow);
  text.addEventListener('keydown', (e) => { if (e.key === 'Enter' && !e.shiftKey && !coarse) { e.preventDefault(); form.requestSubmit(); } });

  // Loading bar while the consultant reads NASA data (and photos or documents); expected time per kind of turn.
  let bar: Loadbar | null = null, hideT = 0;
  function showPending(msg: string, expectMs = 40000, sources: string[] = NASA_SOURCES) {
    clearTimeout(hideT); bar?.done();
    pending.hidden = false;
    pending.innerHTML = loadbarHtml(msg);
    bar = startLoadbar(pending.firstElementChild as HTMLElement, expectMs, sources);
  }
  function hidePending() {
    bar?.done(); bar = null;
    hideT = window.setTimeout(() => { pending.hidden = true; pending.innerHTML = ''; }, 450);
  }

  async function send(body: string, blobs: Blob[]) {
    const a = getA();
    if (!a || sending || a.readOnly) return;
    if (!body.trim() && !blobs.length) return;
    sending = true;
    unlockAudio();
    dict.stop();
    form.classList.add('is-busy');
    const optimistic: ChatMessage = { id: -1, role: 'user', body, created_at: new Date().toISOString(), photos: files.map((f) => (f.kind === 'pdf' ? { url: f.url, kind: 'pdf', name: f.name, bytes: f.blob.size } as Attachment : { url: f.url })) as ChatMessage['photos'] };
    log.insertAdjacentHTML('beforeend', messageHtml(optimistic));
    scrollEnd();
    const docs = files.some((f) => f.kind === 'pdf');
    showPending(t(docs ? 'ws_status_reading_doc' : blobs.length ? 'ws_status_reading' : 'ws_status_checking'), docs || blobs.length ? 60000 : 35000, docs ? ['PDF', ...NASA_SOURCES] : blobs.length ? [t('lb_photos'), ...NASA_SOURCES] : NASA_SOURCES);
    try {
      const r = await sendMessage(a, body, blobs, lang());
      // Older turn responses carry no steps: read them from the analysis so the rail moves in the same update.
      if (!Array.isArray(r.steps) && !a.offline) {
        const fresh = await getAnalysis(a.id, lang()).catch(() => null);
        if (fresh && Array.isArray(fresh.steps)) { r.steps = fresh.steps; r.next_step = fresh.next_step ?? null; }
      }
      onUpdate({
        messages: r.messages, sections: r.sections,
        ...(r.demo_mode != null ? { demo_mode: r.demo_mode } : {}),
        ...(Array.isArray(r.steps) ? { steps: r.steps } : {}),
        ...(r.next_step !== undefined ? { next_step: r.next_step } : {}),
      });
      text.value = ''; text.style.height = 'auto';
      files = []; renderPreviews();
    } catch {
      log.lastElementChild?.remove();
      toast(t('ws_send_err'), 6000);
    } finally {
      sending = false; form.classList.remove('is-busy'); hidePending();
    }
  }
  form.addEventListener('submit', (e) => { e.preventDefault(); void send(text.value.trim(), files.map((f) => f.blob)); });

  // crop inputs with suggestions (never a restriction)
  host.querySelectorAll<HTMLElement>('.crop').forEach((box) => {
    const input = box.querySelector('input')!, list = box.querySelector('ul')!;
    const which = box.dataset.crop as 'now' | 'before';
    let sugg: CropSuggestion[] = [];
    let act = -1;
    const open = (o: boolean) => { list.hidden = !o; input.setAttribute('aria-expanded', String(o)); };
    const paint = () => {
      list.innerHTML = sugg.map((s, i) => `<li role="option" id="${input.id}-o${i}" aria-selected="${i === act}" data-i="${i}">${esc(pick(s.label))} <i>${esc(s.scientific)}</i></li>`).join('');
      input.setAttribute('aria-activedescendant', act >= 0 ? `${input.id}-o${act}` : '');
      open(sugg.length > 0);
    };
    const commit = (name: string) => {
      open(false);
      const v = name.trim();
      if (!v) return;
      input.value = v;
      void send(t(which === 'now' ? 'ws_crop_set_now' : 'ws_crop_set_before', { crop: v }), []);
    };
    const fetchS = debounce(async (v: string) => { sugg = await suggestCrops(v); act = -1; paint(); }, 250);
    input.addEventListener('input', () => fetchS(input.value));
    input.addEventListener('keydown', (e) => {
      if (e.key === 'ArrowDown' && sugg.length) { act = (act + 1) % sugg.length; paint(); e.preventDefault(); }
      else if (e.key === 'ArrowUp' && sugg.length) { act = (act - 1 + sugg.length) % sugg.length; paint(); e.preventDefault(); }
      else if (e.key === 'Enter') { e.preventDefault(); commit(act >= 0 ? pick(sugg[act].label) : input.value); }
      else if (e.key === 'Escape') open(false);
    });
    input.addEventListener('blur', () => setTimeout(() => open(false), 150));
    list.addEventListener('mousedown', (e) => { const li = (e.target as HTMLElement).closest<HTMLElement>('[data-i]'); if (li) { e.preventDefault(); commit(pick(sugg[+li.dataset.i!].label)); } });
  });

  function scrollEnd() { requestAnimationFrame(() => { log.scrollTop = log.scrollHeight; }); }

  const rail = $('.rail', host);
  rail.addEventListener('click', (e) => {
    const b = (e.target as HTMLElement).closest<HTMLButtonElement>('[data-chime]');
    if (!b) return;
    const on = !chimeOn();
    setChime(on);
    if (on) unlockAudio();
    const a = getA(); if (a) rail.innerHTML = railHtml(stepsOf(a));
    rail.querySelector<HTMLElement>('[data-chime]')?.focus();
  });

  // A new field has no consultant message yet: the consultant writes its opening for this exact place.
  let opening = false;
  async function open(a: ClientAnalysis) {
    if (opening || a.offline || a.readOnly || a.messages.some((m) => m.role === 'assistant')) return;
    opening = true; sending = true;
    form.classList.add('is-busy');
    showPending(t('ws_status_opening'), 9000, ['OpenStreetMap', 'NASA POWER']);
    try {
      const r = await startConsultant(a, lang());
      onUpdate({ messages: r.messages, sections: r.sections, ...(r.demo_mode != null ? { demo_mode: r.demo_mode } : {}), ...(Array.isArray(r.steps) ? { steps: r.steps } : {}), ...(r.next_step !== undefined ? { next_step: r.next_step } : {}) });
    } catch {
      opening = false;
      toast(t('ws_send_err'), 6000);
    } finally {
      sending = false; form.classList.remove('is-busy'); hidePending();
    }
  }

  function render(a: ClientAnalysis, just: SectionKey[] = []) {
    const steps = stepsOf(a);
    void open(a);
    rail.innerHTML = railHtml(steps, just);
    rail.hidden = !!a.readOnly;
    // The consultant's greeting arrives with the analysis: the empty state only shows when there is no message at all.
    const shown = a.messages.filter((m) => m.role !== 'status' || m.body);
    const hint = a.readOnly ? '' : `<li class="chat__hint">${icon('pdf')}<p>${esc(t('ws_pdf_hint'))}</p></li>`;
    // The PDF hint and the example-site card sit right under the consultant's greeting.
    const head = shown.length ? messageHtml(shown[0]) + hint + exampleCardHtml(a, steps) : '';
    log.innerHTML = shown.length ? head + shown.slice(1).map(messageHtml).join('') + doneCardHtml(a, steps) : `<li class="chat__empty"><span class="avatar-ai" aria-hidden="true">${icon('advisor')}</span><h3 class="serif">${esc(t('ws_empty_t'))}</h3><p>${esc(t('ws_empty_b'))}</p><p class="chat__empty-hint">${icon('pdf')}${esc(t('ws_pdf_hint'))}</p>${exampleCardHtml(a, steps, true)}</li>`;
    const now = $<HTMLInputElement>('#crop-now', host), before = $<HTMLInputElement>('#crop-before', host);
    const sum = $('.crops__sum', host);
    const both = !!(a.crop_current && a.crop_history[0]);
    host.classList.toggle('has-crops', both);
    if (both) sum.innerHTML = `<span>${esc(t('ws_crops_sum', { now: pick(a.crop_current!.names), before: pick(a.crop_history[0].names) }))}</span><u>${esc(t('ws_edit'))}</u>`;
    if (document.activeElement !== now) now.value = a.crop_current ? pick(a.crop_current.names) : '';
    if (document.activeElement !== before) before.value = a.crop_history[0] ? pick(a.crop_history[0].names) : '';
    const ro = !!a.readOnly;
    form.hidden = ro;
    host.querySelectorAll<HTMLInputElement>('.crop input').forEach((i) => { i.readOnly = ro; });
    scrollEnd();
  }
  $('.crops__sum', host).addEventListener('click', (e) => {
    const b = e.currentTarget as HTMLElement;
    const open = !host.classList.contains('crops-open');
    host.classList.toggle('crops-open', open);
    b.setAttribute('aria-expanded', String(open));
  });
  const off = onLang(() => {
    text.placeholder = t('ws_input_ph');
    host.querySelectorAll<HTMLInputElement>('.crop input').forEach((i) => { i.placeholder = t('ws_crop_ph'); });
    host.querySelector('[for=crop-now]')!.textContent = t('ws_crop_now');
    host.querySelector('[for=crop-before]')!.textContent = t('ws_crop_before');
    $('.composer__send', host).setAttribute('aria-label', t('ws_send'));
    $('.chat__drop p', host).textContent = t('ws_drop');
    $('.chat__intake span', host).textContent = t('in_btn');
    $('.composer__cam', host).title = t('ws_add_photos');
    $('.composer__cam .sr-only', host).textContent = t('ws_add_photos');
    rail.setAttribute('aria-label', t('rail_label'));
    dict.relabel();
    renderPreviews();
    const a = getA(); if (a) render(a);
  });
  function prefill(s: string) {
    text.value = s;
    grow();
    requestAnimationFrame(() => { text.focus(); text.setSelectionRange(s.length, s.length); });
  }
  return { render, prefill, destroy: () => { off(); dict.stop(); } };
}
