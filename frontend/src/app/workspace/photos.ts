// Client-side photo preparation at full resolution: originals go up untouched unless they are larger than
// 12 MB or 4096 px on the long side; only then are they re-encoded (JPEG, quality 0.92, ≤ 4096 px).
export const MAX_PHOTOS = 10;
const MAX_SIDE = 4096;
const MAX_BYTES = 12 * 1024 * 1024;
const QUALITY = 0.92;

async function decode(file: File): Promise<ImageBitmap | HTMLImageElement> {
  try { return await createImageBitmap(file, { imageOrientation: 'from-image' } as ImageBitmapOptions); }
  catch {
    return new Promise<HTMLImageElement>((res, rej) => {
      const u = URL.createObjectURL(file);
      const i = new Image();
      i.onload = () => { URL.revokeObjectURL(u); res(i); };
      i.onerror = (e) => { URL.revokeObjectURL(u); rej(e); };
      i.src = u;
    });
  }
}

const encode = (c: HTMLCanvasElement, q: number): Promise<Blob | null> => new Promise((res) => c.toBlob((b) => res(b), 'image/jpeg', q));

/** The photo to upload: the original when it fits, else a high-quality JPEG of at most 4096 px. */
export async function preparePhoto(file: File): Promise<Blob> {
  let img: ImageBitmap | HTMLImageElement;
  try { img = await decode(file); } catch { return file; } // undecodable here (e.g. HEIC): send as is
  const w = 'naturalWidth' in img ? img.naturalWidth : img.width, h = 'naturalHeight' in img ? img.naturalHeight : img.height;
  if (file.size <= MAX_BYTES && Math.max(w, h) <= MAX_SIDE) { if ('close' in img) img.close(); return file; }
  const k = Math.min(1, MAX_SIDE / Math.max(w, h));
  const c = document.createElement('canvas');
  c.width = Math.round(w * k); c.height = Math.round(h * k);
  const ctx = c.getContext('2d')!;
  ctx.imageSmoothingQuality = 'high';
  ctx.drawImage(img, 0, 0, c.width, c.height);
  if ('close' in img) img.close();
  let out = await encode(c, QUALITY);
  if (out && out.size > MAX_BYTES) out = await encode(c, 0.88);
  return out ?? file;
}

/** "3.4 MB", "820 KB". */
export function sizeText(bytes: number, locale: string): string {
  const mb = bytes / (1024 * 1024);
  if (mb >= 1) return `${mb.toLocaleString(locale, { maximumFractionDigits: 1 })} MB`;
  return `${Math.max(1, Math.round(bytes / 1024)).toLocaleString(locale)} KB`;
}
