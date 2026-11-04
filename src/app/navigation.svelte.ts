import { HOME_ID, getModule } from '../core/modules';

export type View = { kind: 'page'; id: string } | { kind: 'hub' } | { kind: 'settings' };

class Navigation {
  view = $state<View>({ kind: 'page', id: HOME_ID });

  open(view: View): void {
    this.view = view;
  }

  /** Target sent by Rust (tray menu): `hub`, `settings` or a page id. */
  openTarget(target: string): void {
    if (target === 'hub' || target === 'settings') this.open({ kind: target });
    else if (getModule(target)?.kind === 'page') this.open({ kind: 'page', id: target });
  }

  isPage(id: string): boolean {
    return this.view.kind === 'page' && this.view.id === id;
  }
}

export const navigation = new Navigation();
