import { describe, expect, it } from 'vitest';
import { adopt } from './foreign';

const mount = () => () => {};
const exported = {
  id: 'timer',
  icon: 'M0 0',
  tint: 'green',
  order: 50,
  locales: { en: { name: 'Timer' } },
  tool: () => Promise.resolve(mount),
  widget: { slot: 'side', component: () => Promise.resolve(mount) },
};

describe('adopt', () => {
  it('turns what a module exports into a manifest', () => {
    const manifest = adopt(exported, 'timer');
    expect(manifest?.id).toBe('timer');
    expect(manifest?.tint).toBe('green');
    expect(typeof manifest?.tool).toBe('function');
    expect(manifest?.widget?.slot).toBe('side');
    expect(manifest?.page).toBeUndefined();
  });

  it('wraps each view in a shell component', async () => {
    const loaded = await adopt(exported, 'timer')?.tool?.();
    expect(typeof loaded?.default).toBe('function');
  });

  it('refuses another id or a broken manifest', () => {
    expect(adopt(exported, 'other')).toBeNull();
    expect(adopt({ ...exported, tint: 'gold' }, 'timer')).toBeNull();
    expect(adopt({ ...exported, locales: undefined }, 'timer')).toBeNull();
    expect(adopt('timer', 'timer')).toBeNull();
  });

  it('drops elements it cannot use', () => {
    const manifest = adopt(
      { ...exported, widget: { slot: 'floating', component: exported.tool }, search: { order: 1 } },
      'timer',
    );
    expect(manifest?.widget).toBeUndefined();
    expect(manifest?.search).toBeUndefined();
  });
});
