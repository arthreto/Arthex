import { errorMessage } from '../core/api';
import { t } from '../core/i18n';
import { searchProviders } from '../core/modules';
import { mergeResults, parseQuery } from '../core/search/query';
import type { SearchResult } from '../core/search/types';
import { settings } from '../core/settings/settings.svelte';
import { toast } from '../core/toast.svelte';
import { toolSearch } from './toolSearch';

/** Fewer letters bring too many results to be useful, except for commands and calculations. */
export const MIN_LENGTH = 2;

class SearchState {
  query = $state('');
  results = $state<SearchResult[]>([]);
  selected = $state(0);
  /** A result waiting for the user to confirm its action. */
  confirming = $state<SearchResult | null>(null);
  /** Providers still answering the current query. */
  pending = $state(0);

  #run = 0;

  /** Too short to search: the view asks for more letters. Commands and calculations always run. */
  get tooShort(): boolean {
    const query = parseQuery(this.query);
    return !query.commandsOnly && query.text.length < MIN_LENGTH && !/\d/.test(query.text);
  }

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

  /**
   * Asks every enabled provider and shows each answer as it comes, so a slow one (the app list
   * being read) never holds back the others. Answers to an older query are dropped.
   */
  async update(): Promise<void> {
    const run = ++this.#run;
    const query = parseQuery(this.query);
    this.confirming = null;
    if (this.tooShort) {
      this.results = [];
      this.pending = 0;
      return;
    }
    const providers = [toolSearch, ...searchProviders(settings.value)];
    const answers: Parameters<typeof mergeResults>[0] = [];
    let first = true;
    this.pending = providers.length;
    await Promise.all(
      providers.map(async (provider) => {
        let results: SearchResult[] = [];
        try {
          results = await provider.search(query);
        } catch {
          // A failing provider only loses its own group.
        }
        if (run !== this.#run) return;
        answers.push({ order: provider.order, exclusive: provider.exclusive ?? false, results });
        this.pending -= 1;
        this.#show(mergeResults(answers), first);
        first = false;
      }),
    );
  }

  /** Keeps the chosen result chosen when a later answer adds results above it. */
  #show(results: SearchResult[], fresh: boolean): void {
    const chosen = fresh ? undefined : this.current?.key;
    this.results = results;
    this.selected = Math.max(0, chosen ? results.findIndex((result) => result.key === chosen) : 0);
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
