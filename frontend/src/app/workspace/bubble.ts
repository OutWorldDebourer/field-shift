// Floating chat bubble (iteration 4). Leaving the Chat tab turns the chat into a bubble at the bottom right (above
// the tab bar on phones) with an unread dot and a preview of the consultant's last message on hover or focus.
// Clicking it opens a mini chat over the current view (a bottom sheet on phones). It is the SAME chat element,
// moved in the DOM, so the draft text, pending photos and dictation survive. Esc closes it and focus returns to the
// bubble. With reduced motion the bubble simply appears.
import type { ClientAnalysis } from '../../shared/api';
import { onLang, t } from '../../shared/i18n';
import { icon } from '../../shared/icons';
import type { StrKey } from '../../shared/strings';
import type { Chat } from './chat';
import { getViewContext, onViewContext } from '../../shared/viewctx';
import './bubble.css';

type View = 'chat' | 'dashboard' | 'rotation' | 'organic' | 'prevention';
const VIEW_LABEL: Record<View, StrKey> = { chat: 'ap_tab_chat', dashboard: 'ap_tab_dash', rotation: 'ap_tab_rot', organic: 'ap_tab_org', prevention: 'ap_tab_prev' };

export interface ChatBubble { sync(a: ClientAnalysis, view: View): void; isOpen(): boolean; close(): void; open(): void }

