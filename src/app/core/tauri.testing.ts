import { vi, type Mock } from 'vitest';

/**
 * Installs a spy as the Tauri IPC entry point used by `src/app/core/tauri.ts`.
 *
 * The Angular unit-test builder bundles specs before Vitest sees them, so
 * `vi.mock('@tauri-apps/api/core', ...)` does not work (angular/angular-cli#31609).
 * Instead, application code calls `window.__TAURI_INTERNALS__.invoke`, which this
 * helper stubs. The returned spy is the same `invoke` the application uses, so
 * tests can drive it with `mockResolvedValue`/`mockRejectedValue` and assert on
 * its calls just as they would a `vi.fn()`.
 */
export function installTauriInvokeMock(): Mock {
  const invoke = vi.fn();
  (window as unknown as { __TAURI_INTERNALS__: { invoke: Mock } }).__TAURI_INTERNALS__ = { invoke };
  return invoke;
}
