// Workspace: frosted glass over the farmer's own field. Top bar (field chip, Chat | Dashboard | Rotation, EN | ES),
// chat + section column, dashboard and rotation views, pre-diagnosis notice always visible, phone bottom sheet.
import { buildEvidence } from '../../shared/api';
import { mountBackdrop, type Backdrop } from '../../shared/backdrop';
import type { SectionKey } from '../../shared/contract';
import { $, $$, esc, toast } from '../../shared/dom';
import { applyI18n, fmt, onLang, t } from '../../shared/i18n';
import { icon } from '../../shared/icons';
import { langSwitchHtml, wireLangSwitch } from '../../shared/langswitch';
import { set, state, subscribe, syncUrl, type View } from '../state';
import { mountChat } from './chat';
import { dashboardHtml, mountDashboardMap } from './dashboard';
import { wireExports } from './export';
import { rotationHtml } from './rotation';
import { sectionsColumn, sectionsRow, sheetHtml } from './sections';
import './workspace.css';

const TABS: { v: View; label: 'ap_tab_chat' | 'ap_tab_dash' | 'ap_tab_rot'; icon: 'chat' | 'dashboard' | 'rotation' }[] = [
  { v: 'chat', label: 'ap_tab_chat', icon: 'chat' }, { v: 'dashboard', label: 'ap_tab_dash', icon: 'dashboard' }, { v: 'rotation', label: 'ap_tab_rot', icon: 'rotation' },
];

