/** Pictograms a shortcut can show instead of an image, as 24×24 stroked paths. */
export const SYMBOLS = {
  folder: 'M3 7a2 2 0 0 1 2-2h4l2 2h8a2 2 0 0 1 2 2v8a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2z',
  terminal: 'M4 5h16v14H4zM8 10l2.5 2L8 14M12.5 14H16',
  star: 'M12 3.5l2.6 5.3 5.9.9-4.3 4.1 1 5.8-5.2-2.7-5.2 2.7 1-5.8-4.3-4.1 5.9-.9z',
  bolt: 'M13 3 5 13.5h6L10 21l8-10.5h-6z',
  code: 'M8 8l-4 4 4 4M16 8l4 4-4 4M13.5 5l-3 14',
  music: 'M9 18V5l10-2v13M9 18a3 3 0 1 1-6 0a3 3 0 1 1 6 0M19 16a3 3 0 1 1-6 0a3 3 0 1 1 6 0',
  game: 'M6 8h12a3 3 0 0 1 3 3v3a3 3 0 0 1-5.2 2L14 14h-4l-1.8 2A3 3 0 0 1 3 14v-3a3 3 0 0 1 3-3zM8 10.5v3M6.5 12h3M15 11h.01M17 13h.01',
  mail: 'M3.5 6h17v12h-17zM4 7l8 6 8-6',
  book: 'M5 4.5h10a3 3 0 0 1 3 3V20H8a3 3 0 0 1-3-3zM5 17a3 3 0 0 1 3-3h10',
  heart: 'M12 20s-7-4.4-7-10a4 4 0 0 1 7-2.6A4 4 0 0 1 19 10c0 5.6-7 10-7 10z',
  settings:
    'M12 9a3 3 0 1 0 0 6a3 3 0 1 0 0-6M12 3v2M12 19v2M3 12h2M19 12h2M5.6 5.6 7 7M17 17l1.4 1.4M5.6 18.4 7 17M17 7l1.4-1.4',
  globe:
    'M12 3a9 9 0 1 0 0 18a9 9 0 1 0 0-18M3 12h18M12 3c2.4 2.6 3.6 5.6 3.6 9s-1.2 6.4-3.6 9c-2.4-2.6-3.6-5.6-3.6-9S9.6 5.6 12 3',
} as const;

export type SymbolName = keyof typeof SYMBOLS;
export const SYMBOL_NAMES = Object.keys(SYMBOLS) as SymbolName[];

export function symbolPath(name: string): string {
  return SYMBOLS[name as SymbolName] ?? SYMBOLS.globe;
}
