import { describe, expect, it } from 'vitest';
import { hotkeyProblem, keyFromEvent } from './hotkey';

describe('hotkeyProblem', () => {
  it('accepts the default and real combinations', () => {
    expect(hotkeyProblem(['Shift', 'Win'])).toBeNull();
    expect(hotkeyProblem(['Ctrl', 'Alt', 'Space'])).toBeNull();
    expect(hotkeyProblem(['F13'])).toBeNull();
  });

  it('rejects keys that would fire all the time or break typing', () => {
    expect(hotkeyProblem(['Shift'])).toEqual({ kind: 'modifierOnly' });
    expect(hotkeyProblem(['A'])).toEqual({ kind: 'typing' });
    expect(hotkeyProblem(['Shift', '4'])).toEqual({ kind: 'typing' });
    expect(hotkeyProblem(['Space'])).toEqual({ kind: 'typing' });
  });

  it('keeps the main Windows shortcuts', () => {
    expect(hotkeyProblem(['Win', 'L'])).toEqual({ kind: 'reserved', use: 'lock' });
    expect(hotkeyProblem(['Shift', 'Win', 'S'])).toEqual({ kind: 'reserved', use: 'screenshot' });
  });
});

describe('keyFromEvent', () => {
  const key = (code: string) => keyFromEvent({ code } as KeyboardEvent);

  it('names keys by position, not by layout', () => {
    expect(key('KeyQ')).toBe('Q');
    expect(key('Digit7')).toBe('7');
    expect(key('F24')).toBe('F24');
    expect(key('ArrowUp')).toBe('Up');
  });

  it('ignores keys a hotkey cannot use', () => {
    expect(key('F25')).toBeNull();
    expect(key('Escape')).toBeNull();
    expect(key('ShiftLeft')).toBeNull();
  });
});
