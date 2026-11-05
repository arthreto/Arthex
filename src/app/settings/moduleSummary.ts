import type { ModuleOffer, ModuleTexts } from '../../core/api';
import { formatBytes } from '../../core/format';
import { getLocale, t } from '../../core/i18n';
import { ELEMENTS, type Element } from '../../core/modules';
import { TINTS, type Tint } from '../../core/modules/manifest';

/** The texts of a module that is not loaded, in the active language if it has it. */
export function pickTexts(texts: ModuleOffer['texts']): ModuleTexts | undefined {
  return texts[getLocale()] ?? texts['en'] ?? Object.values(texts)[0];
}

/** A tint sent by Rust, or blue when it is not one of the tokens. */
export const tintOf = (name: string): Tint => TINTS.find((tint) => tint === name) ?? 'blue';

export const knownElements = (names: readonly string[]): Element[] =>
  ELEMENTS.filter((element) => names.includes(element));

/** "Tool · 117 KB", with the version first when given. */
export function summary(elements: readonly Element[], size?: number, version?: string): string {
  const parts = elements.map((element) => t(`settings.modules.element.${element}`));
  if (version) parts.push(t('settings.modules.version', { version }));
  if (size !== undefined) parts.push(formatBytes(size));
  return parts.join(' · ');
}
