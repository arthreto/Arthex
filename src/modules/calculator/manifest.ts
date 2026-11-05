import { defineModule } from '../../core/modules/manifest';
import { ICON, ID, TINT } from './identity';
import { calculatorSearch } from './search';

export default defineModule({
  id: ID,
  icon: ICON,
  tint: TINT,
  order: 60,
  search: calculatorSearch,
  tool: () => import('./CalculatorTool.svelte'),
});
