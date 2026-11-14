// Glass chat with the consultant: bubbles, photo tiles (reticle on tagged photos), status line, composer with up to
// 10 photos (drag and drop on desktop, camera on phone, ≤1600 px JPEG), and crop inputs with suggestions.
import { sendMessage, suggestCrops, type ClientAnalysis, type CropSuggestion } from '../../shared/api';
import type { ChatMessage } from '../../shared/contract';
import { $, esc, toast } from '../../shared/dom';
import { lang, onLang, pick, t } from '../../shared/i18n';
import { icon } from '../../shared/icons';
import { debounce } from '../field/search';
import { downscale, MAX_PHOTOS } from './photos';

const TAGS: Record<string, { en: string; es: string }> = { blurry: { en: 'blurry', es: 'borrosa' } };

function photoTile(p: { url: string; tag?: string }, n: number): string {
  const demoBlur = p.tag === 'blurry' && p.url.startsWith('/img/');
  const tag = p.tag ? (TAGS[p.tag]?.[lang()] ?? p.tag) : '';
  return `<figure class="tile${demoBlur ? ' tile--blurry' : ''}"><img src="${esc(p.url)}" alt="${esc(t('ws_photo_n', { n }))}${tag ? ' · ' + esc(tag) : ''}" loading="lazy" decoding="async">
    ${p.tag ? `<div class="reticle is-locking"><i></i></div><figcaption class="tile__tag">${esc(tag)}</figcaption>` : ''}</figure>`;
}

function para(s: string): string { return s.split(/\n{2,}/).map((x) => `<p>${esc(x).replace(/\n/g, '<br>')}</p>`).join(''); }

export function messageHtml(m: ChatMessage): string {
  if (m.role === 'status') return `<li class="msg msg--status"><p class="status-line">${icon('orbit')}${esc(m.body)}</p></li>`;
  const photos = m.photos?.length ? `<div class="tiles tiles--msg">${m.photos.map((p, i) => photoTile(p, i + 1)).join('')}</div>` : '';
  if (m.role === 'assistant') return `<li class="msg msg--ai"><span class="avatar-ai" aria-hidden="true">${icon('spark')}</span>
    <div class="msg__col"><span class="sr-only">${esc(t('ws_consultant'))}:</span><div class="bubble bubble--ai">${para(m.body)}</div>${photos}</div></li>`;
  return `<li class="msg msg--me"><div class="msg__col"><span class="sr-only">${esc(t('ws_you'))}:</span>${m.body ? `<div class="bubble bubble--me">${para(m.body)}</div>` : ''}${photos}</div></li>`;
}

export interface Chat { render(a: ClientAnalysis): void; destroy(): void }

