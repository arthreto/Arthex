import { defineModule } from '../../core/modules/manifest';

export default defineModule({
  id: 'clipboard',
  icon: 'M9 3h6v3H9zM7 4.5H5.5v16h13v-16H17M9 11h6M9 15h4',
  tint: 'violet',
  order: 12,
  page: () => import('./ClipboardPage.svelte'),
  widget: { slot: 'side', component: () => import('./LastCopiedWidget.svelte') },
});
