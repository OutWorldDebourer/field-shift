// Index entry: renders the page, wires the language switch, maps scroll to the globe dive and the sunrise.
import '../shared/tokens.css';
import '../shared/components.css';
import '../shared/charts.css';
import './index.css';
import { mountBackdrop } from '../shared/backdrop';
import { HUARAL_FIELD } from '../shared/demo/huaral';
import { $, $$, esc } from '../shared/dom';
import { applyI18n, fmt, onLang, t } from '../shared/i18n';
import { wireLangSwitch } from '../shared/langswitch';
import { glassPhoneHtml } from '../shared/phone';
import { renderPreview } from './preview';
import { anomalyItem, page } from './sections';
import { mountSky } from './sky';

const root = $('#root');
root.innerHTML = page();
applyI18n(document);
wireLangSwitch(root);

const sky = mountSky();
const groundBg = $('#ground-bg');
const wideStage = () => innerWidth >= 900;
const FOCUS = () => (wideStage() ? { x: 0.5, y: 0.56 } : { x: 0.5, y: 0.5 });
let groundCredit = t('eox_short');
const ground = mountBackdrop(groundBg, {
  lat: HUARAL_FIELD.center.lat, lon: HUARAL_FIELD.center.lon, ring: HUARAL_FIELD.ring,
  sizeM: 3200, zoom: 1.05, focus: FOCUS, reticle: true,
  onCredit: (c, d) => { groundCredit = d ? `${c} · ${d}` : c; },
});
renderPreview($('#dash-preview'));
sky.onCamera(() => { const y = scrollY, d = $('#data').getBoundingClientRect(); if (d.top < innerHeight && d.bottom > 0 && y >= 0) drawDataLeaders(); });

// ---- leaders in "Read from space": from the field centre to each dataset label ----
function drawDataLeaders() {
  const stage = $('#data-stage');
  const svg = $('#data-leaders') as unknown as SVGSVGElement;
  const fieldLbl = $('#data-field');
  const sr = stage.getBoundingClientRect();
  const H = innerHeight;
  const f = FOCUS();
  const ts = sky.isLive() ? sky.camera()?.targetScreen ?? null : null;
  const cx = ts ? ts.x - sr.left : sr.width * f.x, cy = ts ? ts.y - sr.top : H * f.y - (sr.top - $('#data').getBoundingClientRect().top);
  const fb = sky.isLive() ? null : ground.fieldBox();
  const rx = fb ? fb.width / 2 : 92, ry = fb ? fb.height / 2 : 84;
  fieldLbl.style.transform = `translate(${cx - rx - 8}px, ${cy + ry + 10}px)`;
  if (!wideStage()) { svg.innerHTML = ''; return; }
  let d = '';
  $$('.ds', stage).forEach((li) => {
    const r = li.getBoundingClientRect();
    const left = r.left - sr.left, top = r.top - sr.top;
    const onRight = left > cx;
    const ax = onRight ? left - 10 : left + r.width + 10;
    const ay = top + 14;
    const elbow = onRight ? ax - 26 : ax + 26;
    const dx = elbow - cx, dy = ay - cy;
    const k = Math.min(Math.abs(rx / (dx || 1e-6)), Math.abs(ry / (dy || 1e-6)));
    const sx = cx + dx * k, sy = cy + dy * k;
    d += `M${sx.toFixed(1)} ${sy.toFixed(1)}L${elbow.toFixed(1)} ${ay.toFixed(1)}L${ax.toFixed(1)} ${ay.toFixed(1)}`;
  });
  svg.setAttribute('viewBox', `0 0 ${sr.width} ${sr.height}`);
  svg.innerHTML = `<path d="${d}"/>`;
}

// ---- scroll → dive progress, sunrise, daylight veil ----
const clamp = (v: number) => Math.max(0, Math.min(1, v));
let ticking = false;
function onScroll() {
  ticking = false;
  const y = scrollY, H = innerHeight;
  const top = (sel: string) => $(sel).getBoundingClientRect().top + y;
  const how = top('#how'), data = top('#data'), get = top('#get');
  const cta = top('#about');
  const dive = y < how ? (y / Math.max(1, how)) * 0.5 : 0.5 + clamp((y - how) / Math.max(1, data - how)) * 0.5;
  const climb = clamp((y + H - cta) / H);
  const globeP = y < cta - H ? dive : dive * (1 - climb) + 0.06 * climb;
  sky.update(globeP, Math.min(1, globeP), clamp(1 - y / Math.max(1, how * 0.35)));
  document.documentElement.style.setProperty('--dawn', clamp(globeP).toFixed(3));
  const veil = clamp((y - (get - H * 0.75)) / (H * 0.5));
  $('#ground-veil').style.opacity = veil.toFixed(3);
  document.body.classList.toggle('is-day', y > get - H * 0.4 && y < cta - H * 0.4);
  // credit: the ground plate owns the bottom-right corner while it fills the screen
  const inGround = y + H * 0.5 > data && y + H * 0.5 < cta;
  if (inGround && !sky.isLive()) { $('#hud-credit').textContent = groundCredit; $('#hud-alt').textContent = t('alt', { km: '0.7' }); }
  document.body.classList.toggle('is-ground', inGround);
  if (y + H > data && y < get) drawDataLeaders();
}
addEventListener('scroll', () => { if (!ticking) { ticking = true; requestAnimationFrame(onScroll); } }, { passive: true });
addEventListener('resize', () => { ground.layout(); drawDataLeaders(); onScroll(); });

onLang(() => {
  $('#hero-phone').innerHTML = glassPhoneHtml();
  $('#lbl-valley').innerHTML = `<span>${esc(t('huaral'))}</span><br>11.47°S&nbsp;&nbsp;77.21°W`;
  $('#lbl-field').innerHTML = `<b>${esc(t('field_ha', { ha: fmt(HUARAL_FIELD.area_ha, 1) }))}</b><br><span>${esc(t('peas_now'))}</span><br><span>${esc(t('lettuce_before'))}</span>`;
  $('#lbl-how').textContent = t('ix_valley_label');
  $('#ds-key')?.replaceWith(Object.assign(document.createElement('template'), { innerHTML: anomalyItem() }).content);
  $('#data-field').innerHTML = `<b>${esc(t('field_ha', { ha: fmt(HUARAL_FIELD.area_ha, 1) }))}</b>`;
  document.title = `Field Shift · ${t('ix_hero_title')}`;
  renderPreview($('#dash-preview'));
  requestAnimationFrame(() => { drawDataLeaders(); onScroll(); });
});

document.title = `Field Shift · ${t('ix_hero_title')}`;
document.fonts?.ready.then(() => { drawDataLeaders(); onScroll(); });
onScroll();
drawDataLeaders();
applyI18n(root);
