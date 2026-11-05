import { defineModule } from '../../core/modules/manifest';

export default defineModule({
  id: 'notes',
  icon: 'M6 3.5h9l4 4v13H6zM15 3.5v4h4M9 12h7M9 16h5',
  tint: 'green',
  order: 11,
  page: () => import('./NotesPage.svelte'),
  widget: { slot: 'side', component: () => import('./NoteWidget.svelte') },
});
