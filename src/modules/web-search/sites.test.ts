import { describe, expect, it } from 'vitest';
import { ENGINES, address, keyword, knownSites, searchUrl } from './sites';

describe('keyword', () => {
  it('picks the engine typed before the words', () => {
    expect(keyword('yt lofi hip hop')?.engine.name).toBe('YouTube');
    expect(keyword('YT lofi')?.terms).toBe('lofi');
    expect(keyword('g')).toBeNull();
    expect(keyword('lofi hip hop')).toBeNull();
  });
});

describe('address', () => {
  it('recognises web addresses only', () => {
    expect(address('github.com')).toBe('https://github.com');
    expect(address('fr.wikipedia.org/wiki/Chat')).toBe('https://fr.wikipedia.org/wiki/Chat');
    expect(address('http://localhost.dev:8080')).toBe('http://localhost.dev:8080');
    expect(address('report.pdf')).toBeNull();
    expect(address('notes.txt')).toBeNull();
    expect(address('github')).toBeNull();
    expect(address('javascript:alert(1)')).toBeNull();
  });
});

describe('knownSites', () => {
  it('matches the start of a name from three letters', () => {
    expect(knownSites('git').map((site) => site.name)).toEqual(['GitHub']);
    expect(knownSites('gi')).toEqual([]);
    expect(knownSites('hub')).toEqual([]);
  });
});

describe('searchUrl', () => {
  it('encodes the words', () => {
    const youtube = ENGINES['yt'];
    expect(youtube && searchUrl(youtube, 'a & b', 'fr')).toBe(
      'https://www.youtube.com/results?search_query=a%20%26%20b',
    );
    const wikipedia = ENGINES['w'];
    expect(wikipedia && searchUrl(wikipedia, 'chat', 'fr')).toBe(
      'https://fr.wikipedia.org/w/index.php?search=chat',
    );
  });
});
