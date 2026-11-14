// Placeholder until the globe module is delivered: renders nothing but honours the contract.
import type { GlobeHandle, GlobeOptions } from './types';
export type * from './types';

export function createGlobe(host: HTMLElement, opts: GlobeOptions): GlobeHandle {
  host.dataset.globe = 'placeholder';
  opts.onReady?.();
  return { setProgress() {}, lockOn() {}, setLang() {}, resize() {}, dispose() {} };
}
