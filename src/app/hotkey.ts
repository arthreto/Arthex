import { t, type ShellKey } from '../core/i18n';

/** Modifier names as stored in settings, in the order Rust sorts them. */
export const MODIFIERS = ['Ctrl', 'Alt', 'Shift', 'Win'] as const;

const named: Record<string, ShellKey> = {
  shift: 'keys.shift',
  win: 'keys.win',
  ctrl: 'keys.ctrl',
  alt: 'keys.alt',
  enter: 'keys.enter',
  escape: 'keys.escape',
  tab: 'keys.tab',
};

/** Display label of a key stored in settings (`Shift`, `Win`, `F13`…). */
export function keyLabel(key: string): string {
  const shellKey = named[key.toLowerCase()];
  return shellKey ? t(shellKey) : key;
}

export function formatKeys(keys: readonly string[]): string {
  return keys.map(keyLabel).join(' + ');
}

export type ReservedUse =
  | 'copy'
  | 'paste'
  | 'cut'
  | 'undo'
  | 'selectAll'
  | 'switchWindow'
  | 'closeWindow'
  | 'lock'
  | 'desktop'
  | 'explorer'
  | 'run'
  | 'settings'
  | 'screenshot'
  | 'taskView'
  | 'clipboard';

const reserved: Record<string, ReservedUse> = {
  'Ctrl+C': 'copy',
  'Ctrl+V': 'paste',
  'Ctrl+X': 'cut',
  'Ctrl+Z': 'undo',
  'Ctrl+A': 'selectAll',
  'Alt+Tab': 'switchWindow',
  'Alt+F4': 'closeWindow',
  'Win+L': 'lock',
  'Win+D': 'desktop',
  'Win+E': 'explorer',
  'Win+R': 'run',
  'Win+I': 'settings',
  'Shift+Win+S': 'screenshot',
  'Win+Tab': 'taskView',
  'Win+V': 'clipboard',
};

export type HotkeyProblem =
  { kind: 'modifierOnly' } | { kind: 'typing' } | { kind: 'reserved'; use: ReservedUse };

/** Mirrors `Hotkey::is_usable` in Rust, plus the Windows shortcuts worth keeping. */
export function hotkeyProblem(keys: readonly string[]): HotkeyProblem | null {
  const modifiers = keys.filter((key) => (MODIFIERS as readonly string[]).includes(key));
  const main = keys.find((key) => !(MODIFIERS as readonly string[]).includes(key));
  if (main === undefined && modifiers.length < 2) return { kind: 'modifierOnly' };
  const typing = main !== undefined && /^([A-Z0-9]|Space)$/.test(main);
  if (typing && (modifiers.length === 0 || modifiers.join() === 'Shift')) return { kind: 'typing' };
  const use = reserved[keys.join('+')];
  return use ? { kind: 'reserved', use } : null;
}

const codeNames: Record<string, string> = {
  Space: 'Space',
  PageUp: 'PageUp',
  PageDown: 'PageDown',
  End: 'End',
  Home: 'Home',
  ArrowLeft: 'Left',
  ArrowUp: 'Up',
  ArrowRight: 'Right',
  ArrowDown: 'Down',
  PrintScreen: 'PrintScreen',
  Insert: 'Insert',
  Delete: 'Delete',
  Pause: 'Pause',
  ScrollLock: 'ScrollLock',
};

/** Modifiers held during a DOM keyboard event, in canonical order. */
export function heldModifiers(event: KeyboardEvent): string[] {
  const held = [event.ctrlKey, event.altKey, event.shiftKey, event.metaKey];
  return MODIFIERS.filter((_, index) => held[index]);
}

/** Settings name of the key behind a DOM keyboard event (layout independent), if Arthex accepts it. */
export function keyFromEvent(event: KeyboardEvent): string | null {
  const { code } = event;
  if (/^Key[A-Z]$/.test(code)) return code.slice(3);
  if (/^Digit[0-9]$/.test(code)) return code.slice(5);
  if (/^F([1-9]|1[0-9]|2[0-4])$/.test(code)) return code;
  return codeNames[code] ?? null;
}
