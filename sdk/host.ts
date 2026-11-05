import type { HostApi } from '../src/core/modules/foreign';

/** The shell's side, reached only when a view or a call needs it, never while the module loads. */
export function host(): HostApi {
  const api = (globalThis as { __ARTHEX__?: HostApi }).__ARTHEX__;
  if (!api) throw new Error('This module runs inside Arthex only.');
  return api;
}
