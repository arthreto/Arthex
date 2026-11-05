import type { MessageKey, MessageTree, MessageValues } from '../src/core/i18n/types';
import { host } from './host';

let changes = $state(0);
let listening = false;

/** Makes the views that read a text show it again when the language changes. */
function track(): void {
  if (!listening) {
    listening = true;
    host().onLocaleChange(() => (changes += 1));
  }
  void changes;
}

export function getLocale(): string {
  track();
  return host().locale();
}

/** A message by its full key. The UI kit uses it for the shell's own texts (`common.close`). */
export function t(key: string, values?: MessageValues): string {
  track();
  return host().translate(key, values);
}

/** The typed translator of a module, as `moduleTranslator` of the shell. */
export function moduleTranslator<T extends MessageTree>(id: string) {
  return (key: MessageKey<T>, values?: MessageValues): string => t(`modules.${id}.${key}`, values);
}
