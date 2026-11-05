import type { Component } from 'svelte';
import type { Tint } from '../modules/manifest';

export type ResultIcon =
  | { kind: 'image'; src: string }
  | { kind: 'letter'; text: string }
  | { kind: 'folder' }
  | { kind: 'file' }
  | { kind: 'calc' }
  | { kind: 'command'; danger?: boolean }
  /** A module's own pictogram, such as a tool's. */
  | { kind: 'symbol'; path: string; tint: Tint };

export interface ResultAction {
  label: string;
  run: () => unknown;
}

export interface SearchResult {
  /** Unique among all results, e.g. `apps:Microsoft.WindowsCalculator`. */
  key: string;
  /** Translated title of the group the result belongs to. */
  group: string;
  title: string;
  subtitle?: string;
  meta?: string;
  icon: ResultIcon;
  /** Translated kind shown above the title in the preview ("Application"). */
  kind: string;
  details?: { label: string; value: string }[];
  /** Enter. */
  primary: ResultAction;
  /** Ctrl + Enter, such as "Open folder". */
  secondary?: ResultAction;
  /** Text offered by a "Copy path" button. */
  path?: string;
  /** Asked before `primary` runs, for actions that cannot be undone. */
  confirm?: { text: string; label: string };
  /** Replaces the details and buttons of the preview (the shutdown planner). */
  panel?: Component;
}

export interface Query {
  /** As typed, trimmed. */
  text: string;
  /** Lower case without accents. */
  normalized: string;
  /** `>` at the start asks for system commands only. */
  commandsOnly: boolean;
}

export interface SearchProvider {
  /** Groups appear in this order, lowest first. */
  order: number;
  /** When it returns results, other providers are not shown (a calculation). */
  exclusive?: boolean;
  search(query: Query): SearchResult[] | Promise<SearchResult[]>;
}
