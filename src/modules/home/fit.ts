/** Any smaller and the text gets hard to read: the column scrolls at its normal size instead. */
const SMALLEST_SCALE = 0.88;

/** How much to scale content down to fit, or 1 if it fits already or would get too small. */
export function fitScale(available: number, content: number): number {
  if (available <= 0 || content <= available) return 1;
  const scale = Math.floor((available / content) * 1000) / 1000;
  return scale >= SMALLEST_SCALE ? scale : 1;
}
