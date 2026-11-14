// Client-side photo preparation: downscale to ≤1600 px on the long side and re-encode as JPEG before upload.
export const MAX_PHOTOS = 10;
const MAX_SIDE = 1600;

export async function downscale(file: File): Promise<Blob> {
  let bmp: ImageBitmap | HTMLImageElement;
  try { bmp = await createImageBitmap(file, { imageOrientation: 'from-image' } as ImageBitmapOptions); }
  catch {
    bmp = await new Promise<HTMLImageElement>((res, rej) => { const i = new Image(); i.onload = () => res(i); i.onerror = rej; i.src = URL.createObjectURL(file); });
  }
  const w = bmp.width, h = bmp.height;
  const k = Math.min(1, MAX_SIDE / Math.max(w, h));
  if (k === 1 && file.type === 'image/jpeg') return file;
  const c = document.createElement('canvas');
  c.width = Math.round(w * k); c.height = Math.round(h * k);
  c.getContext('2d')!.drawImage(bmp, 0, 0, c.width, c.height);
  return new Promise((res) => c.toBlob((b) => res(b ?? file), 'image/jpeg', 0.86));
}
