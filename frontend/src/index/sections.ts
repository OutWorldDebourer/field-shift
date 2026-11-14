// Markup for the index: top bar, the six sections from design-directions/index-prompts.md, corner instruments.
// Static text carries data-i18n keys so the EN | ES switch updates it in place.
import { DEMO_EVIDENCE, HUARAL_FIELD } from '../shared/demo/huaral';
import { findReading, normalPeriod, signed } from '../shared/evidence';
import { esc } from '../shared/dom';
import { fmt, fmtDate, t } from '../shared/i18n';
import { icon, type IconName } from '../shared/icons';
import { langSwitchHtml } from '../shared/langswitch';
import { glassPhoneHtml } from '../shared/phone';
import type { StrKey } from '../shared/strings';

const T = (k: StrKey, tag = 'span', cls = '') => `<${tag}${cls ? ` class="${cls}"` : ''} data-i18n="${k}">${esc(t(k))}</${tag}>`;

export function topbar(): string {
  return `<header class="top" id="top">
    <a class="skip-link" href="#main" data-i18n="skip">${esc(t('skip'))}</a>
    <div class="top__in">
      <span class="si-tag" title="${esc(t('si_tag_title'))}" data-i18n-attr="title:si_tag_title">${T('si_tag')}</span>
      <span class="top__sep" aria-hidden="true"></span>
      <a class="top__brand serif" href="#main" aria-label="Field Shift">Field Shift</a>
      <nav class="top__nav" aria-label="Main">
        <a href="#how">${T('ix_nav_how')}</a><a href="#data">${T('ix_nav_data')}</a><a href="#about">${T('ix_nav_about')}</a>
      </nav>
      <span class="top__sep top__sep--r" aria-hidden="true"></span>
      ${langSwitchHtml()}
    </div>
  </header>`;
}

export function hud(): string {
  return `<div class="hud" aria-hidden="true">
    <div class="hud__bl"><span class="instr" id="hud-alt">ALT 420 km</span><span class="scalebar" id="hud-scale" hidden><span class="instr" id="hud-scale-t"></span><i></i></span></div>
    <div class="hud__br"><span class="instr" id="hud-credit"></span></div>
  </div>`;
}

const ctas = (cls = '') => `<div class="ctas ${cls}">
  <a class="btn btn--solid" href="app.html">${T('start')}${icon('arrow')}</a>
  <a class="btn btn--glass" href="app.html?site=huaral">${T('try_example')}</a>
</div>`;

function hero(): string {
  return `<section class="hero" aria-labelledby="hero-t">
    <div class="hero__text">
      <h1 id="hero-t" class="serif hero__title">${T('ix_hero_title')}</h1>
      <p class="hero__lede">${T('ix_hero_lede')}</p>
      ${ctas()}
    </div>
    <div class="hero__phone" id="hero-phone">${glassPhoneHtml()}</div>
    <a class="scroll-cue mono" href="#how">${icon('down')}${T('scroll_cue')}</a>
  </section>`;
}

function how(): string {
  const step = (n: string, tk: StrKey, bk: StrKey, ic: IconName) => `<li class="step glass">
    <span class="step__n mono">${n}</span>${icon(ic, 'step__ico')}
    <h3 class="serif step__t">${T(tk)}</h3><p class="step__b">${T(bk)}</p></li>`;
  return `<section class="how" id="how" aria-labelledby="how-t">
    <h2 id="how-t" class="serif sec-title">${T('ix_how_title')}</h2>
    <ol class="steps">
      ${step('01', 'ix_how_1_t', 'ix_how_1_b', 'camera')}
      ${step('02', 'ix_how_2_t', 'ix_how_2_b', 'orbit')}
      ${step('03', 'ix_how_3_t', 'ix_how_3_b', 'sheet')}
    </ol>
  </section>`;
}

