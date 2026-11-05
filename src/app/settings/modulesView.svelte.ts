import type { ModuleOffer } from '../../core/api';

/** What Settings → Modules keeps while the overlay closes, which rebuilds its content. */
export const modulesView = $state<{
  tab: 'configuration' | 'catalog';
  /** `null` until the catalog has loaded once. */
  offers: ModuleOffer[] | null;
}>({ tab: 'configuration', offers: null });
