const FOCUSABLE =
  'button:not([disabled]):not([tabindex="-1"]), [href], input:not([disabled]), textarea:not([disabled]), select:not([disabled]), [tabindex]:not([tabindex="-1"])';

/** Keeps keyboard focus inside a dialog while it is open, then gives it back. Without it, a
 * dialog with no text field leaves focus behind it, so Escape closes the overlay instead. */
export function focusTrap(node: HTMLElement) {
  const previous = document.activeElement instanceof HTMLElement ? document.activeElement : null;
  // After the dialog's own fields had their chance to take focus.
  queueMicrotask(() => {
    if (!node.contains(document.activeElement)) node.focus();
  });

  function onkeydown(event: KeyboardEvent) {
    if (event.key !== 'Tab') return;
    const items = [...node.querySelectorAll<HTMLElement>(FOCUSABLE)];
    const first = items[0];
    const last = items.at(-1);
    if (!first || !last) {
      event.preventDefault();
    } else if (event.shiftKey && (document.activeElement === first || document.activeElement === node)) {
      event.preventDefault();
      last.focus();
    } else if (!event.shiftKey && document.activeElement === last) {
      event.preventDefault();
      first.focus();
    }
  }

  node.addEventListener('keydown', onkeydown);
  return {
    destroy() {
      node.removeEventListener('keydown', onkeydown);
      previous?.focus();
    },
  };
}
