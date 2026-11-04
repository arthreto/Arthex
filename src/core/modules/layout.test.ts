import { describe, expect, it } from 'vitest';
import { launcherIds, moveInBar, placePage, tabBarIds, toggleFavorite, zoneIds } from './layout';
import type { ModuleManifest } from './manifest';

const page = (id: string, order: number, locked = false): ModuleManifest => ({
  id,
  kind: 'page',
  icon: '',
  tint: 'blue',
  order,
  locked,
});

const catalog = [page('home', 0, true), page('filesize', 1), page('notes', 2), page('clipboard', 3)];

describe('tabBarIds', () => {
  it('keeps home first and drops disabled or unknown pages', () => {
    const layout = {
      modules: { notes: false },
      tabBar: ['notes', 'ghost', 'clipboard', 'home'],
      favorites: [],
    };
    expect(tabBarIds(catalog, layout)).toEqual(['home', 'clipboard']);
  });

  it('never lets the home page be turned off', () => {
    const layout = { modules: { home: false }, tabBar: [], favorites: [] };
    expect(tabBarIds(catalog, layout)).toEqual(['home']);
  });
});

describe('launcherIds', () => {
  it('lists favourites first, then the others, then disabled pages', () => {
    const layout = { modules: { filesize: false }, tabBar: [], favorites: ['clipboard'] };
    expect(launcherIds(catalog, layout)).toEqual(['clipboard', 'home', 'notes', 'filesize']);
  });

  it('ignores favourites that are disabled', () => {
    const layout = { modules: { notes: false }, tabBar: [], favorites: ['notes'] };
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
  const layout = { modules: { clipboard: false }, tabBar: ['home', 'notes'], favorites: [] };

  it('splits pages into the bar, the others and the turned-off ones', () => {
    expect(zoneIds(catalog, layout)).toEqual({
      bar: ['home', 'notes'],
      other: ['filesize'],
      off: ['clipboard'],
    });
  });

  it('inserts into the bar after home and turns pages back on', () => {
    const placed = placePage(catalog, layout, 'clipboard', 'bar', 'home');
    expect(placed).toEqual({ ok: true, patch: { tabBar: ['home', 'clipboard', 'notes'], modules: {} } });
  });

  it('turns a page off and takes it out of the bar', () => {
    const placed = placePage(catalog, layout, 'notes', 'off');
    expect(placed).toEqual({
      ok: true,
      patch: { tabBar: ['home'], modules: { clipboard: false, notes: false } },
    });
  });

  it('keeps home in the bar and the bar within its limit', () => {
    expect(placePage(catalog, layout, 'home', 'other')).toEqual({ ok: false, reason: 'locked' });
    const many = [...catalog, page('a', 4), page('b', 5), page('c', 6)];
    const full = { modules: {}, tabBar: ['home', 'filesize', 'notes', 'clipboard', 'a', 'b'], favorites: [] };
    expect(placePage(many, full, 'c', 'bar')).toEqual({ ok: false, reason: 'barFull' });
  });

  it('reorders tabs without passing home', () => {
    const bar = { modules: {}, tabBar: ['home', 'notes', 'filesize'], favorites: [] };
    expect(moveInBar(catalog, bar, 'filesize', -1)).toEqual(['home', 'filesize', 'notes']);
    expect(moveInBar(catalog, bar, 'notes', -1)).toBeNull();
    expect(moveInBar(catalog, bar, 'filesize', 1)).toBeNull();
  });
});
