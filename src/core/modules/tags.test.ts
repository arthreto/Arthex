import { describe, expect, it } from 'vitest';
import { knownTags } from './tags';

describe('knownTags', () => {
  it('keeps known tags only, once each, in catalog order', () => {
    expect(knownTags(['games', 'nope', 'math', 'games'])).toEqual(['math', 'games']);
  });

  it('ignores anything that is not a list', () => {
    expect(knownTags('math')).toEqual([]);
    expect(knownTags(undefined)).toEqual([]);
  });
});
