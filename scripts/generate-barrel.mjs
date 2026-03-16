/**
 * Generate barrel (index.ts) for ts-rs generated types.
 * Run after `cargo test -p grove-domain --features ts` with TS_RS_EXPORT_DIR set.
 */
import { readdirSync, writeFileSync } from 'node:fs';
import { join } from 'node:path';

const dir = 'src/generated';
const files = readdirSync(dir)
  .filter(f => f.endsWith('.ts') && f !== 'index.ts')
  .sort();

const lines = [
  '// Auto-generated barrel file — do not edit manually.',
  `// Regenerate with: pnpm generate:types`,
  '',
  ...files.map(f => `export * from './${f.replace('.ts', '')}';`),
  '',
];

writeFileSync(join(dir, 'index.ts'), lines.join('\n'));
console.log(`  Barrel: ${files.length} modules re-exported in src/generated/index.ts`);