export function mountChat(host: HTMLElement, getA: () => ClientAnalysis | null, onUpdate: (patch: Partial<ClientAnalysis>) => void): Chat {
  const coarse = matchMedia('(pointer: coarse)').matches;
  host.innerHTML = `<header class="chat__head">
      <h2 class="sr-only">${esc(t('ws_chat_label'))}</h2>
      <div class="crops">
        <div class="crop" data-crop="now"><label for="crop-now" class="mono">${esc(t('ws_crop_now'))}</label>
          <input id="crop-now" autocomplete="off" role="combobox" aria-expanded="false" aria-controls="crop-now-list" aria-autocomplete="list" aria-describedby="crop-hint" placeholder="${esc(t('ws_crop_ph'))}">
          <ul id="crop-now-list" class="crop__list" role="listbox" hidden></ul></div>
        <div class="crop" data-crop="before"><label for="crop-before" class="mono">${esc(t('ws_crop_before'))}</label>
          <input id="crop-before" autocomplete="off" role="combobox" aria-expanded="false" aria-controls="crop-before-list" aria-autocomplete="list" aria-describedby="crop-hint" placeholder="${esc(t('ws_crop_ph'))}">
          <ul id="crop-before-list" class="crop__list" role="listbox" hidden></ul></div>
        <p id="crop-hint" class="sr-only">${esc(t('ws_crop_hint'))}</p>
      </div>
    </header>
    <ol class="chat__log" aria-live="polite" aria-label="${esc(t('ws_chat_label'))}"></ol>
    <p class="chat__pending status-line" hidden><i class="tick"></i><span></span></p>
    <form class="composer" novalidate>
      <ul class="composer__previews" hidden></ul>
      <div class="composer__row">
        <label class="icon-btn composer__cam" title="${esc(t('ws_add_photos'))}">${icon('camera')}<span class="sr-only">${esc(t('ws_add_photos'))}</span>
          <input type="file" accept="image/*" ${coarse ? 'capture="environment"' : ''} multiple class="sr-only"></label>
        <label class="sr-only" for="composer-text">${esc(t('ws_input_ph'))}</label>
        <textarea id="composer-text" rows="1" placeholder="${esc(t('ws_input_ph'))}" enterkeyhint="send"></textarea>
        <button type="submit" class="composer__send" aria-label="${esc(t('ws_send'))}">${icon('send')}</button>
      </div>
    </form>
    <div class="chat__drop" hidden><p class="serif">${esc(t('ws_drop'))}</p></div>`;

  const log = $('.chat__log', host), pending = $('.chat__pending', host);
  const form = $<HTMLFormElement>('.composer', host), text = $<HTMLTextAreaElement>('textarea', host);
  const fileIn = $<HTMLInputElement>('input[type=file]', host), previews = $('.composer__previews', host);
  const drop = $('.chat__drop', host);
  let files: { blob: Blob; url: string }[] = [];
  let sending = false;

  const renderPreviews = () => {
    previews.hidden = !files.length;
    previews.innerHTML = files.map((f, i) => `<li><img src="${f.url}" alt="${esc(t('ws_photo_n', { n: i + 1 }))}"><button type="button" class="composer__rm" data-rm="${i}" aria-label="${esc(t('ws_remove_photo', { n: i + 1 }))}">${icon('close')}</button></li>`).join('');
  };
  previews.addEventListener('click', (e) => {
    const b = (e.target as HTMLElement).closest<HTMLElement>('[data-rm]');
    if (!b) return;
    const [f] = files.splice(+b.dataset.rm!, 1);
    URL.revokeObjectURL(f.url);
    renderPreviews();
  });
  async function addFiles(list: FileList | File[]) {
    const imgs = [...list].filter((f) => f.type.startsWith('image/'));
    const room = MAX_PHOTOS - files.length;
    if (imgs.length > room) toast(t('ws_max_photos'));
    showPending(t('ws_status_preparing'));
    for (const f of imgs.slice(0, Math.max(0, room))) {
      const blob = await downscale(f).catch(() => f);
      files.push({ blob, url: URL.createObjectURL(blob) });
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

  text.addEventListener('input', () => { text.style.height = 'auto'; text.style.height = Math.min(160, text.scrollHeight) + 'px'; });
  text.addEventListener('keydown', (e) => { if (e.key === 'Enter' && !e.shiftKey && !coarse) { e.preventDefault(); form.requestSubmit(); } });

  function showPending(msg: string) { pending.hidden = false; pending.querySelector('span')!.textContent = msg; }
  function hidePending() { pending.hidden = true; }

  async function send(body: string, blobs: Blob[]) {
    const a = getA();
    if (!a || sending || a.readOnly) return;
    if (!body.trim() && !blobs.length) return;
    sending = true;
    form.classList.add('is-busy');
    const optimistic: ChatMessage = { id: -1, role: 'user', body, created_at: new Date().toISOString(), photos: files.map((f) => ({ url: f.url })) };
    log.insertAdjacentHTML('beforeend', messageHtml(optimistic));
    scrollEnd();
    showPending(t(blobs.length ? 'ws_status_reading' : 'ws_status_checking'));
    try {
      const r = await sendMessage(a, body, blobs, lang());
      onUpdate({ messages: r.messages, sections: r.sections, ...(r.demo_mode != null ? { demo_mode: r.demo_mode } : {}) });
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

  function render(a: ClientAnalysis) {
    log.innerHTML = a.messages.length ? a.messages.map(messageHtml).join('') : `<li class="chat__empty"><span class="avatar-ai" aria-hidden="true">${icon('spark')}</span><h3 class="serif">${esc(t('ws_empty_t'))}</h3><p>${esc(t('ws_empty_b'))}</p></li>`;
    const now = $<HTMLInputElement>('#crop-now', host), before = $<HTMLInputElement>('#crop-before', host);
    if (document.activeElement !== now) now.value = a.crop_current ? pick(a.crop_current.names) : '';
    if (document.activeElement !== before) before.value = a.crop_history[0] ? pick(a.crop_history[0].names) : '';
    const ro = !!a.readOnly;
    form.hidden = ro;
    host.querySelectorAll<HTMLInputElement>('.crop input').forEach((i) => { i.readOnly = ro; });
    scrollEnd();
  }
  const off = onLang(() => {
    text.placeholder = t('ws_input_ph');
    host.querySelectorAll<HTMLInputElement>('.crop input').forEach((i) => { i.placeholder = t('ws_crop_ph'); });
    host.querySelector('[for=crop-now]')!.textContent = t('ws_crop_now');
    host.querySelector('[for=crop-before]')!.textContent = t('ws_crop_before');
    $('.composer__send', host).setAttribute('aria-label', t('ws_send'));
    $('.chat__drop p', host).textContent = t('ws_drop');
    const a = getA(); if (a) render(a);
  });
  return { render, destroy: off };
}
