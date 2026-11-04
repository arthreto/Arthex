import { normalize } from '../text';
import type { Query, SearchResult } from './types';

export function parseQuery(input: string): Query {
  const text = input.trim();
  const commandsOnly = text.startsWith('>');
  const rest = commandsOnly ? text.slice(1).trim() : text;
  return { text: rest, normalized: normalize(rest), commandsOnly };
}

/** Keeps provider order, and drops the others when an exclusive provider answered. */
export function mergeResults(
  answers: { order: number; exclusive: boolean; results: SearchResult[] }[],
): SearchResult[] {
  const sorted = [...answers].sort((a, b) => a.order - b.order);
  const exclusive = sorted.find((answer) => answer.exclusive && answer.results.length > 0);
  return exclusive ? exclusive.results : sorted.flatMap((answer) => answer.results);
}
