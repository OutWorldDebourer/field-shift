// Index section 5: a frosted browser-like panel previewing the real dashboard markup with the bundled demo data.
import { dashboardHtml } from '../app/workspace/dashboard';
import { mountBackdrop } from '../shared/backdrop';
import { demoAnalysis } from '../shared/demo/huaral';
import { esc } from '../shared/dom';
import { lang, t } from '../shared/i18n';
import { icon } from '../shared/icons';

export function renderPreview(host: HTMLElement): void {
  const a = demoAnalysis(lang());
  host.innerHTML = `<div class="preview__win glass" inert>
    <div class="preview__bar"><span class="serif">Field Shift</span>
      <span class="pill">${icon('pin')}${esc(t('huaral'))} · ${a.field.area_ha.toFixed(1)} ha</span>
      <span class="preview__tabs"><span>${esc(t('ap_tab_chat'))}</span><span class="is-on">${esc(t('ap_tab_dash'))}</span><span>${esc(t('ap_tab_rot'))}</span></span>
      <span class="demo-flag">${esc(t('demo_data'))}</span></div>
    ${dashboardHtml(a, { preview: true })}
  </div>`;
  const m = host.querySelector<HTMLElement>('[data-dmap]');
  if (m) mountBackdrop(m, { lat: a.field.center.lat, lon: a.field.center.lon, ring: a.field.ring, sizeM: 1800, zoom: 1, tilesOnly: false });
}
