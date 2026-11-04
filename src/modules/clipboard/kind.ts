export type ClipKind = 'link' | 'path' | 'color' | 'code' | 'text';

const LINK = /^https?:\/\/\S+$/i;
const PATH = /^([a-z]:\\|\\\\|~?\/)[^\n]*$/i;
const COLOR = /^(#([0-9a-f]{3}|[0-9a-f]{4}|[0-9a-f]{6}|[0-9a-f]{8})|(rgb|hsl)a?\([^)]*\))$/i;
const CODE =
  /[{};]\s*$|^\s*(npm|npx|pnpm|yarn|git|cargo|cd|sudo|docker|select|insert|update|delete|const|let|import|function|def|class|fn|pub)\b|=>|::|\$\(|<\/?[a-z]+>/im;

/** What a copied text looks like, to label it and show code and paths in a fixed-width font. */
export function clipKind(text: string): ClipKind {
  const trimmed = text.trim();
  if (LINK.test(trimmed)) return 'link';
  if (COLOR.test(trimmed)) return 'color';
  if (PATH.test(trimmed)) return 'path';
  if (CODE.test(trimmed)) return 'code';
  return 'text';
}
