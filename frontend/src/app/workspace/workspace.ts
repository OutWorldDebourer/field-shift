// Workspace: frosted glass over the farmer's own field. Top bar (field chip, Chat | Dashboard | Rotation | Organic,
// EN | ES), chat + section column, dashboard, rotation and organic views, pre-diagnosis notice always visible, phone
// bottom sheet, and the "Load information" side sheet (auto-opened once for example sites).
import { buildEvidence } from '../../shared/api';
import { loadbarHtml, NASA_SOURCES, startLoadbar, type Loadbar } from '../../shared/loadbar';
import { mountBackdrop, type Backdrop } from '../../shared/backdrop';
import type { SectionKey, Step } from '../../shared/contract';
import { $, $$, esc, toast } from '../../shared/dom';
import { applyI18n, fmt, onLang, t } from '../../shared/i18n';
import { icon } from '../../shared/icons';
import { langSwitchHtml, wireLangSwitch } from '../../shared/langswitch';
import { set, state, subscribe, syncUrl, type View } from '../state';
import { mountChat } from './chat';
import { dashboardHtml, mountDashboardForecast, mountDashboardMap } from './dashboard';
import { wireExports } from './export';
import { downloadMenuHtml, wireDownloadMenu } from './downloadmenu';
import { rotationHtml } from './rotation';
import { peekHtml, peekSection, sectionsColumn, sectionsRow, sheetHtml } from './sections';
import { enableGlossary } from '../../shared/glosspanel';
import { chime } from './chime';
import { allDone, newlyDone, stepsOf } from './progress';
import { exported, markExported, markSeen, seenViews } from './readylight';
import { flipLine } from './rail';
import { copernicusGloss } from './credit';
import { isFresh, mountIntake } from './intake';
import { mountBubble } from './bubble';
import { wireEvaluate } from './evaluate';
import { setViewContext } from '../../shared/viewctx';
import { organicHtml } from './organic';
import { preventionHtml, wirePrevention } from './prevention';
import './workspace.css';
import './guided.css';
import './complete.css';

