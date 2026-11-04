import { describe, expect, it } from 'vitest';
import { register } from './catalog';
import { resolveLocale, setLocale, translate } from './i18n.svelte';

register('en', 'test', { hello: 'Hello {name}', items: '{count, plural, one {# item} other {# items}}' });
register('fr', 'test', { hello: 'Bonjour {name}' });

describe('translate', () => {
  it('formats variables and plurals', () => {
    setLocale('en');
    expect(translate('test.hello', { name: 'Ada' })).toBe('Hello Ada');
    expect(translate('test.items', { count: 1 })).toBe('1 item');
    expect(translate('test.items', { count: 3 })).toBe('3 items');
  });

  it('falls back to English for a missing message, then to the key', () => {
    setLocale('fr');
    expect(translate('test.hello', { name: 'Ada' })).toBe('Bonjour Ada');
    expect(translate('test.items', { count: 2 })).toBe('2 items');
    expect(translate('test.unknown')).toBe('test.unknown');
  });
});

describe('resolveLocale', () => {
  it('matches on the language and falls back to English', () => {
    expect(resolveLocale('fr-CA')).toBe('fr');
    expect(resolveLocale(null, 'de-DE')).toBe('en');
    expect(resolveLocale(undefined, 'fr_FR')).toBe('fr');
  });
});