const DATASETS: { id: string; name: string; what: StrKey; res: string; field: boolean }[] = [
  { id: 'power', name: 'NASA POWER', what: 'ix_ds_power', res: '~50 km', field: false },
  { id: 'smap', name: 'SMAP', what: 'ix_ds_smap', res: '9 km', field: false },
  { id: 'imerg', name: 'GPM IMERG', what: 'ix_ds_imerg', res: '10 km', field: false },
  { id: 's2', name: 'Sentinel-2 / Landsat', what: 'ix_ds_s2', res: '10–30 m', field: true },
  { id: 'srtm', name: 'SRTM / NASADEM', what: 'ix_ds_srtm', res: '30 m', field: true },
  { id: 'grace', name: 'GRACE-FO', what: 'ix_ds_grace', res: '~300 km', field: false },
];

/** The one amber reading on the ground: the temperature anomaly from the real Huaral package, if present. */
export function anomalyItem(): string {
  const r = findReading(DEMO_EVIDENCE, 'temp_anomaly');
  if (!r || r.value == null) return '';
  const p = normalPeriod(DEMO_EVIDENCE) ?? '40-year';
  const d = fmtDate(r.date);
  return `<li class="ds ds--key" data-ds="key" id="ds-key"><span class="ds__key">${esc(t('ix_anomaly', { v: signed(r.value, '°C'), p }))}</span><span class="ds__res mono">${esc(t('ix_anomaly_src', { d }))}</span></li>`;
}

function data(): string {
  const labels = DATASETS.map((d, i) => `<li class="ds ds--${i}" data-ds="${d.id}">
      <span class="ds__name mono">${esc(d.name)}</span>
      <span class="ds__what">${T(d.what)}</span>
      <span class="ds__res mono">${esc(d.res)} · <span data-i18n="${d.field ? 'field_level' : 'regional'}">${esc(t(d.field ? 'field_level' : 'regional'))}</span></span>
    </li>`).join('');
  return `<section class="ground-sec data" id="data" aria-labelledby="data-t">
    <div class="data__head">
      <h2 id="data-t" class="serif sec-title">${T('ix_data_title')}</h2>
      <p class="sec-lede">${T('ix_data_lede')}</p>
    </div>
    <div class="data__stage" id="data-stage">
      <svg class="data__leaders" id="data-leaders" aria-hidden="true"></svg>
      <div class="ground-label data__field mono" id="data-field"><b>${esc(t('field_ha', { ha: fmt(HUARAL_FIELD.area_ha, 1) }))}</b></div>
      <ul class="ds-list">${labels}
        ${anomalyItem()}
      </ul>
    </div>
  </section>`;
}

function get(): string {
  const item = (tk: StrKey, bk: StrKey, ic: IconName) => `<li class="get__item">${icon(ic, 'get__ico')}<h3 class="serif">${T(tk)}</h3><p>${T(bk)}</p></li>`;
  return `<section class="ground-sec get day" id="get" aria-labelledby="get-t">
    <h2 id="get-t" class="serif sec-title">${T('ix_get_title')}</h2>
    <ul class="get__strip glass--day">
      ${item('ix_get_soil_t', 'ix_get_soil_b', 'soil')}
      ${item('ix_get_pest_t', 'ix_get_pest_b', 'bug')}
      ${item('ix_get_nut_t', 'ix_get_nut_b', 'nutrition')}
      ${item('ix_get_rot_t', 'ix_get_rot_b', 'rotation')}
      ${item('ix_get_sheet_t', 'ix_get_sheet_b', 'sheet')}
    </ul>
    <p class="get__note notice">${icon('info')}${T('ix_get_note')}</p>
  </section>`;
}

function dash(): string {
  return `<section class="ground-sec dash day" id="dash" aria-labelledby="dash-t">
    <div class="dash__head">
      <h2 id="dash-t" class="serif sec-title">${T('ix_dash_title')}</h2>
      <p class="sec-lede">${T('ix_dash_lede')}</p>
    </div>
    <div class="preview" id="dash-preview"></div>
    <a class="btn btn--day dash__open" href="app.html?site=huaral&amp;go=1&amp;tab=dashboard">${T('ix_dash_open')}${icon('arrow')}</a>
  </section>`;
}

