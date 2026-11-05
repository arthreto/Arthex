import { defineModule } from '../../core/modules/manifest';
import { shortcutsSearch } from './search';

export default defineModule({
  id: 'shortcuts',
  icon: 'M12 3a9 9 0 1 0 0 18a9 9 0 1 0 0-18M3 12h18M12 3c2.4 2.6 3.6 5.6 3.6 9s-1.2 6.4-3.6 9c-2.4-2.6-3.6-5.6-3.6-9S9.6 5.6 12 3',
  tint: 'mint',
  order: 30,
  widget: { slot: 'top', component: () => import('./ShortcutsWidget.svelte') },
  search: shortcutsSearch,
});
