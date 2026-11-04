import { describe, expect, it } from 'vitest';
import { releaseNotes } from './releaseNotes';

const NOTES = `### English
#### New
- Clipboard history
#### Fixed
- The Start menu no longer opens

### Français
#### Nouveau
- Historique du presse-papiers
#### Amélioré
#### Corrigé
- Le menu Démarrer ne s’ouvre plus`;

describe('releaseNotes', () => {
  it('reads the section of the language and skips empty parts', () => {
    expect(releaseNotes(NOTES, true)).toEqual([
      { kind: 'new', items: ['Historique du presse-papiers'] },
      { kind: 'fixed', items: ['Le menu Démarrer ne s’ouvre plus'] },
    ]);
    expect(releaseNotes(NOTES, false)[1]).toEqual({
      kind: 'fixed',
      items: ['The Start menu no longer opens'],
    });
  });

  it('falls back to English, and to nothing for free text', () => {
    expect(releaseNotes('### English\n#### New\n- One', true)).toEqual([{ kind: 'new', items: ['One'] }]);
    expect(releaseNotes('Bug fixes.', false)).toEqual([]);
  });
});
