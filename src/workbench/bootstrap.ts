import { globalExtensionRegistry } from '@paulbreuler/shell';
import { extensions } from './extensions';

let bootstrapped = false;

export async function bootstrapWorkbench(): Promise<void> {
  if (bootstrapped) return;
  await Promise.all(
    extensions.map((ext) => globalExtensionRegistry.activate(ext)),
  );
  bootstrapped = true;
}
