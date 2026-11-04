import { describe, expect, it } from 'vitest';
import { guessName, shortAddress, siteProblem } from './site';

describe('siteProblem', () => {
  it('accepts bare hosts and http addresses', () => {
    expect(siteProblem('youtube.com')).toBeNull();
    expect(siteProblem(' http://example.org/a ')).toBeNull();
  });

  it('explains why an address is refused', () => {
    expect(siteProblem('  ')).toBe('missing');
    expect(siteProblem('javascript:alert(1)')).toBe('scheme');
    expect(siteProblem('file:///C:/Windows')).toBe('scheme');
    expect(siteProblem('intranet')).toBe('invalid');
  });
});

describe('shortAddress', () => {
  it('drops the https scheme and the last slash', () => {
    expect(shortAddress('https://github.com/')).toBe('github.com');
    expect(shortAddress('http://intranet.example.org/a/')).toBe('http://intranet.example.org/a');
  });
});

describe('guessName', () => {
  it('uses the first part of the host', () => {
    expect(guessName('https://www.github.com/arthreto')).toBe('Github');
    expect(guessName('mail.google.com')).toBe('Mail');
    expect(guessName('')).toBe('');
  });
});
