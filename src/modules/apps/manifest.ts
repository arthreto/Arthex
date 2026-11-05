import { defineModule } from '../../core/modules/manifest';
import { appsSearch } from './search';

export default defineModule({
  id: 'apps',
  icon: 'M4 4h6v6H4zM14 4h6v6h-6zM4 14h6v6H4zM14 14h6v6h-6z',
  tint: 'blue',
  order: 20,
  search: appsSearch,
});
