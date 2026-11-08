import { core } from "./core.svelte";

const FALLBACK = "hsl(216 70% 52%)";
const SIZE = 24;

export const art = $state({ color: FALLBACK });

const cache = new Map<string, string>();

// Saturation-weighted average hue of the cover, lifted to a usable accent.
async function accent(url: string) {
  const img = new Image();
  img.src = url;
  await img.decode();
  const ctx = new OffscreenCanvas(SIZE, SIZE).getContext("2d")!;
  ctx.drawImage(img, 0, 0, SIZE, SIZE);
  const px = ctx.getImageData(0, 0, SIZE, SIZE).data;
  let r = 0, g = 0, b = 0, w = 0;
  for (let i = 0; i < px.length; i += 4) {
    const max = Math.max(px[i], px[i + 1], px[i + 2]);
    const min = Math.min(px[i], px[i + 1], px[i + 2]);
    const weight = (max - min) * max + 1;
    r += px[i] * weight;
    g += px[i + 1] * weight;
    b += px[i + 2] * weight;
    w += weight;
  }
  [r, g, b] = [r / w / 255, g / w / 255, b / w / 255];
  const max = Math.max(r, g, b);
  const d = max - Math.min(r, g, b);
  if (!d) return FALLBACK;
  const h = max === r ? ((g - b) / d) % 6 : max === g ? (b - r) / d + 2 : (r - g) / d + 4;
  return `hsl(${Math.round((h * 60 + 360) % 360)} 75% 58%)`;
}

$effect.root(() => {
  $effect(() => {
    const url = core.media?.art;
    if (!url) {
      art.color = FALLBACK;
      return;
    }
    const hit = cache.get(url);
    if (hit) {
      art.color = hit;
      return;
    }
    let live = true;
    accent(url)
      .then((c) => {
        cache.set(url, c);
        if (live) art.color = c;
      })
      .catch(() => live && (art.color = FALLBACK));
    return () => (live = false);
  });
});
