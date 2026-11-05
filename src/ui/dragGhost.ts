/** A transparent pixel: the drag shows no image of its own. */
const NOTHING = 'data:image/gif;base64,R0lGODlhAQABAIAAAAAAAP///yH5BAEAAAAALAAAAAABAAEAAAIBRAA7';
let nothing: HTMLImageElement | undefined;

/**
 * Starts a drag that shows a copy of `source` under the pointer. The image the browser draws is
 * a rectangle, and in the transparent overlay its corners come out black around rounded tiles.
 */
export function dragWithGhost(event: DragEvent, source: HTMLElement): void {
  if (!event.dataTransfer) return;
  nothing ??= Object.assign(new Image(), { src: NOTHING });
  event.dataTransfer.setDragImage(nothing, 0, 0);

  const rect = source.getBoundingClientRect();
  const grabX = event.clientX - rect.left;
  const grabY = event.clientY - rect.top;
  const ghost = source.cloneNode(true) as HTMLElement;
  ghost.removeAttribute('id');
  Object.assign(ghost.style, {
    position: 'fixed',
    left: '0',
    top: '0',
    width: `${rect.width}px`,
    height: `${rect.height}px`,
    margin: '0',
    pointerEvents: 'none',
    zIndex: '1000',
    opacity: '0.92',
    transform: `translate(${rect.left}px, ${rect.top}px)`,
  });
  ghost.setAttribute('aria-hidden', 'true');
  document.body.append(ghost);

  const follow = (moved: DragEvent) => {
    ghost.style.transform = `translate(${moved.clientX - grabX}px, ${moved.clientY - grabY}px)`;
  };
  const end = () => {
    ghost.remove();
    document.removeEventListener('dragover', follow);
  };
  document.addEventListener('dragover', follow);
  source.addEventListener('dragend', end, { once: true });
}
