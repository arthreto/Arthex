import type { ModuleManifest } from './manifest';

const found = import.meta.glob<{ default: ModuleManifest }>('../../modules/*/manifest.ts', { eager: true });

/** Every module of `src/modules/*`, discovered at build time. */
const builtIn: readonly ModuleManifest[] = Object.values(found).map((entry) => entry.default);
const builtInIds = new Set(builtIn.map((module) => module.id));

const byOrder = (a: ModuleManifest, b: ModuleManifest) => a.order - b.order;

let all = $state.raw<readonly ModuleManifest[]>([...builtIn].sort(byOrder));

/** Built-in modules plus the installed ones that loaded. */
export const registry = {
  get all(): readonly ModuleManifest[] {
    return all;
  },

  isBuiltIn(id: string): boolean {
    return builtInIds.has(id);
  },

  /** An installed module never replaces a built-in one with the same id. */
  setInstalled(installed: readonly ModuleManifest[]): void {
    all = [...builtIn, ...installed.filter((module) => !builtInIds.has(module.id))].sort(byOrder);
  },
};