function cta(): string {
  return `<section class="cta" id="about" aria-labelledby="cta-t">
    <h2 id="cta-t" class="serif cta__title">${T('ix_cta_title')}</h2>
    ${ctas('ctas--center')}
    <footer class="foot glass">
      <p class="foot__lead">${icon('info')}${T('support_tool')}</p>
      <p class="mono foot__team">${T('ix_foot_team')}</p>
      <p class="foot__data">${T('ix_foot_data')}</p>
      <details class="foot__src">
        <summary>${T('ix_foot_sources')}</summary>
        <ul class="mono">
          <li>NASA POWER · Prediction Of Worldwide Energy Resources (MERRA-2, CERES) · NASA open data</li>
          <li>SMAP L4 Global 3-hourly 9 km Surface and Root Zone Soil Moisture (SPL4SMGP) · NASA NSIDC DAAC · open</li>
          <li>GPM IMERG Late Run daily (GPM_3IMERGDL) · NASA GES DISC · open</li>
          <li>Sentinel-2 L2A · Contains modified Copernicus Sentinel data 2026 · via Microsoft Planetary Computer</li>
          <li>Landsat Collection 2 Level-2 · USGS/NASA · public domain</li>
          <li>NASADEM / SRTM · NASA LP DAAC · open</li>
          <li>GRACE/GRACE-FO JPL mascon CRI v4 · NASA PO.DAAC · open</li>
          <li>NEX-GDDP-CMIP6 · NASA · open-licence models only</li>
          <li>Sentinel-2 cloudless 2016 by EOX IT Services GmbH (Contains modified Copernicus Sentinel data 2016) · CC BY 4.0</li>
          <li>Place search · © OpenStreetMap contributors · ODbL · Nominatim</li>
          <li>Map library · Leaflet · BSD-2-Clause</li>
          <li data-i18n="ix_foot_fonts">${esc(t('ix_foot_fonts'))}</li>
          <li data-i18n="ix_foot_ai">${esc(t('ix_foot_ai'))}</li>
        </ul>
      </details>
      <p class="mono foot__code">${T('ix_foot_code')}</p>
    </footer>
  </section>`;
}

export function page(): string {
  return `${topbar()}
  <div class="sky" id="sky" aria-hidden="true">
    <div class="sky__globe" id="globe-host"></div>
    <div class="sky__still" id="still"><img src="/img/orbit-still.jpg" alt="" width="1672" height="768" decoding="async" fetchpriority="high"></div>
    <div class="sky__dawn"></div>
    <div class="sky__overlay" id="sky-overlay">
      <div class="reticle" id="hero-reticle"><i></i></div>
      <svg class="sky__leaders" id="hero-leaders"></svg>
      <div class="ground-label" id="lbl-valley"><span>${esc(t('huaral'))}</span><br>11.47°S&nbsp;&nbsp;77.21°W</div>
      <div class="ground-label" id="lbl-field"><b>${esc(t('field_ha', { ha: fmt(HUARAL_FIELD.area_ha, 1) }))}</b><br><span>${esc(t('peas_now'))}</span><br><span>${esc(t('lettuce_before'))}</span></div>
      <div class="ground-label" id="lbl-how">${esc(t('ix_valley_label'))}</div>
    </div>
  </div>
  ${hud()}
  <main id="main" tabindex="-1">
    ${hero()}
    ${how()}
    <div class="ground" id="ground">
      <div class="ground__bg" id="ground-bg"><div class="ground__veil" id="ground-veil"></div></div>
      ${data()}
      ${get()}
      ${dash()}
    </div>
    ${cta()}
  </main>`;
}
