import { defineModule } from '../../core/modules/manifest';

export default defineModule({
  id: 'filesize',
  kind: 'page',
  icon: 'M3 13h18v6a1.5 1.5 0 0 1-1.5 1.5h-15A1.5 1.5 0 0 1 3 19zM5 13l2.5-8.5h9L19 13M7 17h.01',
  tint: 'amber',
  order: 10,
  page: () => import('./FileSizePage.svelte'),
  widget: { slot: 'side', component: () => import('./StorageWidget.svelte') },
});
