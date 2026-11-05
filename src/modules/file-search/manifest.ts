import { defineModule } from '../../core/modules/manifest';
import { rescan } from './rescan';
import { fileSearch } from './search';

export default defineModule({
  id: 'file-search',
  icon: 'M10.5 4a6.5 6.5 0 1 0 0 13a6.5 6.5 0 1 0 0-13M20 20l-4.5-4.5',
  tint: 'pink',
  order: 25,
  search: fileSearch,
  settingsSection: {
    component: () => import('./FileSearchSettings.svelte'),
    action: rescan,
  },
});
