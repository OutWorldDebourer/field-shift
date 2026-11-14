// The frosted-glass phone from the approved hero (master component), filled from the bundled demo data.
import { DEMO_EVIDENCE, HUARAL_FIELD } from './demo/huaral';
import { dataRowCompact } from './datarow';
import { esc } from './dom';
import { fmt, t } from './i18n';
import { icon } from './icons';

export function glassPhoneHtml(): string {
  const r = (id: string) => DEMO_EVIDENCE.readings.find((x) => x.id === id)!;
  return `<figure class="phone glass" aria-label="${esc(t('ix_phone_label'))}">
    <div class="phone__status mono" aria-hidden="true"><span>09:24</span><i class="phone__cam"></i><span class="phone__sig"><b></b><b></b><b></b><b></b></span></div>
    <div class="phone__bar"><span class="serif phone__brand">Field Shift</span>${icon('menu', 'phone__menu')}</div>
    <div class="phone__chip">${icon('pin')}<span>${esc(t('huaral'))} · ${fmt(HUARAL_FIELD.area_ha, 1)} ha</span></div>
    <div class="msg"><span class="avatar-ai">${icon('spark')}</span><p class="bubble bubble--ai">${esc(t('bubble_closer'))}</p></div>
    <div class="phone__tiles">
      <div class="tile"><img src="/img/pea-plant.jpg" alt="${esc(t('photo_pea_alt'))}" width="170" height="146" decoding="async"></div>
      <div class="tile"><img src="/img/pea-leaf.jpg" alt="${esc(t('photo_leaf_alt'))}" width="154" height="146" decoding="async"></div>
    </div>
    <div class="phone__rows">
      ${dataRowCompact(r('temp_anomaly'), 'NASA POWER · 1984–2026')}
      ${dataRowCompact(r('soil_moisture'), 'SMAP · 9 km')}
      ${dataRowCompact(r('ndvi'), 'Sentinel-2 · 10 m')}
    </div>
    <figcaption class="phone__foot mono">${esc(t('demo_data'))} · ${esc(t('huaral'))}</figcaption>
  </figure>`;
}
