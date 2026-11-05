import { errorMessage } from '../core/api';
import { t } from '../core/i18n';
import { searchProviders } from '../core/modules';
import { mergeResults, parseQuery } from '../core/search/query';
import type { SearchResult } from '../core/search/types';
import { settings } from '../core/settings/settings.svelte';
import { toast } from '../core/toast.svelte';
import { toolSearch } from './toolSearch';

/** Fewer letters bring too many results to be useful, except for commands and calculations. */
const MIN_LENGTH = 2;

class SearchState {
  query = $state('');
  results = $state<SearchResult[]>([]);
  selected = $state(0);
  /** A result waiting for the user to confirm its action. */
  confirming = $state<SearchResult | null>(null);
  #run = 0;

  get active(): boolean {
    return this.query.trim().length > 0;
  }

  get current(): SearchResult | null {
    return this.results[this.selected] ?? null;
  }

  clear(): void {
    this.query = '';
    this.results = [];
    this.selected = 0;
    this.confirming = null;
  }

  /** Asks every enabled provider; a slower answer to an older query is dropped. */
  async update(): Promise<void> {
    const run = ++this.#run;
    const query = parseQuery(this.query);
    this.confirming = null;
    if (!query.commandsOnly && query.text.length < MIN_LENGTH && !/\d/.test(query.text)) {
      this.results = [];
      return;
    }
    const answers = await Promise.all(
      [toolSearch, ...searchProviders(settings.value)].map(async (provider) => {
        try {
          return {
            order: provider.order,
            exclusive: provider.exclusive ?? false,
            results: await provider.search(query),
          };
        } catch {
          return { order: provider.order, exclusive: false, results: [] };
        }
      }),
    );
    if (run !== this.#run) return;
    this.results = mergeResults(answers);
    this.selected = 0;
  }

  move(step: number): void {
    if (this.results.length === 0) return;
    this.selected = Math.max(0, Math.min(this.results.length - 1, this.selected + step));
    this.confirming = null;
  }

  /** Runs the main action (or the secondary one), asking first when the result requires it. */
  async activate(result: SearchResult | null = this.current, secondary = false): Promise<void> {
    if (!result) return;
    const action = secondary ? result.secondary : result.primary;
    if (!action) return;
    if (!secondary && result.confirm && this.confirming !== result) {
      this.confirming = result;
      return;
    }
    this.confirming = null;
    try {
      await action.run();
    } catch (error) {
      toast.show(t('errors.generic', { message: errorMessage(error) }));
    }
  }
}

export const search = new SearchState();
