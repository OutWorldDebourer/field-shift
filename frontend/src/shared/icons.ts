// Hand-drawn line icons (24px grid, 1.6 stroke, currentColor). One consistent set for both pages.
const P: Record<string, string> = {
  thermo: '<path d="M10 14.5V5a2 2 0 1 1 4 0v9.5a4 4 0 1 1-4 0Z"/><path d="M12 9v7.5"/><circle cx="12" cy="17.6" r="1.4" fill="currentColor" stroke="none"/>',
  drop: '<path d="M12 3.5c3.2 4 5.5 7.2 5.5 10.2a5.5 5.5 0 0 1-11 0C6.5 10.7 8.8 7.5 12 3.5Z"/>',
  leaf: '<path d="M5 19c0-8 5-13 14-14-.6 9-5.6 14-13 14"/><path d="M5 19 13 11"/>',
  rain: '<path d="M7 15a4 4 0 0 1-.3-8A5.5 5.5 0 0 1 17.4 8 3.5 3.5 0 0 1 17 15"/><path d="M9 18l-1 2.5M13 17l-1 3M17 18l-1 2.5"/>',
  balance: '<path d="M12 4v16M5 20h14"/><path d="M4 9h16"/><path d="M4 9l-2 5a2.5 2.5 0 0 0 4 0Zm16 0-2 5a2.5 2.5 0 0 0 4 0Z"/>',
  bug: '<rect x="8" y="8" width="8" height="11" rx="4"/><path d="M12 8V19M9.5 6.5 8 4.5M14.5 6.5 16 4.5M8 12H4.5M16 12h3.5M8 16l-3 1.5M16 16l3 1.5"/>',
  slope: '<path d="M3 19h18L21 7Z"/><path d="M15 19a6 6 0 0 0-1.4-3.9"/>',
  water: '<path d="M3 15c2 0 2-1.5 4.5-1.5S10 15 12 15s2-1.5 4.5-1.5S19 15 21 15"/><path d="M3 19c2 0 2-1.5 4.5-1.5S10 19 12 19s2-1.5 4.5-1.5S19 19 21 19"/><path d="M12 3v8M9 8l3 3 3-3"/>',
  orbit: '<circle cx="12" cy="12" r="3.5"/><ellipse cx="12" cy="12" rx="9.5" ry="4" transform="rotate(-25 12 12)"/>',
  spark: '<path d="M12 3v5M12 16v5M3 12h5M16 12h5M6 6l2.6 2.6M15.4 15.4 18 18M6 18l2.6-2.6M15.4 8.6 18 6"/>',
  pin: '<path d="M12 21s-6.5-6.2-6.5-11a6.5 6.5 0 0 1 13 0c0 4.8-6.5 11-6.5 11Z"/><circle cx="12" cy="10" r="2.3"/>',
  gps: '<circle cx="12" cy="12" r="4"/><circle cx="12" cy="12" r="1" fill="currentColor"/><path d="M12 2.5v3M12 18.5v3M2.5 12h3M18.5 12h3"/>',
  camera: '<path d="M4 8h3l1.6-2.5h6.8L17 8h3v11H4Z"/><circle cx="12" cy="13.2" r="3.6"/>',
  send: '<path d="M4 12 20 4l-5.5 16-3-6.5Z"/><path d="M11.5 13.5 20 4"/>',
  arrow: '<path d="M4 12h15M13 6l6 6-6 6"/>',
  back: '<path d="M20 12H5M11 6l-6 6 6 6"/>',
  lock: '<rect x="5.5" y="10.5" width="13" height="9.5" rx="2"/><path d="M8.5 10.5V8a3.5 3.5 0 0 1 7 0v2.5"/>',
  sheet: '<path d="M6 3h8l4 4v14H6Z"/><path d="M14 3v4h4M9 12h6M9 15.5h6M9 9h2"/>',
  csv: '<path d="M6 3h8l4 4v14H6Z"/><path d="M14 3v4h4M9 11v7M12 11v7M15 11v7M9 14.5h6"/>',
  link: '<path d="M10 14a4 4 0 0 0 5.7 0l3-3a4 4 0 0 0-5.7-5.7l-1 1"/><path d="M14 10a4 4 0 0 0-5.7 0l-3 3a4 4 0 0 0 5.7 5.7l1-1"/>',
  menu: '<path d="M4 7h16M4 12h16M4 17h16"/>',
  close: '<path d="M6 6l12 12M18 6 6 18"/>',
  search: '<circle cx="10.5" cy="10.5" r="6"/><path d="m15 15 5 5"/>',
  polygon: '<path d="M5 7l9-3 6 7-5 9-10-3Z"/><circle cx="5" cy="7" r="1.6" fill="currentColor"/><circle cx="14" cy="4" r="1.6" fill="currentColor"/><circle cx="20" cy="11" r="1.6" fill="currentColor"/><circle cx="15" cy="20" r="1.6" fill="currentColor"/><circle cx="5" cy="17" r="1.6" fill="currentColor"/>',
  point: '<circle cx="12" cy="12" r="3" fill="currentColor"/><path d="M12 3v4M12 17v4M3 12h4M17 12h4"/>',
  trash: '<path d="M5 7h14M9.5 7V4.5h5V7M7 7l1 13h8l1-13"/>',
  layers: '<path d="m12 4 9 4.5-9 4.5-9-4.5Z"/><path d="m3 13 9 4.5 9-4.5"/>',
  alert: '<path d="M12 4 21 19H3Z"/><path d="M12 10v4.5"/><circle cx="12" cy="16.8" r=".9" fill="currentColor"/>',
  info: '<circle cx="12" cy="12" r="9"/><path d="M12 11v5.5"/><circle cx="12" cy="7.8" r=".9" fill="currentColor"/>',
  soil: '<path d="M3 10h18"/><path d="M12 10V5M12 5c0-1.5 1.5-2.5 3.5-2.5C15.5 4 14 5 12 5ZM12 6.5C12 5 10.5 4 8.5 4 8.5 5.5 10 6.5 12 6.5Z"/><path d="M6 14h.01M10 16h.01M15 14h.01M18 17h.01M8 19h.01M13 19.5h.01" stroke-width="2.4"/>',
  nutrition: '<path d="M8 3h8M9 3v5L4.5 18a2 2 0 0 0 1.8 3h11.4a2 2 0 0 0 1.8-3L15 8V3"/><path d="M7 14h10"/>',
  rotation: '<path d="M20 12a8 8 0 0 1-14.3 4.9M4 12a8 8 0 0 1 14.3-4.9"/><path d="M18.5 3v4.2h-4.2M5.5 21v-4.2h4.2"/>',
  field: '<path d="M4 6l8-3 8 3v12l-8 3-8-3Z"/><path d="M4 12l8 3 8-3M12 3v18"/>',
  chat: '<path d="M4 5h16v11H9l-5 4Z"/>',
  dashboard: '<rect x="3.5" y="4" width="7" height="7" rx="1.5"/><rect x="13.5" y="4" width="7" height="4.5" rx="1.5"/><rect x="13.5" y="11.5" width="7" height="8.5" rx="1.5"/><rect x="3.5" y="14" width="7" height="6" rx="1.5"/>',
  check: '<path d="m5 12.5 4.5 4.5L19 7.5"/>',
  chevron: '<path d="m9 6 6 6-6 6"/>',
  down: '<path d="m6 9 6 6 6-6"/>',
  mic: '<rect x="9" y="3" width="6" height="11" rx="3"/><path d="M5.5 11a6.5 6.5 0 0 0 13 0M12 17.5V21M8.5 21h7"/>',
  stop: '<rect x="7" y="7" width="10" height="10" rx="2" fill="currentColor" stroke="none"/>',
  sound: '<path d="M4 9.5h3.5L12 5.5v13l-4.5-4H4Z"/><path d="M15.5 9a4 4 0 0 1 0 6M18 6.5a7.5 7.5 0 0 1 0 11"/>',
  mute: '<path d="M4 9.5h3.5L12 5.5v13l-4.5-4H4Z"/><path d="m16 9.5 5 5M21 9.5l-5 5"/>',
  book: '<path d="M4 5.5A2.5 2.5 0 0 1 6.5 3H20v15H6.5A2.5 2.5 0 0 0 4 20.5Z"/><path d="M4 20.5A2.5 2.5 0 0 0 6.5 23H20v-5"/>',
  download: '<path d="M12 4v11M7 10.5l5 5 5-5M5 20h14"/>',
  // iteration 3
  ladybug: '<path d="M12 7.5a6.5 6.5 0 0 1 6.5 6.5v.5a6.5 6.5 0 0 1-13 0V14A6.5 6.5 0 0 1 12 7.5Z"/><path d="M9.2 8.2a2.8 2.8 0 0 1 5.6 0M12 7.5V21M10.2 5.8 8.6 3.6M13.8 5.8l1.6-2.2"/><circle cx="8.8" cy="12.6" r=".9" fill="currentColor"/><circle cx="15.2" cy="12.6" r=".9" fill="currentColor"/><circle cx="9.4" cy="16.8" r=".9" fill="currentColor"/><circle cx="14.6" cy="16.8" r=".9" fill="currentColor"/>',
  clipboard: '<rect x="5" y="4.5" width="14" height="16.5" rx="2"/><path d="M9 4.5V3h6v1.5M8.5 10h7M8.5 13.5h7M8.5 17h4"/>',
  pdf: '<path d="M6 3h8l4 4v14H6Z"/><path d="M14 3v4h4"/><path d="M8.6 17.5v-4h1.3a1.2 1.2 0 0 1 0 2.4H8.6M12.4 13.5v4h.9a1.6 1.6 0 0 0 1.6-1.6v-.8a1.6 1.6 0 0 0-1.6-1.6ZM16.8 17.5v-4h-.01M16.8 13.5h1.5M16.8 15.5h1.2" stroke-width="1.3"/>',
  external: '<path d="M14 4h6v6M20 4l-8.5 8.5"/><path d="M18 14v5a1 1 0 0 1-1 1H5a1 1 0 0 1-1-1V7a1 1 0 0 1 1-1h5"/>',
};

export type IconName = keyof typeof P;

export function icon(name: IconName, cls = ''): string {
  return `<svg class="ico ${cls}" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true" focusable="false">${P[name]}</svg>`;
}

/** Icon for a reading id from the evidence package. */
export function readingIcon(id: string): IconName {
  if (id.startsWith('temp') || id === 'heat' || id === 'projection') return 'thermo';
  if (id.startsWith('soil')) return 'drop';
  if (id === 'ndvi' || id.startsWith('vigor')) return 'leaf';
  if (id.startsWith('rain')) return 'rain';
  if (id.startsWith('water')) return 'balance';
  if (id.startsWith('pest')) return 'bug';
  if (id === 'slope') return 'slope';
  if (id === 'grace') return 'water';
  return 'orbit';
}
