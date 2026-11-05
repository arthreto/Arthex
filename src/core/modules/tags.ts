/**
 * The categories of the catalog. The list is closed: a module picks among them and never adds
 * one. Each is translated as `settings.modules.tags.<tag>`.
 */
export const TAGS = [
  'productivity',
  'utilities',
  'developer',
  'math',
  'science',
  'education',
  'games',
  'media',
  'design',
  'writing',
  'files',
  'system',
  'network',
  'time',
  'finance',
  'health',
  'weather',
  'travel',
] as const;
export type Tag = (typeof TAGS)[number];

/** The known tags among `values`, in the order of `TAGS`. */
export const knownTags = (values: unknown): Tag[] =>
  Array.isArray(values) ? TAGS.filter((tag) => values.includes(tag)) : [];
