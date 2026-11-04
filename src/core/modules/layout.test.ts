import { describe, expect, it } from 'vitest';
import { launcherIds, tabBarIds, toggleFavorite } from './layout';
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
