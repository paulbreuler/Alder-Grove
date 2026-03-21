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

/** All first-party feature extensions (excluding core). */
const featureExtensions: Extension[] = [
  // Future extensions added here:
  // homeExtension,
  // workspaceExtension,
  // personasExtension,
];

let bootstrapPromise: Promise<void> | null = null;

/**
 * Initialize the Alder Shell for Grove.
 *
 * Activates first-party extensions and registers their contributions
 * with the shell. The core extension (theme) is always activated;
 * feature extensions can be filtered via the `isEnabled` callback.
 *
 * @param isEnabled - Optional policy callback. Called with each feature
 *   extension's `id`; return `false` to skip activation (e.g., tier-based
 *   feature gating). Does not affect the core extension.
 *   Defaults to `() => true` (all feature extensions active).
 *
 * Idempotent: safe to call multiple times (returns the same Promise).
 */
export function bootstrapShell(
  isEnabled: (extensionId: string) => boolean = () => true,
): Promise<void> {
  if (bootstrapPromise) return bootstrapPromise;
  bootstrapPromise = doBootstrap(isEnabled);
  return bootstrapPromise;
}

async function doBootstrap(
  isEnabled: (extensionId: string) => boolean,
): Promise<void> {
  // Core is always activated first — feature extensions may depend on it.
  await globalExtensionRegistry.activate(groveCoreExtension);

  // Activate feature extensions sequentially; later extensions may
  // depend on contributions registered by earlier ones.
  const enabled = featureExtensions.filter((ext) => isEnabled(ext.id));
  for (const ext of enabled) {
    await globalExtensionRegistry.activate(ext);
  }
}
