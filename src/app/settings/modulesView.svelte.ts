import { tick } from 'svelte';
import { commands, type ModuleOffer } from '../../core/api';
import { t } from '../../core/i18n';
import { toast } from '../../core/toast.svelte';
import { moduleHost } from '../moduleHost.svelte';

/** Long enough that pressing ↻ again and again does not hammer the network. */
const REFRESH_REST_MS = 4000;

/** What Settings → Modules keeps while the overlay closes, which rebuilds its content. */
class ModulesView {
  tab = $state<'configuration' | 'catalog'>('configuration');
  /** `null` until the catalog has loaded once. */
  offers = $state<ModuleOffer[] | null>(null);
  failed = $state(false);
  /** The kept catalog answers within a frame or two: the shimmer would only flash before it. */
  lookedForKept = $state(false);
  refreshing = $state(false);
  /** Just after a refresh, when ↻ stays grey. */
  resting = $state(false);
  /** Images of modules, by id: nothing while one loads, `null` when it could not be read. */
  previews = $state<Record<string, string | null | undefined>>({});
  /** True while the rows of the first answer are being added, so that they appear in turn. */
  arriving = false;
  private readonly previewsAsked = new Set<string>();

  async showKept(): Promise<void> {
    const kept = await commands.modulesCatalogKept().catch(() => null);
    this.offers ??= kept;
    this.lookedForKept = true;
  }

  async load(): Promise<void> {
    this.failed = false;
    this.arriving = this.offers === null;
    try {
      this.offers = await commands.modulesCatalog();
    } catch {
      this.failed = true;
      this.offers ??= [];
    }
    await tick();
    this.arriving = false;
  }

  /** ↻: the installed modules and the catalog, and whether updates came with it. */
  async refresh(): Promise<void> {
    if (this.refreshing || this.resting) return;
    this.refreshing = true;
    await Promise.all([this.load(), moduleHost.load()]);
    this.refreshing = false;
    this.resting = true;
    setTimeout(() => (this.resting = false), REFRESH_REST_MS);
    if (this.failed) return;
    const count = (this.offers ?? []).filter((offer) => offer.state === 'update').length;
    if (count === 0) toast.show(t('settings.modules.upToDate'));
    else if (count === 1) toast.show(t('settings.modules.updateFound'));
    else toast.show(t('settings.modules.updatesFound', { count }));
  }

  markOffer(id: string, state: ModuleOffer['state']): void {
    this.offers = (this.offers ?? []).map((offer) => (offer.id === id ? { ...offer, state } : offer));
  }

  async askPreview(id: string): Promise<void> {
    if (this.previewsAsked.has(id)) return;
    this.previewsAsked.add(id);
    this.previews[id] = undefined;
    const preview = await commands.modulesPreview(id).catch(() => null);
    this.previews[id] = preview;
    // Opening the details again tries once more, once the network is back.
    if (preview === null) this.previewsAsked.delete(id);
  }
}

export const modulesView = new ModulesView();