const reduced = () => matchMedia('(prefers-reduced-motion: reduce)').matches;
const plain = (s: string) => s.replace(/[*_#>`]/g, '').replace(/\s+/g, ' ').trim();

export function mountBubble(ws: HTMLElement, chatEl: HTMLElement, chat: Chat): ChatBubble {
  ws.insertAdjacentHTML('beforeend', `
    <button type="button" class="cbub" hidden aria-haspopup="dialog" aria-expanded="false" aria-controls="cmini" aria-describedby="cbub-peek">
      <span class="cbub__ico" aria-hidden="true">${icon('chat')}</span><span class="cbub__dot" hidden></span><span class="sr-only cbub__sr"></span></button>
    <p class="cbub__peek glass" id="cbub-peek" hidden><span class="cbub__who mono"></span> <span class="cbub__txt"></span></p>
    <div class="cmini-scrim" hidden></div>
    <section class="cmini glass" id="cmini" role="dialog" aria-modal="false" aria-labelledby="cmini-t" hidden>
      <header class="cmini__head"><h2 id="cmini-t" class="cmini__t" tabindex="-1"><span class="mono"></span> <b></b></h2>
        <button type="button" class="icon-btn cmini__x">${icon('close')}</button></header>
      <p class="cmini__notice">${icon('info')}<span></span></p>
      <div class="cmini__body"></div>
    </section>`);
  const btn = ws.querySelector<HTMLButtonElement>('.cbub')!, dot = ws.querySelector<HTMLElement>('.cbub__dot')!;
  const peek = ws.querySelector<HTMLElement>('.cbub__peek')!, mini = ws.querySelector<HTMLElement>('.cmini')!;
  const body = mini.querySelector<HTMLElement>('.cmini__body')!, closeBtn = mini.querySelector<HTMLButtonElement>('.cmini__x')!;
  const scrim = ws.querySelector<HTMLElement>('.cmini-scrim')!;
  const phone = () => matchMedia('(max-width: 899px)').matches;
  const canHover = () => matchMedia('(hover: hover)').matches;
  let peekT = 0;
  const home = document.createComment('chat home');
  let view: View = 'chat', seen = -1, lastText = '', open = false;

  const label = () => {
    btn.setAttribute('aria-label', t('cb_open'));
    ws.querySelector('.cbub__sr')!.textContent = dot.hidden ? '' : t('cb_unread');
    mini.querySelector('.cmini__t .mono')!.textContent = t('cb_asking');
    const f = getViewContext().focus;
    mini.querySelector('.cmini__t b')!.textContent = t(VIEW_LABEL[view] ?? 'ap_tab_chat') + (f ? ` · ${f}` : '');
    closeBtn.setAttribute('aria-label', t('cb_close'));
    mini.querySelector('.cmini__notice span')!.textContent = t('cb_notice');
    peek.querySelector('.cbub__who')!.textContent = `${t('cb_consultant')} ·`;
  };

  function openMini() {
    if (open) return;
    open = true;
    chatEl.parentNode?.insertBefore(home, chatEl);
    body.appendChild(chatEl);
    mini.hidden = false; peek.hidden = true; dot.hidden = true;
    // On a phone the sheet covers the tab bar: it is modal there, with a dimmed scrim that closes it.
    mini.setAttribute('aria-modal', String(phone())); scrim.hidden = !phone();
    if (phone()) [...ws.children].forEach((el) => { if (el !== mini && el !== scrim) (el as HTMLElement).inert = true; });
    btn.setAttribute('aria-expanded', 'true'); btn.hidden = true;
    requestAnimationFrame(() => {
      mini.classList.add('is-open');
      if (matchMedia('(pointer: coarse)').matches) mini.querySelector<HTMLElement>('#cmini-t')?.focus(); else chat.focus();
    });
    label();
  }
  function closeMini(focusBubble = true) {
    if (!open) return;
    open = false;
    mini.classList.remove('is-open'); mini.hidden = true; scrim.hidden = true;
    [...ws.children].forEach((el) => { (el as HTMLElement).inert = false; });
    if (home.parentNode) { home.parentNode.insertBefore(chatEl, home); home.remove(); }
    btn.setAttribute('aria-expanded', 'false');
    btn.hidden = view === 'chat';
    if (focusBubble && !btn.hidden) btn.focus();
  }

  /** The chat shrinks into the bubble: a ghost of the chat's box flies to the bubble (skipped with reduced motion). */
  function flyIn(from: DOMRect | null) {
    btn.hidden = false;
    if (!from || reduced() || !from.width) return;
    const to = btn.getBoundingClientRect();
    const g = document.createElement('div');
    g.className = 'cbub__ghost glass';
    Object.assign(g.style, { left: `${from.left}px`, top: `${from.top}px`, width: `${from.width}px`, height: `${from.height}px` });
    document.body.appendChild(g);
    btn.classList.add('is-arriving');
    g.animate([
      { left: `${from.left}px`, top: `${from.top}px`, width: `${from.width}px`, height: `${from.height}px`, borderRadius: '22px', opacity: 0.9 },
      { left: `${to.left}px`, top: `${to.top}px`, width: `${to.width}px`, height: `${to.height}px`, borderRadius: '50%', opacity: 0.2 },
    ], { duration: 420, easing: 'cubic-bezier(.2,.7,.2,1)' }).onfinish = () => { g.remove(); btn.classList.remove('is-arriving'); };
  }

  btn.addEventListener('click', openMini);
  closeBtn.addEventListener('click', () => closeMini());
  mini.addEventListener('keydown', (e) => {
    // Inner popovers (crop suggestions, voice menu) handle their own Esc first.
    if (e.key === 'Escape' && !e.defaultPrevented && !(e.target as HTMLElement).closest('[aria-expanded=true], .voice__menu')) { e.preventDefault(); closeMini(); }
  });
  // The preview names its speaker; it shows on hover or keyboard focus, and by itself for a few seconds when a new
  // reply arrives (the only way touch users see it).
  const showPeek = () => { if (lastText && !open && !btn.hidden) { peek.querySelector('.cbub__txt')!.textContent = lastText; peek.hidden = false; } };
  const hidePeek = () => { clearTimeout(peekT); peek.hidden = true; };
  btn.addEventListener('mouseenter', () => { if (canHover()) showPeek(); }); btn.addEventListener('focus', showPeek);
  btn.addEventListener('mouseleave', hidePeek); btn.addEventListener('blur', hidePeek);
  scrim.addEventListener('click', () => closeMini());
  onLang(label);
  onViewContext(() => label());
  label();

  return {
    sync(a, v) {
      const prev = view;
      const rect = prev === 'chat' && v !== 'chat' ? chatEl.getBoundingClientRect() : null;
      view = v;
      const ai = a.messages.filter((m) => m.role === 'assistant');
      const last = ai[ai.length - 1];
      lastText = last ? plain(last.body).slice(0, 160) + (plain(last.body).length > 160 ? '…' : '') : '';
      if (a.readOnly) { closeMini(false); btn.hidden = true; return; }
      if (v === 'chat') { closeMini(false); btn.hidden = true; seen = ai.length; dot.hidden = true; label(); return; }
      if (seen < 0) seen = ai.length;
      if (open) seen = ai.length;
      if (!open && prev !== v && prev === 'chat') flyIn(rect); else if (!open) btn.hidden = false;
      const fresh = !open && ai.length > seen && dot.hidden;
      dot.hidden = open || ai.length <= seen;
      if (fresh && !dot.hidden) { showPeek(); clearTimeout(peekT); peekT = window.setTimeout(() => { peek.hidden = true; }, 6000); }
      label();
    },
    isOpen: () => open,
    close: () => closeMini(false),
    open: () => { if (open) { label(); chat.focus(); } else openMini(); },
  };
}
