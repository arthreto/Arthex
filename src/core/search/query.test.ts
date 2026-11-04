import { describe, expect, it } from 'vitest';
import { mergeResults, parseQuery } from './query';
import type { SearchResult } from './types';

const result = (key: string): SearchResult => ({
  key,
  group: 'g',
  title: key,
  icon: { kind: 'file' },
  kind: 'k',
  primary: { label: 'Open', run: () => undefined },
});

describe('parseQuery', () => {
  it('normalizes and detects the command prefix', () => {
    expect(parseQuery('  Éditeur ')).toEqual({ text: 'Éditeur', normalized: 'editeur', commandsOnly: false });
    expect(parseQuery('> veille')).toEqual({ text: 'veille', normalized: 'veille', commandsOnly: true });
  });
});

describe('mergeResults', () => {
  it('orders by provider and lets a calculation stand alone', () => {
    const apps = { order: 10, exclusive: false, results: [result('app')] };
    const files = { order: 40, exclusive: false, results: [result('file')] };
    expect(mergeResults([files, apps]).map((r) => r.key)).toEqual(['app', 'file']);
    const calc = { order: 0, exclusive: true, results: [result('calc')] };
    expect(mergeResults([files, calc, apps]).map((r) => r.key)).toEqual(['calc']);
    const noCalc = { order: 0, exclusive: true, results: [] };
    expect(mergeResults([files, noCalc, apps]).map((r) => r.key)).toEqual(['app', 'file']);
  });
});