export function mountWorkspace(host: HTMLElement, onChangeField: () => void): void {
  host.innerHTML = `<div class="ws" data-view="${state.view}">
    <div class="ws__bg" id="ws-bg"></div>
    <header class="bar">
      <a class="skip-link" href="#ws-main" data-i18n="skip">${esc(t('skip'))}</a>
      <span class="si-tag bar__si" id="si-tag" hidden data-i18n="si_tag">${esc(t('si_tag'))}</span>
      <a class="bar__brand serif" href="index.html" aria-label="${esc(t('ap_home'))}" data-i18n-attr="aria-label:ap_home">Field Shift</a>
      <button type="button" class="pill bar__chip" id="chip"></button>
      <nav class="tabs" role="tablist" aria-label="${esc(t('ap_views'))}" data-i18n-attr="aria-label:ap_views">
        ${TABS.map((x) => `<button type="button" role="tab" id="tab-${x.v}" aria-controls="view-${x.v}" data-tab="${x.v}">${icon(x.icon)}<span data-i18n="${x.label}">${esc(t(x.label))}</span></button>`).join('')}
      </nav>
      <span class="bar__flags" id="flags"></span>
      ${langSwitchHtml()}
    </header>
    <ul class="secbar" id="secbar" aria-label="${esc(t('ws_sections'))}" data-i18n-attr="aria-label:ws_sections"></ul>
    <main class="ws__main" id="ws-main" tabindex="-1">
      <section class="view view--chat" id="view-chat" role="tabpanel" aria-labelledby="tab-chat">
        <div class="chat glass" id="chat"></div>
        <div class="ws__gap" aria-hidden="true"></div>
        <aside class="secs glass" id="secs" aria-label="${esc(t('ws_sections'))}"></aside>
      </section>
      <section class="view view--dashboard" id="view-dashboard" role="tabpanel" aria-labelledby="tab-dashboard" hidden></section>
      <section class="view view--rotation" id="view-rotation" role="tabpanel" aria-labelledby="tab-rotation" hidden></section>
    </main>
    <footer class="ws__foot">
      <p class="notice ws__notice">${icon('info')}<span data-i18n="prediag">${esc(t('prediag'))}</span></p>
      <p class="instr ws__credit" id="ws-credit"></p>
    </footer>
    <div class="sheet-scrim" id="scrim" hidden></div>
    <section class="sheet glass" id="sheet" role="dialog" aria-modal="true" aria-labelledby="sheet-t" hidden></section>
  </div>`;
  applyI18n(host);
  wireLangSwitch(host);

  const a0 = state.analysis!;
  const ws = $('.ws', host);
  const isPhone = () => innerWidth < 900;
  let bd: Backdrop | null = null;
  let dashCleanup: (() => void) | null = null;
  let sheetReturn: HTMLElement | null = null;

  // backdrop: the field from above, darkened so white text stays readable
  const focus = () => {
    if (isPhone()) return { x: 0.5, y: 0.3 };
    const chat = $('#chat').getBoundingClientRect(), secs = $('#secs').getBoundingClientRect();
    if (state.view !== 'chat' || !chat.width) return { x: 0.5, y: 0.5 };
    return { x: (chat.right + secs.left) / 2 / innerWidth, y: 0.46 };
  };
  bd = mountBackdrop($('#ws-bg'), {
    lat: a0.field.center.lat, lon: a0.field.center.lon, ring: a0.field.ring, sizeM: 2400, zoom: 1.08, focus,
    outline: state.view === 'rotation' ? 'plan' : 'measured', reticle: true,
    onCredit: (c, d) => { set({ snapshotCredit: d ? `${c} · ${d}` : c }); },
  });

  const chat = mountChat($('#chat'), () => state.analysis, (patch) => { set({ analysis: { ...state.analysis!, ...patch } }); });
  wireExports(ws, () => state.analysis);

  // ---------- renderers ----------
  function renderBar() {
    const a = state.analysis!;
    const area = a.field.area_ha > 0 ? ` · ${fmt(a.field.area_ha, 1)} ha` : '';
    const chip = $('#chip');
    chip.innerHTML = `${icon('pin')}<span class="bar__label">${esc(a.example_site ? t('huaral') : a.field.label || t('fs_my_field'))}${area}</span>${a.readOnly ? '' : `<span class="bar__change">${esc(t('ap_change_field'))}</span>`}`;
    chip.setAttribute('aria-label', `${a.field.label}${area}. ${t('ap_change_field')}`);
    (chip as HTMLButtonElement).disabled = !!a.readOnly;
    const flags: string[] = [];
    if (a.offline) flags.push(`<span class="demo-flag" title="${esc(t('demo_data_long'))}">${esc(t('demo_data'))}</span>`);
    else if (a.demo_mode) flags.push(`<span class="demo-flag demo-flag--soft" title="${esc(t('demo_mode_long'))}">${esc(t('demo_mode'))}</span>`);
    if (a.readOnly) flags.push(`<span class="pill">${esc(t('ws_read_only'))}</span>`);
    $('#flags').innerHTML = flags.join('');
    $('#si-tag').hidden = !a.messages.some((m) => m.photos?.some((p) => p.url.startsWith('/img/')));
    $$<HTMLButtonElement>('[data-tab]', host).forEach((b) => { const on = b.dataset.tab === state.view; b.setAttribute('aria-selected', String(on)); b.tabIndex = on ? 0 : -1; });
    const credit = state.snapshotCredit;
    $('#ws-credit').textContent = credit;
  }
  function renderSections() {
    const a = state.analysis!;
    $('#secs').innerHTML = sectionsColumn(a, state.open);
    $('#secbar').innerHTML = sectionsRow(a);
  }
  function renderView() {
    const a = state.analysis!;
    ws.dataset.view = state.view;
    (['chat', 'dashboard', 'rotation'] as View[]).forEach((v) => { $(`#view-${v}`).hidden = v !== state.view; });
    bd?.setOutline(state.view === 'rotation' ? 'plan' : 'measured');
    bd?.layout();
    dashCleanup?.(); dashCleanup = null;
    if (state.view === 'dashboard') {
      $('#view-dashboard').innerHTML = dashboardHtml(a);
      void mountDashboardMap($('#view-dashboard'), a).then((c) => { dashCleanup = c; });
    } else if (state.view === 'rotation') {
      $('#view-rotation').innerHTML = rotationHtml(a);
    }
  }
  function renderAll() { renderBar(); renderSections(); renderView(); chat.render(state.analysis!); }

  // ---------- sheet (phone) ----------
  function openSheet(k: SectionKey, from: HTMLElement | null) {
    sheetReturn = from;
    const sheet = $('#sheet');
    sheet.innerHTML = sheetHtml(state.analysis!, k);
    sheet.dataset.key = k;
    sheet.hidden = false; $('#scrim').hidden = false;
    requestAnimationFrame(() => { sheet.classList.add('is-open'); (sheet.querySelector('[data-close]') as HTMLElement)?.focus(); });
  }
  function closeSheet() {
    const sheet = $('#sheet');
    sheet.classList.remove('is-open'); $('#scrim').hidden = true;
    setTimeout(() => { sheet.hidden = true; }, 250);
    sheetReturn?.focus();
  }

  // ---------- events ----------
  host.addEventListener('click', (e) => {
    const el = e.target as HTMLElement;
    const tab = el.closest<HTMLElement>('[data-tab]');
    if (tab) { set({ view: tab.dataset.tab as View }); return; }
    const sec = el.closest<HTMLElement>('[data-sec]');
    if (sec) { const k = sec.dataset.sec as SectionKey; set({ open: state.open === k ? null : k }); return; }
    const sh = el.closest<HTMLElement>('[data-sheet]');
    if (sh) { openSheet(sh.dataset.sheet as SectionKey, sh); return; }
    if (el.closest('[data-close]') || el.id === 'scrim') { closeSheet(); return; }
    const go = el.closest<HTMLElement>('[data-goto]');
    if (go) { if (!$('#sheet').hidden) closeSheet(); set({ view: go.dataset.goto as View }); return; }
    if (el.closest('#chip')) { onChangeField(); return; }
    const be = el.closest<HTMLButtonElement>('[data-build-evidence]');
    if (be) { void build(be); }
  });
  host.addEventListener('keydown', (e) => {
    if (e.key === 'Escape' && !$('#sheet').hidden) closeSheet();
    const tab = (e.target as HTMLElement).closest<HTMLElement>('[data-tab]');
    if (tab && (e.key === 'ArrowRight' || e.key === 'ArrowLeft')) {
      const i = TABS.findIndex((x) => x.v === tab.dataset.tab);
      const n = TABS[(i + (e.key === 'ArrowRight' ? 1 : -1) + TABS.length) % TABS.length].v;
      set({ view: n }); $(`#tab-${n}`).focus(); e.preventDefault();
    }
  });

  async function build(btn: HTMLButtonElement) {
    btn.disabled = true;
    const st = host.querySelector<HTMLElement>('[data-build-status]');
    if (st) st.hidden = false;
    try { const ev = await buildEvidence(state.analysis!); set({ analysis: { ...state.analysis!, evidence: ev } }); }
    catch { toast(t('ws_send_err')); btn.disabled = false; if (st) st.hidden = true; }
  }

  subscribe((_, ch) => {
    if ('view' in ch) { renderBar(); renderView(); syncUrl(); if (ch.view === 'chat') requestAnimationFrame(() => bd?.layout()); }
    if ('open' in ch) renderSections();
    if ('analysis' in ch) {
      renderBar(); renderSections(); chat.render(state.analysis!);
      if (state.view !== 'chat') renderView();
      const k = $('#sheet').dataset.key as SectionKey | undefined;
      if (k && !$('#sheet').hidden) $('#sheet').innerHTML = sheetHtml(state.analysis!, k);
    }
    if ('snapshotCredit' in ch) renderBar();
  });
  onLang(() => { renderAll(); });
  addEventListener('resize', () => bd?.layout());

  renderAll();
  syncUrl();
  requestAnimationFrame(() => bd?.layout());

  // Kick off the evidence package when the backend has not built it yet.
  if (!a0.evidence && !a0.offline && !a0.readOnly) {
    void buildEvidence(a0).then((ev) => set({ analysis: { ...state.analysis!, evidence: ev } })).catch(() => {});
  }
}
