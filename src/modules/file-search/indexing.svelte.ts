import { fileSearchApi, type Building } from './api';

/** The first indexing, while no file can be found yet. */
class IndexingState {
  current = $state<Building | null>(null);
  #watching = false;

  watch(): void {
    if (this.#watching) return;
    this.#watching = true;
    void fileSearchApi.onBuilding((building) => (this.current = building));
    void fileSearchApi.building().then((building) => (this.current = building));
  }
}

export const indexing = new IndexingState();
