// Index section 5: a frosted browser-like panel previewing the real dashboard markup with the bundled demo data:
// the same top bar as the app (field chip, four views, Download menu, language), readings, the season chart with
// the NASA GEOS-FP forecast and outlook, recommended topics, the 2031–2050 projection, NDVI and pest generations.
import { dashboardHtml } from '../app/workspace/dashboard';
import { mountBackdrop } from '../shared/backdrop';
import { demoAnalysis } from '../shared/demo/huaral';
import { esc } from '../shared/dom';
import { lang, t } from '../shared/i18n';
import { icon, type IconName } from '../shared/icons';
import type { StrKey } from '../shared/strings';

const TABS: [StrKey, IconName, boolean][] = [['ap_tab_chat', 'chat', false], ['ap_tab_dash', 'dashboard', true], ['ap_tab_rot', 'rotation', false], ['ap_tab_org', 'ladybug', false]];

export function renderPreview(host: HTMLElement): void {
  const a = { ...demoAnalysis(lang()), offline: true };
  host.innerHTML = `<div class="preview__win glass" inert>
    <div class="preview__bar"><span class="serif">Field Shift</span>
      <span class="pill">${icon('pin')}${esc(t('huaral'))} · ${a.field.area_ha.toFixed(1)} ha<span class="preview__change">${esc(t('ap_change_field'))}</span></span>
      <span class="preview__tabs">${TABS.map(([k, ic, on]) => `<span${on ? ' class="is-on"' : ''}>${icon(ic)}${esc(t(k))}</span>`).join('')}</span>
      <span class="pill preview__dl">${icon('download')}${esc(t('dl_menu'))}</span>
      <span class="demo-flag">${esc(t('demo_data'))}</span></div>
    ${dashboardHtml(a, { preview: true })}
  </div>`;
  const m = host.querySelector<HTMLElement>('[data-dmap]');
  if (m) mountBackdrop(m, { lat: a.field.center.lat, lon: a.field.center.lon, ring: a.field.ring, sizeM: 1800, zoom: 1, tilesOnly: false });
}
