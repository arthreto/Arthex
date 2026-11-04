import { defineModule } from '../../core/modules/manifest';

export default defineModule({
  id: 'recent-apps',
  kind: 'feature',
  icon: 'M12 3a9 9 0 1 0 0 18a9 9 0 1 0 0-18M12 7.5V12l3 2',
  tint: 'orange',
  order: 40,
  widget: { slot: 'main', component: () => import('./RecentAppsWidget.svelte') },
  settingsAction: () => import('./ClearHistory.svelte'),
});
