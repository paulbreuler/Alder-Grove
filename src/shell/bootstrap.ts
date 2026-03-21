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

/** All first-party extensions, in activation order. */
const extensions: Extension[] = [
  groveCoreExtension,
  // Future extensions added here:
  // homeExtension,
  // workspaceExtension,
  // personasExtension,
];

/**
 * Initialize the Alder Shell for Grove.
 *
 * Activates first-party extensions and registers their contributions
 * with the shell. The core extension (theme) is always activated;
 * feature extensions can be filtered via the `isEnabled` callback.
 *
 * @param isEnabled - Optional policy callback. Called with each extension's
 *   `id`; return `false` to skip activation (e.g., tier-based feature gating).
 *   Defaults to `() => true` (all extensions active).
 *
 * Idempotent: safe to call multiple times (only activates once).
 */
export async function bootstrapShell(
  isEnabled: (extensionId: string) => boolean = () => true,
): Promise<void> {
  if (initialized) return;
  initialized = true;

  const enabled = extensions.filter((ext) => isEnabled(ext.id));
  await Promise.all(
    enabled.map((ext) => globalExtensionRegistry.activate(ext)),
  );
}
