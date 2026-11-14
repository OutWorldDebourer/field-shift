// The frosted-glass phone from the approved hero (master component), filled from the bundled demo data.
import { DEMO_EVIDENCE, HUARAL_FIELD } from './demo/huaral';
import { dataRowCompact, shortRes } from './datarow';
import { keyReadings, normalPeriod, shortLabel } from './evidence';
import { esc } from './dom';
import { fmt, fmtDateShort, t } from './i18n';
import { icon } from './icons';

export function glassPhoneHtml(): string {
  return `<figure class="phone glass" aria-label="${esc(t('ix_phone_label'))}">
    <div class="phone__status mono" aria-hidden="true"><span>09:24</span><i class="phone__cam"></i><span class="phone__sig"><b></b><b></b><b></b><b></b></span></div>
    <div class="phone__bar"><span class="serif phone__brand">Field Shift</span>${icon('menu', 'phone__menu')}</div>
    <div class="phone__chip">${icon('pin')}<span>${esc(t('huaral'))} · ${fmt(HUARAL_FIELD.area_ha, 1)} ha</span></div>
    <div class="msg"><span class="avatar-ai">${icon('advisor')}</span><p class="bubble bubble--ai">${esc(t('bubble_closer'))}</p></div>
    <div class="phone__tiles">
      <div class="tile"><img src="/img/pea-plant.jpg" alt="${esc(t('photo_pea_alt'))}" width="170" height="146" decoding="async"></div>
      <div class="tile"><img src="/img/pea-leaf.jpg" alt="${esc(t('photo_leaf_alt'))}" width="154" height="146" decoding="async"></div>
    </div>
    <div class="phone__rows">
      ${keyReadings(DEMO_EVIDENCE).map((r) => dataRowCompact(r, r.id.startsWith('temp') && normalPeriod(DEMO_EVIDENCE)
        ? `${r.source} · ${fmtDateShort(r.date)} · vs ${normalPeriod(DEMO_EVIDENCE)}`
        : `${r.source.replace(/\s+L2A$/, '')} · ${shortRes(r.resolution)}`, shortLabel(r))).join('')}
    </div>
    <figcaption class="phone__foot mono">${esc(t('demo_data'))} · ${esc(t('huaral'))}</figcaption>
  </figure>`;
}
