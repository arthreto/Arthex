import { describe, expect, it } from 'vitest';
import {
  elementsOf,
  isActive,
  isEnabled,
  launcherIds,
  moveInBar,
  placePage,
  switchElement,
  tabBarIds,
  toggleFavorite,
  zoneIds,
} from './layout';
import type { ModuleManifest } from './manifest';

const view = () => Promise.resolve({ default: () => ({}) });

const page = (id: string, order: number, locked = false): ModuleManifest => ({
  id,
  icon: '',
  tint: 'blue',
  order,
  locked,
  page: view,
});

const catalog = [page('home', 0, true), page('filesize', 1), page('notes', 2), page('clipboard', 3)];

describe('elements', () => {
  const notes: ModuleManifest = { ...page('notes', 2), widget: { slot: 'side', component: view } };
  const both = [page('home', 0, true), notes];

  it('lists what a module declares', () => {
    expect(elementsOf(notes)).toEqual(['page', 'widget']);
  });

  it('turns elements off one by one', () => {
    const layout = { disabledElements: ['notes:widget'], tabBar: [], favorites: [] };
    expect(isEnabled(both, layout, 'notes', 'page')).toBe(true);
    expect(isEnabled(both, layout, 'notes', 'widget')).toBe(false);
    expect(isEnabled(both, layout, 'notes', 'search')).toBe(false);
    expect(isActive(both, layout, 'notes')).toBe(true);
    const off = { ...layout, disabledElements: ['notes:widget', 'notes:page'] };
    expect(isActive(both, off, 'notes')).toBe(false);
  });

  it('never turns off a locked module', () => {
    const layout = { disabledElements: ['home:page'], tabBar: [], favorites: [] };
    expect(isEnabled(both, layout, 'home', 'page')).toBe(true);
  });

  it('switches an element without duplicates', () => {
    expect(switchElement(['a:page'], 'b', 'tool', false)).toEqual(['a:page', 'b:tool']);
    expect(switchElement(['a:page', 'b:tool'], 'b', 'tool', false)).toEqual(['a:page', 'b:tool']);
    expect(switchElement(['a:page', 'b:tool'], 'a', 'page', true)).toEqual(['b:tool']);
  });
});

describe('tabBarIds', () => {
  it('keeps home first and drops disabled or unknown pages', () => {
    const layout = {
      disabledElements: ['notes:page'],
      tabBar: ['notes', 'ghost', 'clipboard', 'home'],
      favorites: [],
    };
    expect(tabBarIds(catalog, layout)).toEqual(['home', 'clipboard']);
  });

  it('never lets the home page be turned off', () => {
    const layout = { disabledElements: ['home:page'], tabBar: [], favorites: [] };
    expect(tabBarIds(catalog, layout)).toEqual(['home']);
  });
});

describe('launcherIds', () => {
  it('lists favourites first, then the others, then disabled pages', () => {
    const layout = { disabledElements: ['filesize:page'], tabBar: [], favorites: ['clipboard'] };
    expect(launcherIds(catalog, layout)).toEqual(['clipboard', 'home', 'notes', 'filesize']);
  });

  it('ignores favourites that are disabled', () => {
    const layout = { disabledElements: ['notes:page'], tabBar: [], favorites: ['notes'] };
    expect(launcherIds(catalog, layout)).toEqual(['home', 'filesize', 'clipboard', 'notes']);
  });
});

describe('toggleFavorite', () => {
  it('adds at the end and removes in place', () => {
    expect(toggleFavorite(['a'], 'b')).toEqual(['a', 'b']);
    expect(toggleFavorite(['a', 'b'], 'a')).toEqual(['b']);
  });
});

describe('customize', () => {
  const layout = { disabledElements: ['clipboard:page'], tabBar: ['home', 'notes'], favorites: [] };

  it('splits pages into the bar, the others and the turned-off ones', () => {
    expect(zoneIds(catalog, layout)).toEqual({
      bar: ['home', 'notes'],
      other: ['filesize'],
      off: ['clipboard'],
    });
  });

  it('inserts into the bar after home and turns pages back on', () => {
    const placed = placePage(catalog, layout, 'clipboard', 'bar', 'home');
    expect(placed).toEqual({
      ok: true,
      patch: { tabBar: ['home', 'clipboard', 'notes'], disabledElements: [] },
    });
  });

  it('turns a page off and takes it out of the bar', () => {
    const placed = placePage(catalog, layout, 'notes', 'off');
    expect(placed).toEqual({
      ok: true,
      patch: { tabBar: ['home'], disabledElements: ['clipboard:page', 'notes:page'] },
    });
  });

  it('keeps home in the bar and the bar within its limit', () => {
    expect(placePage(catalog, layout, 'home', 'other')).toEqual({ ok: false, reason: 'locked' });
    const many = [...catalog, page('a', 4), page('b', 5), page('c', 6)];
    const full = {
      disabledElements: [],
      tabBar: ['home', 'filesize', 'notes', 'clipboard', 'a', 'b'],
      favorites: [],
    };
    expect(placePage(many, full, 'c', 'bar')).toEqual({ ok: false, reason: 'barFull' });
  });

  it('reorders tabs without passing home', () => {
    const bar = { disabledElements: [], tabBar: ['home', 'notes', 'filesize'], favorites: [] };
    expect(moveInBar(catalog, bar, 'filesize', -1)).toEqual(['home', 'filesize', 'notes']);
    expect(moveInBar(catalog, bar, 'notes', -1)).toBeNull();
    expect(moveInBar(catalog, bar, 'filesize', 1)).toBeNull();
  });
});
