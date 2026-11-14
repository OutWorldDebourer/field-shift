// A soft two-note chime (WebAudio, no audio file) when a topic is done. Muted by the toggle (saved per device)
// and silent when the visitor prefers reduced motion. The context is unlocked on the user's own send gesture.
import { reducedMotion } from '../../shared/dom';

const KEY = 'fieldshift.chime';
let ctx: AudioContext | null = null;

export function chimeOn(): boolean {
  try { return localStorage.getItem(KEY) !== 'off'; } catch { return true; }
}
export function setChime(on: boolean): void {
  try { localStorage.setItem(KEY, on ? 'on' : 'off'); } catch { /* storage blocked */ }
}

/** Call from a user gesture (send) so the browser lets the chime play later. */
export function unlockAudio(): void {
  if (!chimeOn() || reducedMotion()) return;
  try {
    const AC = window.AudioContext ?? (window as unknown as { webkitAudioContext?: typeof AudioContext }).webkitAudioContext;
    if (!AC) return;
    ctx ??= new AC();
    if (ctx.state === 'suspended') void ctx.resume();
  } catch { ctx = null; }
}

export function chime(): void {
  if (!chimeOn() || reducedMotion() || !ctx) return;
  try {
    const t0 = ctx.currentTime + 0.02;
    [[659.25, 0], [987.77, 0.12]].forEach(([f, dt]) => {
      const o = ctx!.createOscillator(), g = ctx!.createGain();
      o.type = 'sine'; o.frequency.value = f;
      g.gain.setValueAtTime(0, t0 + dt);
      g.gain.linearRampToValueAtTime(0.05, t0 + dt + 0.02);
      g.gain.exponentialRampToValueAtTime(0.0001, t0 + dt + 0.6);
      o.connect(g).connect(ctx!.destination);
      o.start(t0 + dt); o.stop(t0 + dt + 0.65);
    });
  } catch { /* audio unavailable */ }
}
