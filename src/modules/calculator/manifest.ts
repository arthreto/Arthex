import { defineModule } from '../../core/modules/manifest';
import { calculatorSearch } from './search';

export default defineModule({
  id: 'calculator',
  kind: 'feature',
  icon: 'M6 3h12v18H6zM9 7h6M9 12h.01M12 12h.01M15 12h.01M9 16h.01M12 16h.01M15 16h.01',
  tint: 'sky',
  order: 60,
  search: calculatorSearch,
});
