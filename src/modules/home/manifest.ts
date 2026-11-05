import { defineModule } from '../../core/modules/manifest';

export default defineModule({
  id: 'home',
  icon: 'M3.5 11 12 4l8.5 7M6 9.5V20h12V9.5M10 20v-5h4v5',
  tint: 'blue',
  order: 0,
  locked: true,
  page: () => import('./HomePage.svelte'),
});