const TABS: { v: View; label: 'ap_tab_chat' | 'ap_tab_dash' | 'ap_tab_rot' | 'ap_tab_prev' | 'ap_tab_org'; icon: 'chat' | 'dashboard' | 'rotation' | 'alert' | 'ladybug' }[] = [
  { v: 'chat', label: 'ap_tab_chat', icon: 'chat' }, { v: 'dashboard', label: 'ap_tab_dash', icon: 'dashboard' }, { v: 'rotation', label: 'ap_tab_rot', icon: 'rotation' },
  { v: 'prevention', label: 'ap_tab_prev', icon: 'alert' }, { v: 'organic', label: 'ap_tab_org', icon: 'ladybug' },
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
        ${TABS.map((x) => `<button type="button" role="tab" id="tab-${x.v}" aria-controls="view-${x.v}" data-tab="${x.v}">${icon(x.icon)}<span data-i18n="${x.label}">${esc(t(x.label))}</span><span class="sr-only tab__new"></span></button>`).join('')}
      </nav>
      ${downloadMenuHtml()}
      <span class="bar__flags" id="flags"></span>
      <div class="bar__load" id="bar-load" hidden></div>
      ${langSwitchHtml()}
    </header>
    <ul class="secbar" id="secbar" aria-label="${esc(t('ws_sections'))}" data-i18n-attr="aria-label:ws_sections"></ul>
    <main class="ws__main" id="ws-main" tabindex="-1">
      <section class="view view--chat" id="view-chat" role="tabpanel" aria-labelledby="tab-chat">
        <div class="chat glass" id="chat"></div>
        <div class="ws__gap" aria-hidden="true"></div>
        <aside class="secs glass" id="secs" aria-label="${esc(t('ws_sections'))}"></aside>
        <button type="button" class="peek glass" id="peek"></button>
      </section>
      <section class="view view--dashboard" id="view-dashboard" role="tabpanel" aria-labelledby="tab-dashboard" hidden></section>
      <section class="view view--rotation" id="view-rotation" role="tabpanel" aria-labelledby="tab-rotation" hidden></section>
      <section class="view view--prevention" id="view-prevention" role="tabpanel" aria-labelledby="tab-prevention" hidden></section>
      <section class="view view--organic" id="view-organic" role="tabpanel" aria-labelledby="tab-organic" hidden></section>
    </main>
    <footer class="ws__foot">
      <p class="notice ws__notice" id="ws-notice" data-short="${esc(t('foot_short'))}">${icon('info')}<span data-i18n="prediag">${esc(t('prediag'))}</span></p>
      <p class="instr ws__credit" id="ws-credit"></p>
    </footer>
    <div class="sheet-scrim" id="scrim" hidden></div>
    <section class="sheet glass" id="sheet" role="dialog" aria-modal="true" aria-labelledby="sheet-t" hidden></section>
    <p class="sr-only" id="flip-live" aria-live="polite"></p>
  </div>`;
  applyI18n(host);
  wireLangSwitch(host);
  enableGlossary(host);

  const a0 = state.analysis!;
  const ws = $('.ws', host);
  const isPhone = () => innerWidth < 900;
  let bd: Backdrop | null = null;
  let dashCleanup: (() => void) | null = null;
  let shortCredit = '';
  let sheetReturn: HTMLElement | null = null;
  let prevSteps: Step[] = stepsOf(a0);
  let just: SectionKey[] = [];

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
    onCredit: (c, d) => { shortCredit = d ? `${(c.match(/Contains modified Copernicus Sentinel data \d{4}/) ?? [c])[0]} · ${d}` : c; set({ snapshotCredit: d ? `${c} · ${d}` : c }); },
  });

  const chat = mountChat($('#chat'), () => state.analysis, (patch) => { set({ analysis: { ...state.analysis!, ...patch } }); });
  const bubble = mountBubble(ws, $('#chat'), chat);
  wireExports(ws, () => state.analysis);
  wireDownloadMenu(ws, () => state.analysis);
  // The first export puts out the Download light.
  ws.addEventListener('click', (e) => {
    const a = state.analysis;
    const b = (e.target as HTMLElement).closest<HTMLElement>('[data-export=sheet], [data-export=zip]');
    if (a && b && !a.offline) { markExported(a.id); ws.classList.add('is-exported'); }
  });
  const intake = mountIntake(ws, () => state.analysis!, (next) => {
    const cur = state.analysis!;
    set({ analysis: { ...cur, ...next, offline: cur.offline, readOnly: cur.readOnly, shareToken: cur.shareToken, evidence: next.evidence ?? cur.evidence } });
  });

  // ---------- renderers ----------
  function renderBar() {
    const a = state.analysis!;
    const area = a.field.area_ha > 0 ? ` · ${fmt(a.field.area_ha, 1)} ha` : '';
    const chip = $('#chip');
    const name = a.example_site ? t('huaral') : a.field.label || t('fs_my_field');
    chip.innerHTML = `${icon('pin')}<span class="bar__label">${esc(name)}${area}</span>${a.readOnly ? '' : `<span class="bar__change">${esc(t('ap_change_field'))}</span>`}`;
    chip.setAttribute('aria-label', `${name}${area}. ${t('ap_change_field')}`);
    (chip as HTMLButtonElement).disabled = !!a.readOnly;
    const flags: string[] = [];
    if (a.offline) flags.push(`<span class="demo-flag" title="${esc(t('demo_data_long'))}">${esc(t('demo_data'))}</span>`);
    else if (a.demo_mode) flags.push(`<span class="demo-flag demo-flag--soft" title="${esc(t('demo_mode_long'))}"><span class="df-long">${esc(t('demo_mode'))}</span><span class="df-short">Demo</span></span>`);
    if (a.readOnly) flags.push(`<span class="pill">${esc(t('ws_read_only'))}</span>`);
    $('#flags').innerHTML = flags.join('');
    $('#si-tag').hidden = !a.messages.some((m) => m.photos?.some((p) => p.url.startsWith('/img/')));
    // Every topic done: the views not opened yet and the Download button light up ("you can go on"); each goes out
    // once it has been used (readylight.ts), so the call never keeps shouting.
    const complete = allDone(stepsOf(a)) && !a.readOnly;
    if (complete) markSeen(a.id, state.view);
    const seen = seenViews(a.id);
    let anyNew = false;
    $$<HTMLButtonElement>('[data-tab]', host).forEach((b) => {
      const isNew = complete && b.dataset.tab !== 'chat' && !seen.has(b.dataset.tab ?? '');
      anyNew ||= isNew;
      b.classList.toggle('is-new', isNew);
      const sr = b.querySelector('.tab__new'); if (sr) sr.textContent = isNew ? t('ap_tab_new') : '';
    });
    ws.classList.toggle('is-complete', complete);
    ws.classList.toggle('has-new', anyNew);
    ws.classList.toggle('is-exported', exported(a.id));
    const next = $('#dlm-btn').querySelector('.dlm__next'); if (next) next.textContent = complete && !exported(a.id) ? t('dl_next') : '';
    $$<HTMLButtonElement>('[data-tab]', host).forEach((b) => { const on = b.dataset.tab === state.view; b.setAttribute('aria-selected', String(on)); b.tabIndex = on ? 0 : -1; });
    const credit = state.snapshotCredit;
    const gloss = copernicusGloss(credit);
    $('#ws-credit').innerHTML = `<span class="cr-long">${esc(credit)}${gloss ? `<span class="cr-gloss" lang="es"> ${esc(gloss)}</span>` : ''}</span><span class="cr-short">${esc(shortCredit || credit)}${gloss ? `<span class="cr-gloss" lang="es"> ${esc(copernicusGloss(shortCredit || credit))}</span>` : ''}</span>`;
    $('#ws-notice').dataset.short = t('foot_short');
  }
  function renderSections() {
    const a = state.analysis!;
    $('#secs').innerHTML = sectionsColumn(a, state.open, just);
    requestAnimationFrame(moreCue);
    $('#secbar').innerHTML = sectionsRow(a, just);
    const p = peekSection(a);
    const peek = $('#peek');
    peek.dataset.sheet = p.key;
    peek.innerHTML = peekHtml(a, p.key);
  }
  function renderView() {
    const a = state.analysis!;
    ws.dataset.view = state.view;
    (['chat', 'dashboard', 'rotation', 'prevention', 'organic'] as View[]).forEach((v) => { $(`#view-${v}`).hidden = v !== state.view; });
    bd?.setOutline(state.view === 'rotation' ? 'plan' : 'measured');
    bd?.layout();
    dashCleanup?.(); dashCleanup = null;
    if (state.view === 'dashboard') {
      $('#view-dashboard').innerHTML = dashboardHtml(a);
      mountDashboardForecast($('#view-dashboard'), () => state.analysis!);
      syncBuild();
      void mountDashboardMap($('#view-dashboard'), a).then((c) => { dashCleanup = c; });
    } else if (state.view === 'rotation') {
      $('#view-rotation').innerHTML = rotationHtml(a);
      wireEvaluate($('#view-rotation'), () => state.analysis!, (next) => { const cur = state.analysis!; set({ analysis: { ...cur, ...next, offline: cur.offline, readOnly: cur.readOnly, shareToken: cur.shareToken } }); });
    } else if (state.view === 'prevention') {
      $('#view-prevention').innerHTML = preventionHtml(a);
      const pv = $('#view-prevention'); if (!pv.dataset.wired) { wirePrevention(pv); pv.dataset.wired = '1'; }
    } else if (state.view === 'organic') {
      $('#view-organic').innerHTML = organicHtml(a);
    }
  }
  function renderAll() { renderBar(); renderSections(); renderView(); chat.render(state.analysis!); }

  function moreCue() {
    const secs = $('#secs'), more = secs.querySelector<HTMLElement>('[data-more]');
    if (more) more.hidden = secs.scrollHeight - secs.scrollTop - secs.clientHeight < 24;
  }
  $('#secs').addEventListener('scroll', moreCue, { passive: true });

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
    if (sec) { const k = sec.dataset.sec as SectionKey | 'organic'; set({ open: state.open === k ? null : k }); return; }
    const intakeBtn = el.closest<HTMLElement>('[data-intake]');
    if (intakeBtn) { intake.open(intakeBtn); return; }
    const sh = el.closest<HTMLElement>('[data-sheet]');
    if (sh) { openSheet(sh.dataset.sheet as SectionKey, sh); return; }
    if (el.closest('[data-close]') || el.id === 'scrim') { closeSheet(); return; }
    // "Ask about this": remember the card, then open the chat right here (the bubble's mini chat off the Chat tab).
    const ask = el.closest<HTMLElement>('[data-ask]');
    if (ask) {
      setViewContext(state.view, ask.dataset.ask ?? null);
      if (state.view === 'chat') chat.focus(); else bubble.open();
      return;
    }
    const go = el.closest<HTMLElement>('[data-goto]');
    if (go) { if (!$('#sheet').hidden) closeSheet(); set({ view: go.dataset.goto as View }); return; }
    if (el.closest('#chip')) { onChangeField(); return; }
    const topic = el.closest<HTMLElement>('[data-topic]');
    if (topic) { set({ view: 'chat' }); chat.prefill(topic.dataset.topic ?? ''); return; }
    const rs = el.closest<HTMLElement>('[data-railsec]');
    if (rs) { const k = rs.dataset.railsec as SectionKey; if (isPhone()) openSheet(k, rs); else { set({ open: k }); requestAnimationFrame(() => $(`#secs [data-seckey="${k}"]`)?.scrollIntoView({ block: 'nearest', behavior: 'smooth' })); } return; }
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

  // While the NASA evidence package is built: a thin bar under the top bar (any tab) and a full bar in the panel.
  let buildAt = 0;
  let topBar: Loadbar | null = null;
  function syncBuild() {
    const st = host.querySelector<HTMLElement>('[data-build-status]');
    const btn = host.querySelector<HTMLButtonElement>('[data-build-evidence]');
    if (btn) btn.hidden = !!buildAt;
    if (st && buildAt && st.hidden) { st.hidden = false; startLoadbar(st.firstElementChild as HTMLElement, 35000, NASA_SOURCES, buildAt); }
  }
  async function loadEvidence(manual: boolean) {
    if (buildAt) return;
    buildAt = performance.now();
    const top = $('#bar-load');
    top.hidden = false;
    top.innerHTML = loadbarHtml(t('db_building_short'), 'lb--thin');
    topBar = startLoadbar(top.firstElementChild as HTMLElement, 35000, NASA_SOURCES, buildAt);
    syncBuild();
    try {
      const ev = await buildEvidence(state.analysis!);
      topBar.done(); buildAt = 0;
      set({ analysis: { ...state.analysis!, evidence: ev } });
    } catch {
      topBar.fail(); buildAt = 0;
      if (manual) toast(t('ws_send_err'));
      syncBuild();
      const st = host.querySelector<HTMLElement>('[data-build-status]'); if (st) st.hidden = true;
    } finally {
      setTimeout(() => { if (!buildAt) { top.hidden = true; top.innerHTML = ''; } }, 600);
    }
  }
  async function build(btn: HTMLButtonElement) { btn.disabled = true; await loadEvidence(true); btn.disabled = false; }

  subscribe((_, ch) => {
    if ('view' in ch) { setViewContext(state.view); renderBar(); renderView(); syncUrl(); if (ch.view === 'chat') requestAnimationFrame(() => bd?.layout()); }
    if ('open' in ch) renderSections();
    if ('analysis' in ch) {
      const steps = stepsOf(state.analysis!);
      const finished = allDone(steps) && !allDone(prevSteps);
      just = newlyDone(prevSteps, steps);
      prevSteps = steps;
      if (just.length) { chime(); $('#flip-live').textContent = just.map((k) => flipLine(steps, k)).join(' '); }
      renderBar(); renderSections(); chat.render(state.analysis!, just);
      if (finished) { ws.classList.add('is-completing'); setTimeout(() => ws.classList.remove('is-completing'), 2000); }
      just = [];
      if (state.view !== 'chat') renderView();
      const k = $('#sheet').dataset.key as SectionKey | undefined;
      if (k && !$('#sheet').hidden) $('#sheet').innerHTML = sheetHtml(state.analysis!, k);
    }
    if ('snapshotCredit' in ch) renderBar();
    if ('view' in ch || 'analysis' in ch) bubble.sync(state.analysis!, state.view);
  });
  onLang((l) => {
    // Scripted demo bubbles carry both languages: re-render them in the new one so history never mixes languages.
    const a = state.analysis!;
    if (a.offline || a.demo_mode) state.analysis = { ...a, messages: a.messages.map((m) => { const b = (m as { _bi?: { en: string; es: string } })._bi; return b ? { ...m, body: b[l] } : m; }) };
    renderAll();
  });
  addEventListener('resize', () => bd?.layout());

  // A new analysis starts with the one-screen intake (no model call); the consultant's opening waits for "Skip".
  const fresh = isFresh(a0);
  if (fresh) chat.holdOpening(true);
  renderAll();
  setViewContext(state.view);
  bubble.sync(a0, state.view);
  syncUrl();
  requestAnimationFrame(() => bd?.layout());
  if (fresh) intake.openFirst(() => chat.holdOpening(false), (files) => { void chat.addFiles(files); });
  else intake.autoOpen();

  // Kick off the evidence package when the backend has not built it yet.
  if (!a0.evidence && !a0.offline && !a0.readOnly) {
    void loadEvidence(false);
  }
}
