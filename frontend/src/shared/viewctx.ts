// What the farmer is looking at (iteration 4, phase B): the current tab and the card they asked about. Sent with
// every chat turn (`view`, `focus`) so the consultant can resolve "this one" and read the same numbers with
// get_view. Also the "Ask about this" button markup used on cards.
import { esc } from './dom';
import { t } from './i18n';
import { icon } from './icons';
import './askbtn.css';

let view = 'chat';
let focus: string | null = null;
const listeners: ((v: string, f: string | null) => void)[] = [];

export function setViewContext(v: string, f: string | null = null): void {
  view = v; focus = f;
  listeners.forEach((fn) => fn(view, focus));
}
export const getViewContext = (): { view: string; focus: string | null } => ({ view, focus });
export function onViewContext(fn: (v: string, f: string | null) => void): void { listeners.push(fn); }

/** "Ask about this" on a card. `label` names the card the way the screen shows it ("strategy 2 (Follow with maize)"). */
export const askBtn = (label: string): string =>
  `<button type="button" class="askbtn" data-ask="${esc(label)}" title="${esc(t('ask_this_t'))}">${icon('chat')}<span>${esc(t('ask_this'))}</span></button>`;
