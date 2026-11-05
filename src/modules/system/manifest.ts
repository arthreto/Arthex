import { defineModule } from '../../core/modules/manifest';
import { systemSearch } from './search';

export default defineModule({
  id: 'system',
  icon: 'M12 3v9M6.3 6.3a8 8 0 1 0 11.4 0',
  tint: 'red',
  order: 70,
  search: systemSearch,
  badge: () => import('./SchedulePill.svelte'),
});
