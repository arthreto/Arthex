import { normalize } from '../../core/text';

export interface Engine {
  name: string;
  /** Where the typed words go, already encoded. */
  url: (terms: string, language: string) => string;
}

/** Typed before the words: "yt lofi" searches YouTube. */
export const ENGINES: Record<string, Engine> = {
  g: { name: 'Google', url: (terms) => `https://www.google.com/search?q=${terms}` },
  yt: { name: 'YouTube', url: (terms) => `https://www.youtube.com/results?search_query=${terms}` },
  w: {
    name: 'Wikipedia',
    url: (terms, language) => `https://${language}.wikipedia.org/w/index.php?search=${terms}`,
  },
  gh: { name: 'GitHub', url: (terms) => `https://github.com/search?q=${terms}` },
  ddg: { name: 'DuckDuckGo', url: (terms) => `https://duckduckgo.com/?q=${terms}` },
};

export const DEFAULT_ENGINE = 'g';

/** Well-known sites, found by typing the start of their name. */
export const WEBSITES: readonly { name: string; url: string }[] = [
  { name: 'YouTube', url: 'https://www.youtube.com' },
  { name: 'Google', url: 'https://www.google.com' },
  { name: 'Gmail', url: 'https://mail.google.com' },
  { name: 'GitHub', url: 'https://github.com' },
  { name: 'Wikipedia', url: 'https://www.wikipedia.org' },
  { name: 'Reddit', url: 'https://www.reddit.com' },
  { name: 'Netflix', url: 'https://www.netflix.com' },
  { name: 'Twitch', url: 'https://www.twitch.tv' },
  { name: 'Amazon', url: 'https://www.amazon.com' },
  { name: 'LinkedIn', url: 'https://www.linkedin.com' },
  { name: 'Instagram', url: 'https://www.instagram.com' },
  { name: 'Discord', url: 'https://discord.com/app' },
  { name: 'Spotify', url: 'https://open.spotify.com' },
  { name: 'DeepL', url: 'https://www.deepl.com/translator' },
];

const TLDS = 'com|org|net|fr|be|ch|ca|de|uk|eu|io|dev|app|ai|gg|tv|me|co|info|edu|gov';
const ADDRESS = new RegExp(`^(?:https?://)?(?:[a-z0-9-]+\\.)+(?:${TLDS})(?::\\d+)?(?:/\\S*)?$`, 'i');

/** "yt lofi" → YouTube and "lofi"; anything else → null. */
export function keyword(text: string): { engine: Engine; terms: string } | null {
  const match = /^(\S+)\s+(.+)$/.exec(text.trim());
  const engine = match && ENGINES[match[1]?.toLowerCase() ?? ''];
  return engine && match[2] ? { engine, terms: match[2] } : null;
}

/** A typed web address, as a full `https://` URL. Files such as "report.pdf" are not addresses. */
export function address(text: string): string | null {
  const typed = text.trim();
  if (!ADDRESS.test(typed)) return null;
  return /^https?:\/\//i.test(typed) ? typed : `https://${typed}`;
}

export function knownSites(query: string): { name: string; url: string }[] {
  const typed = normalize(query.trim());
  if (typed.length < 3) return [];
  return WEBSITES.filter((site) => normalize(site.name).startsWith(typed)).slice(0, 2);
}

export function searchUrl(engine: Engine, terms: string, language: string): string {
  return engine.url(encodeURIComponent(terms.trim()), language);
}
