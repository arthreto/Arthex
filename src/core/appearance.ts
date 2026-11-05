import type { Theme } from './settings/types';

const prefersLight = window.matchMedia('(prefers-color-scheme: light)');

export function applyAppearance(theme: Theme, translucent: boolean, reduceMotion: boolean): void {
  const root = document.documentElement;
  root.dataset.theme = theme === 'system' ? (prefersLight.matches ? 'light' : 'dark') : theme;
  root.dataset.backdrop = translucent ? 'translucent' : 'solid';
  root.dataset.motion = reduceMotion ? 'reduced' : 'full';
}

export function onSystemThemeChange(listener: () => void): () => void {
  prefersLight.addEventListener('change', listener);
  return () => prefersLight.removeEventListener('change', listener);
}
