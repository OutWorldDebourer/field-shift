// Voice dictation for the composer. It only TRANSCRIBES into the text box, never sends.
// Engines: "Gemini" (default; MediaRecorder → POST /api/consultant/transcribe, ≤ 120 s) and "Browser"
// (Web Speech API, interim results shown live). Gemini failures fall back to the browser with a toast.
import { transcribe } from '../../shared/api';
import { esc, toast } from '../../shared/dom';
import { lang, t } from '../../shared/i18n';
import { icon } from '../../shared/icons';

type Engine = 'gemini' | 'browser';
const KEY = 'fieldshift.dictation';
const MAX_S = 120;

type Rec = { start(): void; stop(): void; abort(): void; lang: string; interimResults: boolean; continuous: boolean;
  onresult: ((e: { resultIndex: number; results: ArrayLike<{ isFinal: boolean; 0: { transcript: string } }> }) => void) | null;
  onerror: ((e: { error: string }) => void) | null; onend: (() => void) | null };
const SR = (): (new () => Rec) | null => {
  const w = window as unknown as { SpeechRecognition?: new () => Rec; webkitSpeechRecognition?: new () => Rec };
  return w.SpeechRecognition ?? w.webkitSpeechRecognition ?? null;
};
const canRecord = () => typeof MediaRecorder !== 'undefined' && !!navigator.mediaDevices?.getUserMedia;

function saved(): Engine {
  try { const v = localStorage.getItem(KEY); if (v === 'browser' && SR()) return 'browser'; } catch { /* blocked */ }
  return canRecord() ? 'gemini' : 'browser';
}
function save(e: Engine) { try { localStorage.setItem(KEY, e); } catch { /* blocked */ } }

function mime(): string {
  for (const m of ['audio/webm;codecs=opus', 'audio/webm', 'audio/ogg;codecs=opus', 'audio/mp4']) {
    try { if (MediaRecorder.isTypeSupported(m)) return m; } catch { /* old browser */ }
  }
  return '';
}

/** Mic button markup for the composer row. */
export const micButtonHtml = () => `<button type="button" class="composer__mic" data-mic aria-pressed="false" aria-label="${esc(t('mic_start'))}" title="${esc(t('mic_start'))}">${icon('mic')}</button>`;

/** The voice line under the composer: live status and the engine switch. */
export function voiceLineHtml(): string {
  const br = !!SR();
  return `<div class="voice"><p class="voice__status" data-voice-status aria-live="polite"></p>
    <div class="voice__eng" role="radiogroup" aria-label="${esc(t('mic_engine'))}"><span class="voice__lbl mono" aria-hidden="true">${esc(t('mic_label'))}</span>
      <button type="button" role="radio" data-engine="gemini"${canRecord() ? '' : ` aria-disabled="true" title="${esc(t('mic_unavailable'))}"`}>${esc(t('mic_gemini'))}</button>
      <button type="button" role="radio" data-engine="browser"${br ? '' : ` aria-disabled="true" title="${esc(t('mic_browser_na'))}"`}>${esc(t('mic_browser'))}</button></div></div>`;
}

export interface Dictation { stop(): void; relabel(): void }

