import type { InvokeArgs, InvokeOptions } from '@tauri-apps/api/core';

interface TauriInternals {
  invoke<T>(cmd: string, ...rest: [args?: InvokeArgs, options?: InvokeOptions]): Promise<T>;
}

/**
 * Thin, testable wrapper around Tauri's IPC entry point.
 *
 * The Angular `@angular/build:unit-test` builder bundles spec files before
 * handing them to Vitest, so `vi.mock('@tauri-apps/api/core', ...)` is silently
 * ignored (see angular/angular-cli#31609) and the imported `invoke` cannot be
 * replaced. This wrapper calls `window.__TAURI_INTERNALS__.invoke` — the same
 * runtime IPC entry that `@tauri-apps/api`'s own `invoke` delegates to — which
 * tests can stub with a spy. Arguments are forwarded verbatim so call
 * assertions observe exactly what application code passed.
 */
export function invoke<T>(cmd: string, ...rest: [args?: InvokeArgs, options?: InvokeOptions]): Promise<T> {
  const internals = (window as unknown as { __TAURI_INTERNALS__: TauriInternals }).__TAURI_INTERNALS__;
  return internals.invoke<T>(cmd, ...rest);
}
