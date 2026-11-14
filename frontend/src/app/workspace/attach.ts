// Chat attachments since iteration 3: photos and PDF documents (e.g. an earlier technical sheet).
// PDFs go up untouched (≤ 15 MB each) with their real file name; photos keep the iteration-2 rules (photos.ts).
// Renders the composer previews and the message chips/tiles for both kinds.
import { esc } from '../../shared/dom';
import { lang, t } from '../../shared/i18n';
import { icon } from '../../shared/icons';
import { sizeText } from './photos';
import './attach.css';

export const MAX_PDF_BYTES = 15 * 1024 * 1024; // backend limit: the base64 PDF must fit the model request
export const ACCEPT = 'image/*,application/pdf';

/** A file waiting in the composer. `url` is an object URL (image thumbnail, or the PDF to open). */
export interface Pending { blob: Blob; url: string; kind: 'image' | 'pdf'; name: string }

/** Attachment as the backend sends it in ChatMessage.photos (iteration 3 adds kind, name, bytes). */
export type Attachment = { url: string; tag?: string; kind?: 'image' | 'pdf' | string; name?: string; bytes?: number };

export const isPdfFile = (f: File): boolean => f.type === 'application/pdf' || (!f.type && /\.pdf$/i.test(f.name));
export const isPdfAttachment = (p: Attachment): boolean =>
  p.kind === 'pdf' || /\.pdf($|\?)/i.test(p.name ?? '') || /\.pdf($|\?)/i.test(p.url);

/** PDFs with an empty MIME type (some systems) are re-typed so the backend accepts them. */
export const asPdf = (f: File): File => (f.type === 'application/pdf' ? f : new File([f], f.name, { type: 'application/pdf', lastModified: f.lastModified }));

const loc = () => (lang() === 'es' ? 'es-PE' : 'en-US');
const baseName = (p: Attachment) => p.name || decodeURIComponent(p.url.split('?')[0].split('/').pop() || '') || t('ws_doc');

/** Composer preview item: thumbnail for a photo, document tile (icon, name, size) for a PDF. */
export function previewHtml(f: Pending, i: number): string {
  const size = sizeText(f.blob.size, loc());
  if (f.kind === 'pdf') {
    return `<li class="composer__doc" title="${esc(f.name)}">${icon('pdf', 'composer__docico')}<span class="composer__docname">${esc(f.name)}</span><span class="composer__docsize mono">PDF · ${esc(size)}</span>
      <button type="button" class="composer__rm" data-rm="${i}" aria-label="${esc(t('ws_remove_file', { name: f.name }))}">${icon('close')}</button></li>`;
  }
  return `<li><img src="${f.url}" alt="${esc(t('ws_photo_size', { n: i + 1, s: size }))}"><span class="composer__size mono" aria-hidden="true">${esc(size)}</span><button type="button" class="composer__rm" data-rm="${i}" aria-label="${esc(t('ws_remove_photo', { n: i + 1 }))}">${icon('close')}</button></li>`;
}

/** Document chip in a message bubble: opens the PDF in a new tab. */
export function docChipHtml(p: Attachment): string {
  const name = baseName(p);
  const size = typeof p.bytes === 'number' && p.bytes > 0 ? ` · ${sizeText(p.bytes, loc())}` : '';
  return `<a class="doc-chip" href="${esc(p.url)}" target="_blank" rel="noopener" aria-label="${esc(t('ws_doc_open', { name }))}">
    ${icon('pdf', 'doc-chip__ico')}<span class="doc-chip__t"><span class="doc-chip__name">${esc(name)}</span><span class="doc-chip__meta mono">PDF${esc(size)}</span></span>${icon('external', 'doc-chip__go')}</a>`;
}