export function mountDictation(root: HTMLElement, text: HTMLTextAreaElement, onText: () => void): Dictation {
  const btn = root.querySelector<HTMLButtonElement>('[data-mic]')!;
  const status = root.querySelector<HTMLElement>('[data-voice-status]')!;
  let engine: Engine = saved();
  let busy: 'idle' | 'rec' | 'listen' | 'upload' = 'idle';
  let media: MediaRecorder | null = null, stream: MediaStream | null = null, chunks: Blob[] = [];
  let timer = 0, startedAt = 0;
  let rec: Rec | null = null;

  const paintEngine = () => root.querySelectorAll<HTMLButtonElement>('[data-engine]').forEach((b) => b.setAttribute('aria-checked', String(b.dataset.engine === engine)));
  const setBusy = (b: typeof busy, msg = '') => {
    busy = b;
    const on = b === 'rec' || b === 'listen';
    btn.setAttribute('aria-pressed', String(on));
    btn.classList.toggle('is-rec', on);
    btn.classList.toggle('is-wait', b === 'upload');
    btn.innerHTML = icon(on ? 'stop' : 'mic');
    const l = t(on ? 'mic_stop' : 'mic_start');
    btn.setAttribute('aria-label', l); btn.title = l;
    status.textContent = msg;
    root.classList.toggle('is-dictating', b !== 'idle');
  };

  /** Insert at the caret (or append), keeping one space between words. */
  const insert = (s: string, at: { start: number; end: number }) => {
    const v = text.value, before = v.slice(0, at.start), after = v.slice(at.end);
    const pre = before && !/\s$/.test(before) ? ' ' : '', post = after && !/^\s/.test(after) ? ' ' : '';
    text.value = before + pre + s + post + after;
    const caret = (before + pre + s).length;
    text.setSelectionRange(caret, caret);
    onText();
  };
  const caret = () => ({ start: text.selectionStart ?? text.value.length, end: text.selectionEnd ?? text.value.length });

  // ---------- Gemini ----------
  async function startGemini() {
    const at = caret();
    try { stream = await navigator.mediaDevices.getUserMedia({ audio: { echoCancellation: true, noiseSuppression: true } }); }
    catch { toast(t('mic_denied'), 6000); setBusy('idle'); return; }
    const m = mime();
    try { media = new MediaRecorder(stream, m ? { mimeType: m, audioBitsPerSecond: 48000 } : undefined); }
    catch { stream.getTracks().forEach((x) => x.stop()); fallback(at); return; }
    chunks = [];
    media.ondataavailable = (e) => { if (e.data.size) chunks.push(e.data); };
    media.onstop = async () => {
      clearInterval(timer);
      stream?.getTracks().forEach((x) => x.stop()); stream = null;
      const blob = new Blob(chunks, { type: media?.mimeType || m || 'audio/webm' });
      media = null;
      if (blob.size < 800) { setBusy('idle'); toast(t('mic_none')); return; }
      setBusy('upload', t('mic_transcribing'));
      try {
        const s = (await transcribe(blob, lang())).trim();
        setBusy('idle');
        if (s) insert(s, at); else toast(t('mic_none'));
      } catch { fallback(at); }
    };
    media.start(1000);
    startedAt = Date.now();
    const tick = () => {
      const s = Math.min(MAX_S, Math.floor((Date.now() - startedAt) / 1000));
      status.textContent = t('mic_rec', { t: `${Math.floor(s / 60)}:${String(s % 60).padStart(2, '0')}` });
      if (s >= MAX_S) media?.stop();
    };
    setBusy('rec'); tick();
    timer = window.setInterval(tick, 500);
  }

  function fallback(at: { start: number; end: number }) {
    setBusy('idle');
    if (!SR()) { toast(t('mic_unavailable'), 6000); return; }
    toast(t('mic_fallback'), 6000);
    engine = 'browser'; paintEngine();
    startBrowser(at);
  }

  // ---------- Browser (Web Speech) ----------
  function startBrowser(at = caret()) {
    const C = SR();
    if (!C) { toast(t('mic_browser_na')); return; }
    rec = new C();
    rec.lang = lang() === 'es' ? 'es-PE' : 'en-US';
    rec.interimResults = true; rec.continuous = true;
    const base = text.value, before = base.slice(0, at.start), after = base.slice(at.end);
    const pre = before && !/\s$/.test(before) ? ' ' : '';
    let final = '';
    rec.onresult = (e) => {
      let interim = '';
      for (let i = e.resultIndex; i < e.results.length; i++) {
        const r = e.results[i];
        if (r.isFinal) final += r[0].transcript; else interim += r[0].transcript;
      }
      const said = (final + interim).replace(/\s+/g, ' ').trim();
      text.value = before + (said ? pre + said : '') + (after && said && !/^\s/.test(after) ? ' ' : '') + after;
      const c = (before + (said ? pre + said : '')).length;
      text.setSelectionRange(c, c);
      onText();
    };
    rec.onerror = (e) => { if (e.error === 'not-allowed' || e.error === 'service-not-allowed') toast(t('mic_denied'), 6000); else if (e.error === 'no-speech') toast(t('mic_none')); };
    rec.onend = () => { rec = null; setBusy('idle'); };
    try { rec.start(); setBusy('listen', t('mic_listen')); } catch { rec = null; setBusy('idle'); toast(t('mic_unavailable')); }
  }

  btn.addEventListener('click', () => {
    if (busy === 'upload') return;
    if (busy === 'rec') { media?.stop(); return; }
    if (busy === 'listen') { rec?.stop(); return; }
    if (engine === 'gemini' && canRecord()) void startGemini(); else startBrowser();
  });
  root.querySelector('.voice__eng')?.addEventListener('click', (e) => {
    const b = (e.target as HTMLElement).closest<HTMLButtonElement>('[data-engine]');
    if (!b || b.getAttribute('aria-disabled') === 'true') { if (b) toast(b.title); return; }
    if (busy !== 'idle') return;
    engine = b.dataset.engine as Engine; save(engine); paintEngine();
  });
  paintEngine();

  return {
    stop() { if (busy === 'rec') media?.stop(); if (busy === 'listen') rec?.abort(); },
    relabel() {
      const l = t(busy === 'rec' || busy === 'listen' ? 'mic_stop' : 'mic_start');
      btn.setAttribute('aria-label', l); btn.title = l;
      root.querySelector('.voice__eng')?.setAttribute('aria-label', t('mic_engine'));
      const lbl = root.querySelector('.voice__lbl'); if (lbl) lbl.textContent = t('mic_label');
      const br = root.querySelector('[data-engine=browser]'); if (br) br.textContent = t('mic_browser');
    },
  };
}
