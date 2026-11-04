import { describe, expect, it } from 'vitest';
import { matchesAll, normalize } from './text';

describe('text matching', () => {
  it('ignores case and accents', () => {
    expect(normalize('Éditeur de Texte')).toBe('editeur de texte');
  });

  it('needs every word, in any order', () => {
    expect(matchesAll('Visual Studio Code Code.exe', 'code vis')).toBe(true);
    expect(matchesAll('Visual Studio Code', 'code word')).toBe(false);
    expect(matchesAll('Anything', '   ')).toBe(true);
  });
});
