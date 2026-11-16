// The how-to film ("tour"): links that open it in a dialog, and the card with chapters on the index.
// The film is hosted on YouTube (one video per language) so the 1 GB/month free VM never streams it.
// Nothing loads from YouTube until the viewer presses play: the dialog first shows our own poster (facade),
// then a youtube-nocookie.com iframe. With no video id configured for a language, every tour link stays hidden.
import './tour.css';
import { esc } from './dom';
import { lang, onLang, t } from './i18n';
import { icon } from './icons';
import type { StrKey } from './strings';

/** YouTube video ids of the tutorial film (film/tutorial/), per language. Empty = not published yet. */
export const TOUR_VIDEO: Record<'en' | 'es', string> = { en: 'siLFiw89jxk', es: 'MyDycfiTa2w' };
const POSTER = { en: '/img/tour-poster-en.webp', es: '/img/tour-poster-es.webp' };
/** Chapter starts in seconds: the same in both cuts (film/tutorial/YOUTUBE.md). */
export const TOUR_CHAPTERS: { s: number; k: StrKey }[] = [
  { s: 24, k: 'tour_ch1' }, { s: 37, k: 'tour_ch2' }, { s: 55, k: 'tour_ch3' }, { s: 87, k: 'tour_ch4' }, { s: 116, k: 'tour_ch5' },
];

const vid = (): string => TOUR_VIDEO[lang()] || TOUR_VIDEO.en;
const T = (k: StrKey) => `<span data-i18n="${k}">${esc(t(k))}</span>`;

/** A link that opens the film. `kind`: 'hero' (under the index buttons), 'line' (one quiet line), 'chip' (app bar). */
export function tourLinkHtml(kind: 'hero' | 'line' | 'chip'): string {
  const label: StrKey = kind === 'hero' ? 'tour_watch' : kind === 'line' ? 'tour_new' : 'tour_chip';
  // the app bar is full: there the chip is the play icon alone, its label kept for screen readers and as a tooltip
  const tip = kind === 'chip' ? ` title="${esc(t('tour_title'))}" data-i18n-attr="title:tour_title"` : '';
  return `<button type="button" class="tour-link tour-link--${kind}" data-tour="0"${tip}${vid() ? '' : ' hidden'}>${icon('play')}${T(label)}</button>`;
}

/** The index card: our poster with a play button, and one button per chapter. */
export function tourCardHtml(): string {
  const ch = TOUR_CHAPTERS.map((c, i) => `<li><button type="button" class="tour-ch" data-tour="${c.s}"><span class="mono">0${i + 1}</span>${T(c.k)}<span class="tour-ch__t mono">${Math.floor(c.s / 60)}:${String(c.s % 60).padStart(2, '0')}</span></button></li>`).join('');
  return `<figure class="tour-card glass"${vid() ? '' : ' hidden'}>
    <button type="button" class="tour-card__poster" data-tour="0" aria-label="${esc(t('tour_play'))}" data-i18n-attr="aria-label:tour_play">
      <img src="${POSTER[lang()]}" alt="" loading="lazy" width="1280" height="720" data-tour-poster>
      <span class="tour-card__play">${icon('play')}</span>
    </button>
    <figcaption><p class="tour-card__t serif">${T('tour_title')}</p><p class="tour-card__d">${T('tour_desc')}</p><ol class="tour-card__ch">${ch}</ol></figcaption>
  </figure>`;
}

let dlg: HTMLDialogElement | null = null;
function dialog(): HTMLDialogElement {
  if (dlg) return dlg;
  dlg = document.createElement('dialog');
  dlg.className = 'tour-dlg';
  dlg.setAttribute('aria-labelledby', 'tour-dlg-t');
  dlg.innerHTML = `<div class="tour-dlg__bar"><p id="tour-dlg-t" class="serif">${T('tour_title')}</p>
    <a class="tour-dlg__yt mono" target="_blank" rel="noopener">${T('tour_on_yt')}${icon('arrow')}</a>
    <button type="button" class="tour-dlg__x" aria-label="${esc(t('close'))}" data-i18n-attr="aria-label:close">${icon('close')}</button></div>
    <div class="tour-dlg__frame"></div>`;
  dlg.querySelector('.tour-dlg__x')!.addEventListener('click', () => dlg!.close());
  dlg.addEventListener('click', (e) => { if (e.target === dlg) dlg!.close(); });           // click on the backdrop
  dlg.addEventListener('close', () => { dlg!.querySelector('.tour-dlg__frame')!.innerHTML = ''; });   // stop the video
  document.body.append(dlg);
  return dlg;
}

/** Open the film at `start` seconds, in the page's language. */
export function openTour(start = 0): void {
  const id = vid(); if (!id) return;
  const d = dialog();
  const q = new URLSearchParams({ autoplay: '1', rel: '0', modestbranding: '1', playsinline: '1', cc_load_policy: '0', hl: lang(), enablejsapi: '1', origin: location.origin });
  if (start > 0) q.set('start', String(start));
  d.querySelector('.tour-dlg__frame')!.innerHTML = `<iframe src="https://www.youtube-nocookie.com/embed/${encodeURIComponent(id)}?${q}"
    title="${esc(t('tour_title'))}" allow="autoplay; encrypted-media; picture-in-picture; fullscreen" referrerpolicy="strict-origin-when-cross-origin"></iframe>`;
  // The film has its captions burned in; YouTube turns its own track on for muted autoplay, which would double them.
  // The player API (postMessage, no extra script) unloads that module a few times while the player starts.
  const f = d.querySelector('iframe')!;
  f.addEventListener('load', () => {
    const send = (msg: object) => f.contentWindow?.postMessage(JSON.stringify(msg), 'https://www.youtube-nocookie.com');
    send({ event: 'listening' });
    for (const ms of [400, 1200, 2500, 4500]) setTimeout(() => { send({ event: 'command', func: 'unloadModule', args: ['captions'] }); send({ event: 'command', func: 'unloadModule', args: ['cc'] }); }, ms);
  });
  (d.querySelector('.tour-dlg__yt') as HTMLAnchorElement).href = `https://www.youtube.com/watch?v=${encodeURIComponent(id)}${start ? `&t=${start}s` : ''}`;
  if (!d.open) d.showModal();
}

/** One delegated listener for every [data-tour] on the page; posters and visibility follow the language switch. */
export function initTour(): void {
  document.addEventListener('click', (e) => {
    const b = (e.target as HTMLElement).closest<HTMLElement>('[data-tour]');
    if (!b) return;
    e.preventDefault();
    openTour(Number(b.dataset.tour) || 0);
  });
  onLang(() => {
    document.querySelectorAll<HTMLElement>('.tour-link, .tour-card').forEach((el) => { el.hidden = !vid(); });
    document.querySelectorAll<HTMLImageElement>('[data-tour-poster]').forEach((im) => { im.src = POSTER[lang()]; });
  });
}
