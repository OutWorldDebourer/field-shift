// App entry: field selection (no analysis yet) or the workspace (?a=<id>, ?share=<token>, ?site=huaral&go=1).
import '../shared/tokens.css';
import '../shared/components.css';
import '../shared/charts.css';
import './app.css';
import '../shared/phone.css';
import { getAnalysis, getShared, type ClientAnalysis } from '../shared/api';
import { $, esc } from '../shared/dom';
import { applyI18n, lang, onLang, t } from '../shared/i18n';
import { icon } from '../shared/icons';
import { langSwitchHtml, wireLangSwitch } from '../shared/langswitch';
import { DEMO_ID } from '../shared/demo/huaral';
import { isView, set, state } from './state';

const root = $('#root');
const q = new URLSearchParams(location.search);
const tab = q.get('tab');
if (isView(tab)) state.view = tab;
let cleanup: (() => void) | null = null;

function title() { document.title = t('ap_title'); }
onLang(title);
title();

async function showField(site: string | null, go: boolean) {
  cleanup?.();
  root.innerHTML = `<div class="fsel">
    <header class="bar bar--fs"><a class="skip-link" href="#fs-t" data-i18n="skip">${esc(t('skip'))}</a>
      <a class="bar__brand serif" href="index.html" aria-label="${esc(t('ap_home'))}" data-i18n-attr="aria-label:ap_home">Field Shift</a>${langSwitchHtml()}</header>
    <div class="fsel__body" id="fsel"></div></div>`;
  applyI18n(root);
  wireLangSwitch(root);
  const { mountFieldSelection } = await import('./field/select');
  cleanup = mountFieldSelection($('#fsel'), { site, go, onDone: (a) => openWorkspace(a) });
}

async function openWorkspace(a: ClientAnalysis) {
  cleanup?.(); cleanup = null;
  set({ analysis: a });
  const { mountWorkspace } = await import('./workspace/workspace');
  mountWorkspace(root, () => {
    const u = new URL(location.href);
    ['a', 'tab', 'share'].forEach((k) => u.searchParams.delete(k));
    history.pushState(null, '', u);
    void showField(state.analysis?.example_site ?? null, false);
  });
}

function loading() {
  root.innerHTML = `<div class="app-load"><p class="status-line"><i class="tick"></i>${esc(t('ap_loading'))}</p></div>`;
}

function failed(retry: () => void) {
  root.innerHTML = `<div class="app-load"><div class="app-err glass"><p>${esc(t('ap_offline_custom'))}</p>
    <div class="app-err__a"><button type="button" class="btn btn--glass btn--sm" id="retry">${esc(t('ap_retry'))}</button>
    <a class="btn btn--solid btn--sm" href="app.html?a=${DEMO_ID}">${esc(t('ap_open_example'))}${icon('arrow')}</a></div></div></div>`;
  $('#retry').addEventListener('click', retry);
}

async function boot() {
  const id = q.get('a'), share = q.get('share');
  if (share || id) {
    loading();
    try { await openWorkspace(share ? await getShared(share) : await getAnalysis(id!, lang())); }
    catch { failed(boot); }
    return;
  }
  await showField(q.get('site'), q.get('go') === '1');
}
addEventListener('popstate', () => location.reload());
void boot();
