import type { Extension } from '@paulbreuler/shell';
import { globalExtensionRegistry } from '@paulbreuler/shell';
import { GROVE_DARK } from './themes';

/**
 * Core extension that registers Grove's theme and global contributions.
 * Future extensions (Home, Workspace, etc.) will be activated separately.
 */
const groveCoreExtension: Extension = {
  id: 'grove.core',
  name: 'Grove Core',
  activate(ctx) {
    ctx.registerTheme(GROVE_DARK);
  },
};

let initialized = false;

/**
 * Initialize the Alder Shell for Grove.
 *
 * Activates the core extension (theme registration) and will be the
 * entry point for activating feature extensions in future cycles.
 *
 * Idempotent: safe to call multiple times (only activates once).
 */
export async function bootstrapShell(): Promise<void> {
  if (initialized) return;
  initialized = true;
  await globalExtensionRegistry.activate(groveCoreExtension);
}
