import { describe, expect, it } from 'vitest';
import { clipKind } from './kind';

describe('clipKind', () => {
  it('recognises links, colours and paths', () => {
    expect(clipKind('https://github.com/arthreto/Arthex')).toBe('link');
    expect(clipKind('#4CC2FF')).toBe('color');
    expect(clipKind('rgb(76, 194, 255)')).toBe('color');
    expect(clipKind('C:\\Users\\Léa\\Documents\\Invoices')).toBe('path');
    expect(clipKind('/home/lea/notes.md')).toBe('path');
  });

  it('tells code from plain text', () => {
    expect(clipKind('npm run tauri build')).toBe('code');
    expect(clipKind('SELECT name, size FROM files ORDER BY size DESC LIMIT 20;')).toBe('code');
    expect(clipKind('const answer = 42;')).toBe('code');
    expect(clipKind('Thanks for your feedback, I will look at it tomorrow!')).toBe('text');
    expect(clipKind('See https://example.org for details')).toBe('text');
  });
});
