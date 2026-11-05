import { describe, expect, it } from 'vitest';
import { dayName, languageName, rightClickEntries, systemNames } from './moduleDetails';

describe('module details', () => {
  it('names languages in the language of the interface', () => {
    expect(languageName('fr', 'fr')).toBe('Français');
    expect(languageName('en', 'en')).toBe('English');
  });

  it('says nothing of a system for a module that runs everywhere', () => {
    expect(systemNames(['any'])).toBeNull();
    expect(systemNames(['windows-x86_64', 'windows-aarch64'])).toBe('Windows');
    expect(systemNames(['windows-x86_64', 'linux-x86_64'])).toBe('Windows, Linux');
  });

  it('writes a day out and leaves anything else alone', () => {
    expect(dayName('2026-11-05', 'fr')).toBe('5 novembre 2026');
    expect(dayName('soon', 'fr')).toBe('soon');
  });

  it('shows a label once with all its choices', () => {
    const entries = rightClickEntries(
      [
        { id: 'compress', labels: { fr: 'Compresser' }, submenu: [] },
        {
          id: 'image',
          labels: { fr: 'Convertir en', en: 'Convert to' },
          submenu: [{ id: 'png', label: 'PNG' }],
        },
        {
          id: 'video',
          labels: { fr: 'Convertir en', en: 'Convert to' },
          submenu: [
            { id: 'mp4', label: 'MP4' },
            { id: 'png', label: 'PNG' },
          ],
        },
      ],
      'fr',
    );
    expect(entries).toEqual([
      { label: 'Compresser', choices: [] },
      { label: 'Convertir en', choices: ['PNG', 'MP4'] },
    ]);
  });
});
