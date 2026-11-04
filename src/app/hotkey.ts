import { t, type ShellKey } from '../core/i18n';

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
