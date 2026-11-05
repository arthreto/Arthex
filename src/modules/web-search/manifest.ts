import { defineModule } from '../../core/modules/manifest';
import { ICON, webSearch } from './search';

export default defineModule({
  id: 'web-search',
  icon: ICON,
  tint: 'blue',
  order: 35,
  search: webSearch,
});
